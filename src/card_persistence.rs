//! Host persistence only: never truncate the previous card before replacement.
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
};

struct TemporaryCard { path: PathBuf, remove: bool }

impl Drop for TemporaryCard {
    fn drop(&mut self) {
        if self.remove {
            let _ = fs::remove_file(&self.path);
        }
    }
}

pub fn save(path: &Path, bytes: &[u8]) -> io::Result<()> {
    replace_with(path, |file| file.write_all(bytes))
}

fn replace_with(path: &Path, write: impl FnOnce(&mut File) -> io::Result<()>) -> io::Result<()> {
    let permissions = match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if !metadata.is_file() || metadata.permissions().readonly() {
                return Err(io::Error::new(io::ErrorKind::PermissionDenied,
                    "card destination must be a writable regular file, not a link or directory"));
            }
            Some(metadata.permissions())
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => None,
        Err(error) => return Err(error),
    };
    let filename = path.file_name().ok_or_else(||
        io::Error::new(io::ErrorKind::InvalidInput, "card destination has no filename"))?;
    let mut temporary = None;
    for attempt in 0..100 {
        let mut name = filename.to_os_string();
        name.push(format!(".hp67-tmp-{}-{attempt}", std::process::id()));
        let candidate = path.with_file_name(name);
        match OpenOptions::new().write(true).create_new(true).open(&candidate) {
            Ok(file) => {
                temporary = Some((TemporaryCard { path: candidate, remove: true }, file));
                break;
            }
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }
    let (mut temporary, mut file) = temporary.ok_or_else(||
        io::Error::new(io::ErrorKind::AlreadyExists, "no free card temporary filename"))?;
    // Always close before cleanup/rename, including errors, for Windows handles.
    let result = write(&mut file).and_then(|()| {
        if let Some(permissions) = permissions {
            file.set_permissions(permissions)?;
        }
        file.sync_all()
    });
    drop(file);
    result?;
    // Never remove the destination to work around a failed replacement.
    // A hard-linked destination gets a new file; its other links retain old bytes.
    fs::rename(&temporary.path, path)?;
    temporary.remove = false;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn core_audit_card_save_preserves_old_bytes_on_write_and_replace_failure() {
        let root = std::env::temp_dir().join(format!("hp67-card-save-{}", std::process::id()));
        fs::create_dir(&root).unwrap();
        let path = root.join("card.hp67card");
        save(&path, b"old card").unwrap();
        let result = replace_with(&path, |file| {
            file.write_all(b"partial new card")?;
            Err(io::Error::new(io::ErrorKind::Other, "injected write failure"))
        });
        assert!(result.is_err());
        assert_eq!(fs::read(&path).unwrap(), b"old card");
        assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
        save(&path, b"complete new card").unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"complete new card");
        let directory = root.join("directory");
        fs::create_dir(&directory).unwrap();
        assert!(save(&directory, b"must not replace directory").is_err());
        assert!(directory.is_dir());
        let failed_destination = root.join("raced-destination");
        assert!(replace_with(&failed_destination, |file| {
            file.write_all(b"complete")?;
            fs::create_dir(&failed_destination)
        }).is_err());
        assert!(failed_destination.is_dir());
        assert_eq!(fs::read(&path).unwrap(), b"complete new card");
        assert_eq!(fs::read_dir(&root).unwrap().count(), 3);
        fs::remove_dir_all(root).unwrap();
    }
}
