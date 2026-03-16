use crate::diagnostics::UnpackDiagnosticsCollector;
use crate::error::{PathValidationError, UnixFsError};
use crate::path::ensure_safe_relative_path;
use nix::errno::Errno;
use nix::fcntl::{AtFlags, OFlag, openat};
use nix::sys::stat::{
    FchmodatFlags, FileStat, Mode, SFlag, UtimensatFlags, fchmod, fchmodat, fstatat, futimens,
    mkdirat, utimensat,
};
use nix::sys::time::TimeSpec;
use nix::unistd::{Gid, Uid, fchownat, linkat, symlinkat, unlinkat};
use std::collections::{HashMap, HashSet, VecDeque};
use std::ffi::{OsStr, OsString};
use std::fs::{self, File};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::FileExt;
use std::path::{Component, Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Instant;

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
    root_dir: Option<File>,
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
            root_dir: None,
            files: HashMap::new(),
            created_files: HashSet::new(),
            open_order: VecDeque::new(),
            dir_finalize_order: Vec::new(),
        }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    fn root_dir(&mut self) -> Result<&File, UnixFsError> {
        if self.root_dir.is_none() {
            self.root_dir = Some(File::open(&self.root)?);
        }
        Ok(self.root_dir.as_ref().expect("root dir initialized"))
    }

    fn open_root_dir(&self) -> Result<File, UnixFsError> {
        Ok(File::open(&self.root)?)
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

    pub fn prepare_regular_file(&mut self, target: &RestoreTarget) -> Result<PathBuf, UnixFsError> {
        if self.created_files.contains(&target.entry_id) {
            return Ok(target.relative_path.clone());
        }

        create_or_open_regular_file(&self.root, &target.relative_path, self.policy.overwrite)?;
        self.created_files.insert(target.entry_id);
        Ok(target.relative_path.clone())
    }

    pub fn prepare_regular_path(&self, target: &RestoreTarget) -> Result<PathBuf, UnixFsError> {
        ensure_safe_relative_path(&target.relative_path)?;
        Ok(target.relative_path.clone())
    }

    pub fn finalize_regular_data_file(
        &mut self,
        target: &RestoreTarget,
        file: &File,
    ) -> Result<(), UnixFsError> {
        apply_open_file_metadata(file, &target.metadata, self.policy.restore_owner)?;
        self.files.remove(&target.entry_id);
        self.open_order
            .retain(|entry_id| *entry_id != target.entry_id);
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
        create_directory(&self.root, &target.relative_path, self.policy.overwrite)?;
        self.dir_finalize_order.push(target.clone());
        Ok(())
    }

    fn ensure_file(&mut self, target: &RestoreTarget) -> Result<(), UnixFsError> {
        if self.files.contains_key(&target.entry_id) {
            return Ok(());
        }

        let file = if self.created_files.contains(&target.entry_id) {
            open_existing_regular_file(&self.root, &target.relative_path)?
        } else {
            self.created_files.insert(target.entry_id);
            create_or_open_regular_file(&self.root, &target.relative_path, self.policy.overwrite)?
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
        let (parent_dir, leaf, full_path) =
            parent_dir_handle(&self.root, &target.relative_path, true)?;
        if let Some(stat) = stat_entry(&parent_dir, leaf.as_os_str())? {
            match self.policy.overwrite {
                OverwritePolicy::Error => {
                    return Err(existing_entry_error(&full_path, &stat));
                }
                OverwritePolicy::Replace => {
                    remove_existing_entry(&parent_dir, leaf.as_os_str(), &full_path, &stat)?;
                }
            }
        }
        symlinkat(
            OsStr::from_bytes(link_target),
            &parent_dir,
            leaf.as_os_str(),
        )
        .map_err(UnixFsError::from)?;
        Ok(())
    }

    fn create_hardlink(
        &mut self,
        target: &RestoreTarget,
        master: &RestoreTarget,
    ) -> Result<(), UnixFsError> {
        ensure_safe_relative_path(&master.relative_path)?;
        let root_dir = self.open_root_dir()?;
        let (parent_dir, leaf, full_path) =
            parent_dir_handle(&self.root, &target.relative_path, true)?;
        if let Some(stat) = stat_entry(&parent_dir, leaf.as_os_str())? {
            match self.policy.overwrite {
                OverwritePolicy::Error => {
                    return Err(existing_entry_error(&full_path, &stat));
                }
                OverwritePolicy::Replace => {
                    remove_existing_entry(&parent_dir, leaf.as_os_str(), &full_path, &stat)?;
                }
            }
        }
        linkat(
            &root_dir,
            &master.relative_path,
            &parent_dir,
            leaf.as_os_str(),
            AtFlags::empty(),
        )
        .map_err(UnixFsError::from)?;
        Ok(())
    }

    fn finalize_entry(&mut self, target: &RestoreTarget) -> Result<(), UnixFsError> {
        let restore_owner = self.policy.restore_owner;
        let root_dir = self.root_dir()?;
        apply_entry_metadata(
            root_dir,
            &target.relative_path,
            &target.metadata,
            restore_owner,
        )?;
        self.files.remove(&target.entry_id);
        self.open_order
            .retain(|entry_id| *entry_id != target.entry_id);
        Ok(())
    }

    fn finalize_dirs(&mut self) -> Result<(), UnixFsError> {
        let restore_owner = self.policy.restore_owner;
        let targets = self.dir_finalize_order.clone();
        let root_dir = self.root_dir()?;
        for target in targets.iter().rev() {
            apply_entry_metadata(
                root_dir,
                &target.relative_path,
                &target.metadata,
                restore_owner,
            )?;
        }
        Ok(())
    }
}

struct FileCacheState {
    files: HashMap<u32, Arc<File>>,
    open_order: VecDeque<u32>,
    prepared_files: HashSet<u32>,
}

#[derive(Clone)]
pub(crate) struct PreparedRegularFile {
    pub(crate) full_path: PathBuf,
    pub(crate) parent_dir: Arc<File>,
    pub(crate) leaf: OsString,
}

pub struct ConcurrentFileWriter {
    files: HashMap<u32, PreparedRegularFile>,
    overwrite: OverwritePolicy,
    restore_owner: bool,
    max_open_files_by_shard: Vec<usize>,
    shards: Vec<Mutex<FileCacheState>>,
    diagnostics: Option<Arc<UnpackDiagnosticsCollector>>,
}

impl ConcurrentFileWriter {
    pub(crate) fn new(
        files: HashMap<u32, PreparedRegularFile>,
        overwrite: OverwritePolicy,
        restore_owner: bool,
        max_open_files: usize,
        shard_count: usize,
        diagnostics: Option<Arc<UnpackDiagnosticsCollector>>,
    ) -> Self {
        let max_open_files = max_open_files.max(1);
        let shard_count = shard_count.max(1).min(max_open_files);
        let base = max_open_files / shard_count;
        let extra = max_open_files % shard_count;
        let max_open_files_by_shard = (0..shard_count)
            .map(|idx| base + usize::from(idx < extra))
            .collect::<Vec<_>>();
        if let Some(collector) = diagnostics.as_ref() {
            collector.record_writer_config(
                max_open_files,
                shard_count,
                max_open_files_by_shard.iter().copied().max().unwrap_or(1),
            );
        }
        Self {
            files,
            overwrite,
            restore_owner,
            max_open_files_by_shard,
            shards: (0..shard_count)
                .map(|_| {
                    Mutex::new(FileCacheState {
                        files: HashMap::new(),
                        open_order: VecDeque::new(),
                        prepared_files: HashSet::new(),
                    })
                })
                .collect(),
            diagnostics,
        }
    }

    pub fn write_extent(
        &self,
        entry_id: u32,
        file_offset: u64,
        buf: &[u8],
    ) -> Result<(), UnixFsError> {
        let file = self.file_for(entry_id)?;
        let started = Instant::now();
        LocalRestorer::write_all_at(file.as_ref(), file_offset, buf)?;
        if let Some(collector) = self.diagnostics.as_ref() {
            collector.record_write(started.elapsed(), buf.len());
        }
        Ok(())
    }

    pub fn close_entry(&self, entry_id: u32) -> Result<(), UnixFsError> {
        let shard_idx = self.shard_index_for(entry_id);
        let mut state = self
            .state_for(shard_idx)
            .lock()
            .map_err(|_| UnixFsError::InvalidState("file-writer cache lock poisoned"))?;
        state.files.remove(&entry_id);
        state.open_order.retain(|open_id| *open_id != entry_id);
        Ok(())
    }

    pub fn take_entry(&self, entry_id: u32) -> Result<Option<Arc<File>>, UnixFsError> {
        let shard_idx = self.shard_index_for(entry_id);
        let mut state = self
            .state_for(shard_idx)
            .lock()
            .map_err(|_| UnixFsError::InvalidState("file-writer cache lock poisoned"))?;
        state.open_order.retain(|open_id| *open_id != entry_id);
        Ok(state.files.remove(&entry_id))
    }

    pub fn take_or_open_entry(&self, entry_id: u32) -> Result<Arc<File>, UnixFsError> {
        if let Some(file) = self.take_entry(entry_id)? {
            return Ok(file);
        }

        let prepared = self.file_spec(entry_id)?;

        let _state = self
            .state_for(self.shard_index_for(entry_id))
            .lock()
            .map_err(|_| UnixFsError::InvalidState("file-writer cache lock poisoned"))?;
        let file = open_existing_regular_file_at(
            prepared.parent_dir.as_ref(),
            prepared.leaf.as_os_str(),
            &prepared.full_path,
        )?;
        Ok(Arc::new(file))
    }

    pub fn close_all(&self) -> Result<(), UnixFsError> {
        for shard in &self.shards {
            let mut state = shard
                .lock()
                .map_err(|_| UnixFsError::InvalidState("file-writer cache lock poisoned"))?;
            state.files.clear();
            state.open_order.clear();
        }
        Ok(())
    }

    pub fn write_extent_once(
        &self,
        target: &RestoreTarget,
        file_offset: u64,
        buf: &[u8],
    ) -> Result<(), UnixFsError> {
        let prepared = self.file_spec(target.entry_id)?;
        let open_started = Instant::now();
        let file = create_or_open_regular_file_at_with_mode(
            prepared.parent_dir.as_ref(),
            prepared.leaf.as_os_str(),
            &prepared.full_path,
            self.overwrite,
            target.metadata.mode,
        )?;
        if let Some(collector) = self.diagnostics.as_ref() {
            collector.record_file_create(open_started.elapsed());
        }
        let write_started = Instant::now();
        LocalRestorer::write_all_at(&file, file_offset, buf)?;
        if let Some(collector) = self.diagnostics.as_ref() {
            collector.record_write(write_started.elapsed(), buf.len());
        }
        let finalize_started = Instant::now();
        apply_open_file_metadata(&file, &target.metadata, self.restore_owner)?;
        if let Some(collector) = self.diagnostics.as_ref() {
            collector.record_regular_finalize(finalize_started.elapsed());
        }
        Ok(())
    }

    fn file_for(&self, entry_id: u32) -> Result<Arc<File>, UnixFsError> {
        let shard_idx = self.shard_index_for(entry_id);
        let lock_started = Instant::now();
        let mut state = self
            .state_for(shard_idx)
            .lock()
            .map_err(|_| UnixFsError::InvalidState("file-writer cache lock poisoned"))?;
        if let Some(collector) = self.diagnostics.as_ref() {
            collector.record_writer_lock_wait(lock_started.elapsed());
        }

        if let Some(file) = state.files.get(&entry_id).cloned() {
            touch_open_order(&mut state.open_order, entry_id);
            if let Some(collector) = self.diagnostics.as_ref() {
                collector.record_file_cache_hit();
            }
            return Ok(file);
        }
        if let Some(collector) = self.diagnostics.as_ref() {
            collector.record_file_cache_miss();
        }

        let prepared = self.file_spec(entry_id)?;
        let file = if state.prepared_files.contains(&entry_id) {
            let open_started = Instant::now();
            let file = Arc::new(open_existing_regular_file_at(
                prepared.parent_dir.as_ref(),
                prepared.leaf.as_os_str(),
                &prepared.full_path,
            )?);
            if let Some(collector) = self.diagnostics.as_ref() {
                collector.record_file_reopen(open_started.elapsed());
            }
            file
        } else {
            let open_started = Instant::now();
            let file = Arc::new(create_or_open_regular_file_at(
                prepared.parent_dir.as_ref(),
                prepared.leaf.as_os_str(),
                &prepared.full_path,
                self.overwrite,
            )?);
            state.prepared_files.insert(entry_id);
            if let Some(collector) = self.diagnostics.as_ref() {
                collector.record_file_create(open_started.elapsed());
            }
            file
        };
        let evicted = evict_open_handles(&mut state, self.max_open_files_by_shard[shard_idx]);
        if let Some(collector) = self.diagnostics.as_ref() {
            collector.record_handle_evictions(evicted);
        }
        state.files.insert(entry_id, file.clone());
        touch_open_order(&mut state.open_order, entry_id);
        Ok(file)
    }

    fn file_spec(&self, entry_id: u32) -> Result<&PreparedRegularFile, UnixFsError> {
        self.files.get(&entry_id).ok_or(UnixFsError::InvalidState(
            "missing prepared regular-file path",
        ))
    }

    fn shard_index_for(&self, entry_id: u32) -> usize {
        entry_id as usize % self.shards.len()
    }

    fn state_for(&self, shard_idx: usize) -> &Mutex<FileCacheState> {
        &self.shards[shard_idx]
    }
}

pub(crate) fn prepare_regular_descriptor(
    root: &Path,
    rel: &Path,
    dir_cache: &mut HashMap<PathBuf, Arc<File>>,
    diagnostics: Option<&UnpackDiagnosticsCollector>,
) -> Result<PreparedRegularFile, UnixFsError> {
    let (parent_rel, leaf) = split_parent_and_leaf(rel)?;
    let parent_dir = directory_handle_for(root, dir_cache, &parent_rel, diagnostics)?;
    Ok(PreparedRegularFile {
        full_path: root.join(rel),
        parent_dir,
        leaf,
    })
}

fn relative_components(path: &Path) -> Result<Vec<OsString>, UnixFsError> {
    ensure_safe_relative_path(path)?;
    Ok(path
        .components()
        .filter_map(|component| match component {
            Component::Normal(segment) => Some(segment.to_os_string()),
            Component::CurDir
            | Component::ParentDir
            | Component::RootDir
            | Component::Prefix(_) => None,
        })
        .collect())
}

fn parent_dir_handle(
    root: &Path,
    rel: &Path,
    create_missing: bool,
) -> Result<(File, OsString, PathBuf), UnixFsError> {
    let segments = relative_components(rel)?;
    let (leaf, parents) = segments
        .split_last()
        .ok_or_else(|| UnixFsError::MissingParent(rel.to_path_buf()))?;

    let mut current = File::open(root)?;
    let mut current_path = PathBuf::new();
    for parent in parents {
        current_path.push(parent);
        match stat_entry(&current, parent.as_os_str())? {
            Some(stat) => {
                if is_symlink(&stat) {
                    return Err(
                        PathValidationError::SymlinkTraversal(root.join(&current_path)).into(),
                    );
                }
                if !is_directory(&stat) {
                    return Err(PathValidationError::NotADirectory(root.join(&current_path)).into());
                }
            }
            None if create_missing => {
                mkdirat(
                    &current,
                    parent.as_os_str(),
                    Mode::from_bits_truncate(0o755),
                )
                .map_err(UnixFsError::from)?;
            }
            None => return Err(UnixFsError::MissingParent(root.join(&current_path))),
        }
        current = open_directory_nofollow(&current, parent.as_os_str())?;
    }

    Ok((current, leaf.clone(), root.join(rel)))
}

fn split_parent_and_leaf(rel: &Path) -> Result<(PathBuf, OsString), UnixFsError> {
    ensure_safe_relative_path(rel)?;
    let leaf = rel
        .file_name()
        .ok_or_else(|| UnixFsError::MissingParent(rel.to_path_buf()))?;
    Ok((
        rel.parent().unwrap_or_else(|| Path::new("")).to_path_buf(),
        leaf.to_os_string(),
    ))
}

fn open_directory_handle(root: &Path, rel: &Path) -> Result<File, UnixFsError> {
    let mut current = File::open(root)?;
    let mut current_path = PathBuf::new();
    for segment in relative_components(rel)? {
        current_path.push(&segment);
        match stat_entry(&current, segment.as_os_str())? {
            Some(stat) => {
                if is_symlink(&stat) {
                    return Err(
                        PathValidationError::SymlinkTraversal(root.join(&current_path)).into(),
                    );
                }
                if !is_directory(&stat) {
                    return Err(PathValidationError::NotADirectory(root.join(&current_path)).into());
                }
            }
            None => return Err(UnixFsError::MissingParent(root.join(&current_path))),
        }
        current = open_directory_nofollow(&current, segment.as_os_str())?;
    }
    Ok(current)
}

fn directory_handle_for(
    root: &Path,
    cache: &mut HashMap<PathBuf, Arc<File>>,
    rel: &Path,
    diagnostics: Option<&UnpackDiagnosticsCollector>,
) -> Result<Arc<File>, UnixFsError> {
    let key = rel.to_path_buf();
    if let Some(dir) = cache.get(&key).cloned() {
        if let Some(collector) = diagnostics {
            collector.record_dir_cache_hit();
        }
        return Ok(dir);
    }
    if let Some(collector) = diagnostics {
        collector.record_dir_cache_miss();
    }
    let started = Instant::now();
    let dir = Arc::new(open_directory_handle(root, rel)?);
    if let Some(collector) = diagnostics {
        collector.record_directory_open(started.elapsed());
    }
    cache.insert(key, dir.clone());
    Ok(dir)
}

fn open_directory_nofollow(parent: &File, segment: &OsStr) -> Result<File, UnixFsError> {
    let fd = openat(
        parent,
        segment,
        OFlag::O_RDONLY | OFlag::O_CLOEXEC | OFlag::O_DIRECTORY | OFlag::O_NOFOLLOW,
        Mode::empty(),
    )
    .map_err(UnixFsError::from)?;
    Ok(File::from(fd))
}

fn stat_entry(parent: &File, leaf: &OsStr) -> Result<Option<FileStat>, UnixFsError> {
    match fstatat(parent, leaf, AtFlags::AT_SYMLINK_NOFOLLOW) {
        Ok(stat) => Ok(Some(stat)),
        Err(Errno::ENOENT) => Ok(None),
        Err(err) => Err(err.into()),
    }
}

fn create_directory(
    root: &Path,
    rel: &Path,
    overwrite: OverwritePolicy,
) -> Result<(), UnixFsError> {
    let (parent_dir, leaf, full_path) = parent_dir_handle(root, rel, true)?;
    match stat_entry(&parent_dir, leaf.as_os_str())? {
        Some(stat) if is_directory(&stat) => Ok(()),
        Some(stat) => match overwrite {
            OverwritePolicy::Error => Err(existing_entry_error(&full_path, &stat)),
            OverwritePolicy::Replace => {
                remove_existing_entry(&parent_dir, leaf.as_os_str(), &full_path, &stat)?;
                mkdirat(
                    &parent_dir,
                    leaf.as_os_str(),
                    Mode::from_bits_truncate(0o755),
                )
                .map_err(UnixFsError::from)
            }
        },
        None => mkdirat(
            &parent_dir,
            leaf.as_os_str(),
            Mode::from_bits_truncate(0o755),
        )
        .map_err(UnixFsError::from),
    }
}

fn create_or_open_regular_file(
    root: &Path,
    rel: &Path,
    overwrite: OverwritePolicy,
) -> Result<File, UnixFsError> {
    let (parent_dir, leaf, full_path) = parent_dir_handle(root, rel, true)?;
    create_or_open_regular_file_at(&parent_dir, leaf.as_os_str(), &full_path, overwrite)
}

fn create_or_open_regular_file_at(
    parent_dir: &File,
    leaf: &OsStr,
    full_path: &Path,
    overwrite: OverwritePolicy,
) -> Result<File, UnixFsError> {
    create_or_open_regular_file_at_with_mode(parent_dir, leaf, full_path, overwrite, 0o600)
}

fn create_or_open_regular_file_at_with_mode(
    parent_dir: &File,
    leaf: &OsStr,
    full_path: &Path,
    overwrite: OverwritePolicy,
    create_mode: u32,
) -> Result<File, UnixFsError> {
    let flags = match overwrite {
        OverwritePolicy::Error => {
            OFlag::O_WRONLY | OFlag::O_CLOEXEC | OFlag::O_CREAT | OFlag::O_EXCL | OFlag::O_NOFOLLOW
        }
        OverwritePolicy::Replace => {
            OFlag::O_WRONLY | OFlag::O_CLOEXEC | OFlag::O_CREAT | OFlag::O_TRUNC | OFlag::O_NOFOLLOW
        }
    };
    let create_mode = Mode::from_bits_truncate(create_mode as _);
    match openat(parent_dir, leaf, flags, create_mode) {
        Ok(fd) => Ok(File::from(fd)),
        Err(err) => match overwrite {
            OverwritePolicy::Error => Err(map_create_leaf_error(err, full_path)),
            OverwritePolicy::Replace => match err {
                Errno::ELOOP | Errno::EISDIR | Errno::ENOTDIR => {
                    if let Some(stat) = stat_entry(parent_dir, leaf)? {
                        if is_symlink(&stat) || is_directory(&stat) {
                            remove_existing_entry(parent_dir, leaf, full_path, &stat)?;
                            let fd = openat(parent_dir, leaf, flags, create_mode)
                                .map_err(|retry_err| map_leaf_open_error(retry_err, full_path))?;
                            Ok(File::from(fd))
                        } else {
                            Err(map_leaf_open_error(err, full_path))
                        }
                    } else {
                        Err(map_leaf_open_error(err, full_path))
                    }
                }
                other => Err(map_leaf_open_error(other, full_path)),
            },
        },
    }
}

fn open_existing_regular_file(root: &Path, rel: &Path) -> Result<File, UnixFsError> {
    let (parent_dir, leaf, full_path) = parent_dir_handle(root, rel, false)?;
    open_existing_regular_file_at(&parent_dir, leaf.as_os_str(), &full_path)
}

fn open_existing_regular_file_at(
    parent_dir: &File,
    leaf: &OsStr,
    full_path: &Path,
) -> Result<File, UnixFsError> {
    let fd = openat(
        parent_dir,
        leaf,
        OFlag::O_WRONLY | OFlag::O_CLOEXEC | OFlag::O_NOFOLLOW,
        Mode::empty(),
    )
    .map_err(|err| match err {
        Errno::ENOENT => UnixFsError::InvalidState("missing prepared regular-file path"),
        other => map_leaf_open_error(other, full_path),
    })?;
    Ok(File::from(fd))
}

fn remove_existing_entry(
    parent_dir: &File,
    leaf: &OsStr,
    full_path: &Path,
    stat: &FileStat,
) -> Result<(), UnixFsError> {
    if is_directory(stat) {
        fs::remove_dir_all(full_path)?;
    } else {
        unlinkat(parent_dir, leaf, nix::unistd::UnlinkatFlags::NoRemoveDir)
            .map_err(UnixFsError::from)?;
    }
    Ok(())
}

fn existing_entry_error(path: &Path, stat: &FileStat) -> UnixFsError {
    if is_symlink(stat) {
        PathValidationError::SymlinkTraversal(path.to_path_buf()).into()
    } else if is_directory(stat) {
        PathValidationError::NotADirectory(path.to_path_buf()).into()
    } else {
        std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            format!("path already exists: {}", path.display()),
        )
        .into()
    }
}

fn map_leaf_open_error(err: Errno, path: &Path) -> UnixFsError {
    match err {
        Errno::ELOOP => PathValidationError::SymlinkTraversal(path.to_path_buf()).into(),
        Errno::ENOTDIR | Errno::EISDIR => {
            PathValidationError::NotADirectory(path.to_path_buf()).into()
        }
        _ => err.into(),
    }
}

fn map_create_leaf_error(err: Errno, path: &Path) -> UnixFsError {
    match err {
        Errno::EEXIST => std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            format!("path already exists: {}", path.display()),
        )
        .into(),
        other => map_leaf_open_error(other, path),
    }
}

fn apply_entry_metadata(
    root_dir: &File,
    rel: &Path,
    metadata: &EntryMetadata,
    restore_owner: bool,
) -> Result<(), UnixFsError> {
    if rel.as_os_str().is_empty() {
        return Ok(());
    }

    fchmodat(
        root_dir,
        rel,
        Mode::from_bits_truncate(metadata.mode as _),
        FchmodatFlags::FollowSymlink,
    )
    .map_err(UnixFsError::from)?;

    if restore_owner && Uid::effective().is_root() {
        let uid = Uid::from_raw(metadata.uid);
        let gid = Gid::from_raw(metadata.gid);
        let _ = fchownat(root_dir, rel, Some(uid), Some(gid), AtFlags::empty());
    }

    utimensat(
        root_dir,
        rel,
        &TimeSpec::UTIME_OMIT,
        &TimeSpec::new(metadata.mtime_sec, metadata.mtime_nsec as _),
        UtimensatFlags::FollowSymlink,
    )
    .map_err(UnixFsError::from)?;
    Ok(())
}

fn apply_open_file_metadata(
    file: &File,
    metadata: &EntryMetadata,
    restore_owner: bool,
) -> Result<(), UnixFsError> {
    fchmod(file, Mode::from_bits_truncate(metadata.mode as _)).map_err(UnixFsError::from)?;

    if restore_owner && Uid::effective().is_root() {
        let uid = Uid::from_raw(metadata.uid);
        let gid = Gid::from_raw(metadata.gid);
        let _ = nix::unistd::fchown(file, Some(uid), Some(gid));
    }

    futimens(
        file,
        &TimeSpec::UTIME_OMIT,
        &TimeSpec::new(metadata.mtime_sec, metadata.mtime_nsec as _),
    )
    .map_err(UnixFsError::from)?;
    Ok(())
}

fn file_type(stat: &FileStat) -> SFlag {
    SFlag::from_bits_truncate(stat.st_mode) & SFlag::S_IFMT
}

fn is_directory(stat: &FileStat) -> bool {
    file_type(stat) == SFlag::S_IFDIR
}

fn is_symlink(stat: &FileStat) -> bool {
    file_type(stat) == SFlag::S_IFLNK
}

fn touch_open_order(open_order: &mut VecDeque<u32>, entry_id: u32) {
    open_order.retain(|open_id| *open_id != entry_id);
    open_order.push_back(entry_id);
}

fn evict_open_handles(state: &mut FileCacheState, max_open_files: usize) -> u64 {
    let mut evicted = 0_u64;
    while state.files.len() >= max_open_files.max(1) {
        if let Some(entry_id) = state.open_order.pop_front() {
            state.files.remove(&entry_id);
            evicted = evicted.saturating_add(1);
        } else {
            break;
        }
    }
    evicted
}

impl From<Errno> for UnixFsError {
    fn from(value: Errno) -> Self {
        std::io::Error::from_raw_os_error(value as i32).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::os::unix::fs::MetadataExt;
    use std::os::unix::fs::PermissionsExt;
    use std::os::unix::fs::symlink;
    use tempfile::TempDir;

    fn meta() -> EntryMetadata {
        EntryMetadata {
            mode: 0o755,
            uid: 0,
            gid: 0,
            mtime_sec: 1_234,
            mtime_nsec: 567,
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
        assert_eq!(
            fs::metadata(root.join("a/data.bin"))
                .expect("metadata")
                .permissions()
                .mode()
                & 0o777,
            0o755
        );
    }

    #[test]
    fn write_extent_once_restores_mode_and_mtime_without_owner_restore() {
        let temp = TempDir::new().expect("temp");
        let root = temp.path().to_path_buf();
        let relative_path = PathBuf::from("a/one-shot.bin");
        fs::create_dir_all(root.join("a")).expect("dir");
        let target = RestoreTarget {
            entry_id: 7,
            relative_path: relative_path.clone(),
            metadata: EntryMetadata {
                mode: 0o741,
                uid: 0,
                gid: 0,
                mtime_sec: 4_321,
                mtime_nsec: 765,
            },
        };
        let mut dir_cache = HashMap::new();
        let prepared =
            prepare_regular_descriptor(&root, &relative_path, &mut dir_cache, None).expect("prep");
        let writer = ConcurrentFileWriter::new(
            HashMap::from([(target.entry_id, prepared)]),
            OverwritePolicy::Replace,
            false,
            4,
            2,
            None,
        );

        writer
            .write_extent_once(&target, 0, b"payload")
            .expect("write one-shot");

        let metadata = fs::metadata(root.join(&relative_path)).expect("metadata");
        assert_eq!(metadata.permissions().mode() & 0o777, 0o741);
        assert_eq!(metadata.mtime(), 4_321);
        assert_eq!(metadata.mtime_nsec(), 765);
    }

    #[test]
    fn concurrent_writer_keeps_total_fd_budget_bounded() {
        let temp = TempDir::new().expect("temp");
        let root = temp.path().to_path_buf();
        fs::create_dir_all(root.join("a")).expect("dir");
        let mut dir_cache = HashMap::new();
        let prepared =
            prepare_regular_descriptor(&root, Path::new("a/data.bin"), &mut dir_cache, None)
                .expect("prep");
        let writer = ConcurrentFileWriter::new(
            HashMap::from([(1, prepared)]),
            OverwritePolicy::Replace,
            false,
            3,
            8,
            None,
        );

        assert_eq!(writer.shards.len(), 3);
        assert_eq!(writer.max_open_files_by_shard.iter().sum::<usize>(), 3);
        assert!(writer.max_open_files_by_shard.iter().all(|cap| *cap == 1));
    }
}
