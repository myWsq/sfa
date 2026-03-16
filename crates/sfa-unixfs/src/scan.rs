use crate::error::UnixFsError;
use crate::path::ensure_safe_relative_path;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryKind {
    Root,
    Directory,
    Regular,
    Symlink,
    Hardlink,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScannedEntry {
    pub entry_id: u32,
    pub parent_id: Option<u32>,
    pub relative_path: PathBuf,
    pub name: Vec<u8>,
    pub kind: EntryKind,
    pub mode: u32,
    pub uid: u32,
    pub gid: u32,
    pub mtime_sec: i64,
    pub mtime_nsec: u32,
    pub size: u64,
    pub dev: u64,
    pub ino: u64,
    pub symlink_target: Option<Vec<u8>>,
    pub hardlink_master: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct ScanResult {
    pub entries: Vec<ScannedEntry>,
    pub directory_entry_ids: Vec<u32>,
    pub regular_entry_ids: Vec<u32>,
}

pub fn scan_tree(root: &Path) -> Result<ScanResult, UnixFsError> {
    let root_meta = fs::symlink_metadata(root)?;
    if !root_meta.is_dir() {
        return Err(UnixFsError::UnsupportedEntryKind(root.to_path_buf()));
    }

    let root_entry = ScannedEntry {
        entry_id: 0,
        parent_id: None,
        relative_path: PathBuf::new(),
        name: Vec::new(),
        kind: EntryKind::Root,
        mode: root_meta.mode(),
        uid: root_meta.uid(),
        gid: root_meta.gid(),
        mtime_sec: root_meta.mtime(),
        mtime_nsec: root_meta.mtime_nsec() as u32,
        size: 0,
        dev: root_meta.dev(),
        ino: root_meta.ino(),
        symlink_target: None,
        hardlink_master: None,
    };

    let mut entries = vec![root_entry];
    let mut directory_entry_ids = Vec::new();
    let mut regular_entry_ids = Vec::new();
    let mut hardlink_master = HashMap::<(u64, u64), u32>::new();

    scan_dir_recursive(
        root,
        0,
        Path::new(""),
        &mut entries,
        &mut directory_entry_ids,
        &mut regular_entry_ids,
        &mut hardlink_master,
    )?;

    Ok(ScanResult {
        entries,
        directory_entry_ids,
        regular_entry_ids,
    })
}

fn scan_dir_recursive(
    fs_dir: &Path,
    parent_id: u32,
    rel_dir: &Path,
    entries: &mut Vec<ScannedEntry>,
    directories: &mut Vec<u32>,
    regulars: &mut Vec<u32>,
    hardlink_master: &mut HashMap<(u64, u64), u32>,
) -> Result<(), UnixFsError> {
    let mut children = fs::read_dir(fs_dir)?
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .map(|de| de.file_name())
        .collect::<Vec<_>>();

    children.sort_by(|a, b| a.as_os_str().as_bytes().cmp(b.as_os_str().as_bytes()));

    for child_name in &children {
        let child_fs_path = fs_dir.join(child_name);
        let child_rel_path = rel_dir.join(child_name);
        ensure_safe_relative_path(&child_rel_path)?;
        let child_meta = fs::symlink_metadata(&child_fs_path)?;
        let file_type = child_meta.file_type();
        let entry_id = entries.len() as u32;
        let mut kind = EntryKind::Regular;
        let mut symlink_target = None;
        let mut hardlink_target = None;

        if file_type.is_dir() {
            kind = EntryKind::Directory;
        } else if file_type.is_symlink() {
            kind = EntryKind::Symlink;
            symlink_target = Some(read_link_target_bytes(&child_fs_path)?);
        } else if file_type.is_file() {
            if child_meta.nlink() > 1 {
                let key = (child_meta.dev(), child_meta.ino());
                if let Some(master) = hardlink_master.get(&key).copied() {
                    kind = EntryKind::Hardlink;
                    hardlink_target = Some(master);
                } else {
                    hardlink_master.insert(key, entry_id);
                }
            }
        } else {
            return Err(UnixFsError::UnsupportedEntryKind(child_rel_path));
        }

        let entry = ScannedEntry {
            entry_id,
            parent_id: Some(parent_id),
            relative_path: child_rel_path.clone(),
            name: child_name.as_os_str().as_bytes().to_vec(),
            kind,
            mode: child_meta.mode(),
            uid: child_meta.uid(),
            gid: child_meta.gid(),
            mtime_sec: child_meta.mtime(),
            mtime_nsec: child_meta.mtime_nsec() as u32,
            size: child_meta.size(),
            dev: child_meta.dev(),
            ino: child_meta.ino(),
            symlink_target,
            hardlink_master: hardlink_target,
        };

        entries.push(entry);

        match kind {
            EntryKind::Directory => {
                directories.push(entry_id);
                scan_dir_recursive(
                    &child_fs_path,
                    entry_id,
                    &child_rel_path,
                    entries,
                    directories,
                    regulars,
                    hardlink_master,
                )?;
            }
            EntryKind::Regular => regulars.push(entry_id),
            EntryKind::Root | EntryKind::Symlink | EntryKind::Hardlink => {}
        }
    }

    Ok(())
}

fn read_link_target_bytes(path: &Path) -> Result<Vec<u8>, UnixFsError> {
    let target = fs::read_link(path)?;
    Ok(target.as_os_str().as_bytes().to_vec())
}

fn _segment_bytes(seg: &OsStr) -> &[u8] {
    seg.as_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::symlink;
    use tempfile::TempDir;

    #[test]
    fn scan_is_deterministic_and_detects_hardlinks() {
        let tmp = TempDir::new().expect("tempdir");
        let root = tmp.path();
        fs::create_dir(root.join("dir")).expect("dir");
        fs::write(root.join("b.txt"), b"b").expect("write b");
        fs::write(root.join("a.txt"), b"a").expect("write a");
        fs::write(root.join("hard-master.txt"), b"same").expect("write hard master");
        fs::hard_link(root.join("hard-master.txt"), root.join("hard-peer.txt")).expect("hardlink");
        symlink("a.txt", root.join("sym-a")).expect("symlink");
        fs::write(root.join("dir/child.txt"), b"child").expect("child");

        let run1 = scan_tree(root).expect("scan1");
        let run2 = scan_tree(root).expect("scan2");
        let list1 = run1
            .entries
            .iter()
            .map(|e| e.relative_path.to_string_lossy().into_owned())
            .collect::<Vec<_>>();
        let list2 = run2
            .entries
            .iter()
            .map(|e| e.relative_path.to_string_lossy().into_owned())
            .collect::<Vec<_>>();
        assert_eq!(list1, list2);

        let hard_entries = run1
            .entries
            .iter()
            .filter(|e| matches!(e.kind, EntryKind::Hardlink))
            .collect::<Vec<_>>();
        assert_eq!(hard_entries.len(), 1);
        assert!(hard_entries[0].hardlink_master.is_some());
    }
}
