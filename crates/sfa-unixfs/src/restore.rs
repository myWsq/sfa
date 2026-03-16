use crate::error::{PathValidationError, UnixFsError};
use crate::path::{ensure_safe_relative_path, safe_join};
use std::collections::{HashMap, HashSet, VecDeque};
use std::fs::{self, File, OpenOptions};
use std::os::unix::fs::{FileExt, PermissionsExt};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverwritePolicy {
    Error,
    Replace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RestorePolicy {
    pub overwrite: OverwritePolicy,
    pub restore_owner: bool,
    pub max_open_files: usize,
}

impl Default for RestorePolicy {
    fn default() -> Self {
        Self {
            overwrite: OverwritePolicy::Error,
            restore_owner: false,
            max_open_files: 256,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntryMetadata {
    pub mode: u32,
    pub uid: u32,
    pub gid: u32,
    pub mtime_sec: i64,
    pub mtime_nsec: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestoreTarget {
    pub entry_id: u32,
    pub relative_path: PathBuf,
    pub metadata: EntryMetadata,
}

pub trait Restorer {
    fn create_dir(&mut self, target: &RestoreTarget) -> Result<(), UnixFsError>;
    fn ensure_file(&mut self, target: &RestoreTarget) -> Result<(), UnixFsError>;
    fn write_extent(
        &mut self,
        target: &RestoreTarget,
        file_offset: u64,
        buf: &[u8],
    ) -> Result<(), UnixFsError>;
    fn create_symlink(
        &mut self,
        target: &RestoreTarget,
        link_target: &[u8],
    ) -> Result<(), UnixFsError>;
    fn create_hardlink(
        &mut self,
        target: &RestoreTarget,
        master: &RestoreTarget,
    ) -> Result<(), UnixFsError>;
    fn finalize_entry(&mut self, target: &RestoreTarget) -> Result<(), UnixFsError>;
    fn finalize_dirs(&mut self) -> Result<(), UnixFsError>;
}

pub struct LocalRestorer {
    root: PathBuf,
    policy: RestorePolicy,
    files: HashMap<u32, File>,
    created_files: HashSet<u32>,
    open_order: VecDeque<u32>,
    dir_finalize_order: Vec<RestoreTarget>,
}

impl LocalRestorer {
    pub fn new(root: PathBuf, policy: RestorePolicy) -> Self {
        Self {
            root,
            policy,
            files: HashMap::new(),
            created_files: HashSet::new(),
            open_order: VecDeque::new(),
            dir_finalize_order: Vec::new(),
        }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    fn resolve_safe_path(&self, rel: &Path) -> Result<PathBuf, UnixFsError> {
        safe_join(&self.root, rel).map_err(UnixFsError::from)
    }

    fn ensure_parent_dirs(&self, rel: &Path) -> Result<(), UnixFsError> {
        ensure_safe_relative_path(rel)?;
        let parent = rel
            .parent()
            .ok_or_else(|| UnixFsError::MissingParent(rel.to_path_buf()))?;
        let mut current = self.root.clone();
        for segment in parent.components() {
            let seg = segment.as_os_str();
            current = current.join(seg);
            match fs::symlink_metadata(&current) {
                Ok(md) => {
                    if md.file_type().is_symlink() {
                        return Err(PathValidationError::SymlinkTraversal(current).into());
                    }
                    if !md.is_dir() {
                        return Err(PathValidationError::NotADirectory(current).into());
                    }
                }
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                    fs::create_dir(&current)?;
                }
                Err(err) => return Err(err.into()),
            }
        }
        Ok(())
    }

    fn remove_if_exists(&self, path: &Path) -> Result<(), UnixFsError> {
        match fs::symlink_metadata(path) {
            Ok(md) => {
                if md.file_type().is_dir() && !md.file_type().is_symlink() {
                    fs::remove_dir_all(path)?;
                } else {
                    fs::remove_file(path)?;
                }
                Ok(())
            }
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(err) => Err(err.into()),
        }
    }

    fn write_all_at(file: &File, mut offset: u64, mut buf: &[u8]) -> Result<(), UnixFsError> {
        while !buf.is_empty() {
            let written = file.write_at(buf, offset)?;
            if written == 0 {
                return Err(UnixFsError::InvalidState("write_at returned zero"));
            }
            offset += written as u64;
            buf = &buf[written..];
        }
        Ok(())
    }

    fn evict_if_needed(&mut self) {
        while self.files.len() >= self.policy.max_open_files.max(1) {
            if let Some(entry_id) = self.open_order.pop_front() {
                self.files.remove(&entry_id);
            } else {
                break;
            }
        }
    }
}

impl Restorer for LocalRestorer {
    fn create_dir(&mut self, target: &RestoreTarget) -> Result<(), UnixFsError> {
        ensure_safe_relative_path(&target.relative_path)?;
        let path = self.resolve_safe_path(&target.relative_path)?;
        self.ensure_parent_dirs(&target.relative_path)?;

        match fs::symlink_metadata(&path) {
            Ok(md) => {
                if md.file_type().is_symlink() {
                    return Err(PathValidationError::SymlinkTraversal(path).into());
                }
                if !md.is_dir() {
                    match self.policy.overwrite {
                        OverwritePolicy::Error => {
                            return Err(UnixFsError::InvalidState(
                                "directory path exists as non-dir",
                            ));
                        }
                        OverwritePolicy::Replace => {
                            self.remove_if_exists(&path)?;
                            fs::create_dir(&path)?;
                        }
                    }
                }
            }
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                fs::create_dir(&path)?;
            }
            Err(err) => return Err(err.into()),
        }

        self.dir_finalize_order.push(target.clone());
        Ok(())
    }

    fn ensure_file(&mut self, target: &RestoreTarget) -> Result<(), UnixFsError> {
        if self.files.contains_key(&target.entry_id) {
            return Ok(());
        }
        ensure_safe_relative_path(&target.relative_path)?;
        let path = self.resolve_safe_path(&target.relative_path)?;
        self.ensure_parent_dirs(&target.relative_path)?;

        let file = if self.created_files.contains(&target.entry_id) {
            OpenOptions::new().write(true).open(&path)?
        } else {
            if let Ok(md) = fs::symlink_metadata(&path) {
                if md.file_type().is_symlink() {
                    return Err(PathValidationError::SymlinkTraversal(path).into());
                }
                if md.is_dir() {
                    return Err(PathValidationError::NotADirectory(path).into());
                }
                if matches!(self.policy.overwrite, OverwritePolicy::Replace) {
                    self.remove_if_exists(&path)?;
                }
            }

            let mut opts = OpenOptions::new();
            opts.write(true);
            match self.policy.overwrite {
                OverwritePolicy::Error => {
                    opts.create_new(true);
                }
                OverwritePolicy::Replace => {
                    opts.create(true).truncate(true);
                }
            }
            let file = opts.open(&path)?;
            self.created_files.insert(target.entry_id);
            file
        };

        self.evict_if_needed();
        self.files.insert(target.entry_id, file);
        self.open_order.push_back(target.entry_id);
        Ok(())
    }

    fn write_extent(
        &mut self,
        target: &RestoreTarget,
        file_offset: u64,
        buf: &[u8],
    ) -> Result<(), UnixFsError> {
        if !self.files.contains_key(&target.entry_id) {
            self.ensure_file(target)?;
        }
        let file = self
            .files
            .get(&target.entry_id)
            .ok_or(UnixFsError::InvalidState("missing cached file handle"))?;
        Self::write_all_at(file, file_offset, buf)
    }

    fn create_symlink(
        &mut self,
        target: &RestoreTarget,
        link_target: &[u8],
    ) -> Result<(), UnixFsError> {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;
        use std::os::unix::fs::symlink;

        ensure_safe_relative_path(&target.relative_path)?;
        self.ensure_parent_dirs(&target.relative_path)?;
        let link_path = self.resolve_safe_path(&target.relative_path)?;

        if matches!(self.policy.overwrite, OverwritePolicy::Replace) {
            self.remove_if_exists(&link_path)?;
        }
        let target_path = Path::new(OsStr::from_bytes(link_target));
        symlink(target_path, &link_path)?;
        Ok(())
    }

    fn create_hardlink(
        &mut self,
        target: &RestoreTarget,
        master: &RestoreTarget,
    ) -> Result<(), UnixFsError> {
        ensure_safe_relative_path(&target.relative_path)?;
        ensure_safe_relative_path(&master.relative_path)?;
        self.ensure_parent_dirs(&target.relative_path)?;
        let target_path = self.resolve_safe_path(&target.relative_path)?;
        let master_path = self.resolve_safe_path(&master.relative_path)?;
        if !master_path.exists() {
            return Err(UnixFsError::InvalidState("hardlink master does not exist"));
        }
        if matches!(self.policy.overwrite, OverwritePolicy::Replace) {
            self.remove_if_exists(&target_path)?;
        }
        fs::hard_link(master_path, target_path)?;
        Ok(())
    }

    fn finalize_entry(&mut self, target: &RestoreTarget) -> Result<(), UnixFsError> {
        let path = self.resolve_safe_path(&target.relative_path)?;
        let perms = fs::Permissions::from_mode(target.metadata.mode);
        fs::set_permissions(&path, perms)?;
        if self.policy.restore_owner && nix::unistd::Uid::effective().is_root() {
            let uid = nix::unistd::Uid::from_raw(target.metadata.uid);
            let gid = nix::unistd::Gid::from_raw(target.metadata.gid);
            let _ = nix::unistd::chown(&path, Some(uid), Some(gid));
        }
        self.files.remove(&target.entry_id);
        self.open_order
            .retain(|entry_id| *entry_id != target.entry_id);
        Ok(())
    }

    fn finalize_dirs(&mut self) -> Result<(), UnixFsError> {
        for target in self.dir_finalize_order.iter().rev() {
            let path = self.resolve_safe_path(&target.relative_path)?;
            let perms = fs::Permissions::from_mode(target.metadata.mode);
            fs::set_permissions(path, perms)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::symlink;
    use tempfile::TempDir;

    fn meta() -> EntryMetadata {
        EntryMetadata {
            mode: 0o755,
            uid: 0,
            gid: 0,
            mtime_sec: 0,
            mtime_nsec: 0,
        }
    }

    #[test]
    fn restorer_blocks_symlink_traversal() {
        let temp = TempDir::new().expect("temp");
        let root = temp.path().to_path_buf();
        fs::create_dir(root.join("safe")).expect("safe dir");
        symlink("/tmp", root.join("safe/escape")).expect("symlink");

        let mut restorer = LocalRestorer::new(root, RestorePolicy::default());
        let target = RestoreTarget {
            entry_id: 1,
            relative_path: PathBuf::from("safe/escape/payload.txt"),
            metadata: meta(),
        };
        let err = restorer.ensure_file(&target).expect_err("must fail");
        match err {
            UnixFsError::PathValidation(PathValidationError::SymlinkTraversal(_)) => {}
            other => panic!("unexpected error: {other}"),
        }
    }

    #[test]
    fn write_extent_and_hardlink_roundtrip() {
        let temp = TempDir::new().expect("temp");
        let root = temp.path().to_path_buf();
        let mut restorer = LocalRestorer::new(
            root.clone(),
            RestorePolicy {
                overwrite: OverwritePolicy::Replace,
                restore_owner: false,
                max_open_files: 256,
            },
        );

        let file = RestoreTarget {
            entry_id: 10,
            relative_path: PathBuf::from("a/data.bin"),
            metadata: meta(),
        };
        let link = RestoreTarget {
            entry_id: 11,
            relative_path: PathBuf::from("a/data-link.bin"),
            metadata: meta(),
        };

        restorer.write_extent(&file, 0, b"hello ").expect("write 1");
        restorer.write_extent(&file, 6, b"world").expect("write 2");
        restorer.finalize_entry(&file).expect("finalize file");
        restorer.create_hardlink(&link, &file).expect("hardlink");

        let content = fs::read(root.join("a/data.bin")).expect("read file");
        let link_content = fs::read(root.join("a/data-link.bin")).expect("read link");
        assert_eq!(content, b"hello world");
        assert_eq!(link_content, b"hello world");
    }
}
