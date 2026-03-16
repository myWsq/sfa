use crate::error::PathValidationError;
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::{Component, Path, PathBuf};

pub fn ensure_safe_relative_path(path: &Path) -> Result<(), PathValidationError> {
    if path.as_os_str().is_empty() {
        return Ok(());
    }
    if path.is_absolute() {
        return Err(PathValidationError::AbsolutePath);
    }

    for component in path.components() {
        match component {
            Component::Normal(seg) => validate_segment(seg)?,
            Component::CurDir => return Err(PathValidationError::DotSegment),
            Component::ParentDir => return Err(PathValidationError::ParentTraversal),
            Component::RootDir | Component::Prefix(_) => {
                return Err(PathValidationError::AbsolutePath);
            }
        }
    }

    Ok(())
}

pub fn safe_join(root: &Path, relative: &Path) -> Result<PathBuf, PathValidationError> {
    ensure_safe_relative_path(relative)?;
    let joined = root.join(relative);
    if !joined.starts_with(root) {
        return Err(PathValidationError::OutsideRoot(joined));
    }
    Ok(joined)
}

fn validate_segment(seg: &OsStr) -> Result<(), PathValidationError> {
    let bytes = seg.as_bytes();
    if bytes.is_empty() {
        return Err(PathValidationError::EmptySegment);
    }
    if bytes.contains(&0) {
        return Err(PathValidationError::NulByte);
    }
    if bytes == b"." {
        return Err(PathValidationError::DotSegment);
    }
    if bytes == b".." {
        return Err(PathValidationError::ParentTraversal);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_safe_relative_path() {
        assert!(ensure_safe_relative_path(Path::new("a/b/c")).is_ok());
        assert!(ensure_safe_relative_path(Path::new("")).is_ok());
        assert!(ensure_safe_relative_path(Path::new("../x")).is_err());
        assert!(ensure_safe_relative_path(Path::new("/x")).is_err());
        assert!(ensure_safe_relative_path(Path::new("./x")).is_err());
    }
}
