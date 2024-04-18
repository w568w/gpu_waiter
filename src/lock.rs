use std::{
    fs::File, io, path::{Path, PathBuf}
};

use fs4::FileExt;

/// A heuristic way to decide a global runtime directory.
fn guess_global_runtime_dir() -> PathBuf {
    if cfg!(windows) {
        PathBuf::from("C:\\ProgramData")
    } else if cfg!(target_os = "android") {
        PathBuf::from("/data/local/tmp")
    } else if cfg!(unix) {
        PathBuf::from("/tmp")
    } else {
        panic!("You are running on an unsupported platform!")
    }
}

pub struct FileRWLock {
    file: std::fs::File,
}

pub struct RWLockReadGuard<'a> {
    _lock: &'a FileRWLock,
}

pub struct RWLockWriteGuard<'a> {
    _lock: &'a FileRWLock,
}

impl FileRWLock {
    pub fn new(path: impl AsRef<Path>) -> io::Result<Self> {
        let base_rt_dir = guess_global_runtime_dir();
        if !base_rt_dir.exists() {
            panic!(
                "The guessed runtime directory on your platform does not exist: {:?}",
                base_rt_dir
            );
        }
        let p = base_rt_dir.join(path);
        let f = File::create(p)?;
        Ok(Self { file: f })
    }

    pub fn read(&self) -> io::Result<RWLockReadGuard<'_>> {
        self.file.lock_shared()?;
        Ok(RWLockReadGuard { _lock: self })
    }

    pub fn write(&self) -> io::Result<RWLockWriteGuard<'_>> {
        self.file.lock_exclusive()?;
        Ok(RWLockWriteGuard { _lock: self })
    }
}

impl Drop for RWLockReadGuard<'_> {
    fn drop(&mut self) {
        self._lock.file.unlock().expect("Failed to unlock file");
    }
}

impl Drop for RWLockWriteGuard<'_> {
    fn drop(&mut self) {
        self._lock.file.unlock().expect("Failed to unlock file");
    }
}