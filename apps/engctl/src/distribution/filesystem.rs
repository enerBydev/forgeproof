use super::{Error, fail};
use eng_catalog::read_bounded;
use std::{
    fs::{self, File, OpenOptions, TryLockError},
    io::{ErrorKind, Write},
    os::unix::fs::{MetadataExt, OpenOptionsExt},
    path::{Component, Path, PathBuf},
};

pub fn regular(path: &Path) -> Result<bool, Error> {
    match fs::symlink_metadata(path) {
        Ok(m) if m.is_file() && m.nlink() == 1 => Ok(true),
        Ok(_) => Err(fail("unsafe_install_path")),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(false),
        Err(_) => Err(fail("installation_io_error")),
    }
}
pub fn directory(path: &Path) -> Result<(), Error> {
    let mut prefix = PathBuf::new();
    for part in path.components() {
        if part == Component::ParentDir {
            return Err(fail("unsafe_install_path"));
        }
        prefix.push(part);
        let meta = fs::symlink_metadata(&prefix).map_err(|_| fail("installation_io_error"))?;
        if !meta.is_dir() || meta.file_type().is_symlink() {
            return Err(fail("unsafe_install_path"));
        }
    }
    if path.as_os_str().is_empty() {
        return Err(fail("unsafe_install_path"));
    }
    Ok(())
}
pub fn read_optional(path: &Path, limit: u64, code: &str) -> Result<Option<String>, Error> {
    if !regular(path)? {
        return Ok(None);
    }
    let bytes = read_bounded(path, limit).map_err(|_| fail(code))?;
    String::from_utf8(bytes).map(Some).map_err(|_| fail(code))
}
pub fn sync_dir(path: &Path) -> Result<(), Error> {
    File::open(path)
        .and_then(|f| f.sync_all())
        .map_err(|_| fail("installation_io_error"))
}
pub struct LockedProject {
    pub project: PathBuf,
    pub control: PathBuf,
    _lock: File,
}
impl LockedProject {
    pub fn open(project: &Path, create: bool) -> Result<Option<Self>, Error> {
        directory(project)?;
        let control = project.join(".forgeproof");
        match fs::symlink_metadata(&control) {
            Ok(m) if m.is_dir() && !m.file_type().is_symlink() => (),
            Ok(_) => return Err(fail("unsafe_install_path")),
            Err(e) if e.kind() == ErrorKind::NotFound && !create => return Ok(None),
            Err(e) if e.kind() == ErrorKind::NotFound => match fs::create_dir(&control) {
                Ok(()) => sync_dir(project)?,
                Err(e) if e.kind() == ErrorKind::AlreadyExists => directory(&control)?,
                Err(_) => return Err(fail("installation_io_error")),
            },
            Err(_) => return Err(fail("installation_io_error")),
        }
        let path = control.join("install.lock");
        regular(&path)?;
        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .mode(0o600)
            .open(&path)
            .map_err(|_| fail("installation_io_error"))?;
        match lock.try_lock() {
            Ok(()) => (),
            Err(TryLockError::WouldBlock) => return Err(fail("installation_busy")),
            Err(_) => return Err(fail("installation_io_error")),
        }
        let result = Self {
            project: project.into(),
            control,
            _lock: lock,
        };
        for item in fs::read_dir(&result.control).map_err(|_| fail("installation_io_error"))? {
            let item = item.map_err(|_| fail("installation_io_error"))?;
            regular(&item.path())?;
            if !["install.lock", "state.json", "pending.json", "write.tmp"]
                .iter()
                .any(|name| item.file_name() == *name)
            {
                return Err(fail("invalid_installation_state"));
            }
        }
        Ok(Some(result))
    }
    pub fn replace(&self, destination: &Path, text: Option<&str>) -> Result<(), Error> {
        regular(destination)?;
        let temp = self.control.join("write.tmp");
        if regular(&temp)? {
            fs::remove_file(&temp).map_err(|_| fail("installation_io_error"))?;
        }
        match text {
            Some(text) => {
                let mut file = OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .mode(0o600)
                    .open(&temp)
                    .map_err(|_| fail("installation_io_error"))?;
                file.write_all(text.as_bytes())
                    .and_then(|_| file.sync_all())
                    .map_err(|_| fail("installation_io_error"))?;
                fs::rename(&temp, destination).map_err(|_| fail("installation_io_error"))?;
            }
            None => {
                if regular(destination)? {
                    fs::remove_file(destination).map_err(|_| fail("installation_io_error"))?;
                }
            }
        }
        sync_dir(
            destination
                .parent()
                .ok_or_else(|| fail("unsafe_install_path"))?,
        )?;
        // A native-file rename crosses directories; persist removal of the staging name too.
        sync_dir(&self.control)
    }
}
