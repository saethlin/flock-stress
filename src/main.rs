fn main() {
    loop {
        if let Ok(lock) = Lock::new(
            Path::new("lock"),
            /* wait */ false,
            /* create */ false,
            /* exclusive */ true,
        ) {
            let path = Path::new("test");
            assert!(!path.exists());
            File::create(path).unwrap();
            fs::remove_file(path).unwrap();
            drop(lock);
        }
    }
}

use std::fs;
use std::fs::{File, OpenOptions};
use std::io;
use std::mem;
use std::os::unix::prelude::*;
use std::path::Path;
use std::path::PathBuf;

#[derive(Debug)]
pub struct Lock {
    file: File,
}

impl Lock {
    pub fn new(p: &Path, wait: bool, create: bool, exclusive: bool) -> io::Result<Lock> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(create)
            .mode(libc::S_IRWXU as u32)
            .open(p)?;

        let lock_type = if exclusive {
            libc::F_WRLCK
        } else {
            libc::F_RDLCK
        };

        let mut flock: libc::flock = unsafe { mem::zeroed() };
        #[cfg(not(all(target_os = "hurd", target_arch = "x86")))]
        {
            flock.l_type = lock_type as libc::c_short;
            flock.l_whence = libc::SEEK_SET as libc::c_short;
        }
        #[cfg(all(target_os = "hurd", target_arch = "x86"))]
        {
            flock.l_type = lock_type as libc::c_int;
            flock.l_whence = libc::SEEK_SET as libc::c_int;
        }
        flock.l_start = 0;
        flock.l_len = 0;

        let cmd = if wait { libc::F_SETLKW } else { libc::F_SETLK };
        let ret = unsafe { libc::fcntl(file.as_raw_fd(), cmd, &flock) };
        if ret == -1 {
            Err(io::Error::last_os_error())
        } else {
            Ok(Lock { file })
        }
    }

    pub fn error_unsupported(err: &io::Error) -> bool {
        matches!(err.raw_os_error(), Some(libc::ENOTSUP) | Some(libc::ENOSYS))
    }
}

impl Drop for Lock {
    fn drop(&mut self) {
        let mut flock: libc::flock = unsafe { mem::zeroed() };
        #[cfg(not(all(target_os = "hurd", target_arch = "x86")))]
        {
            flock.l_type = libc::F_UNLCK as libc::c_short;
            flock.l_whence = libc::SEEK_SET as libc::c_short;
        }
        #[cfg(all(target_os = "hurd", target_arch = "x86"))]
        {
            flock.l_type = libc::F_UNLCK as libc::c_int;
            flock.l_whence = libc::SEEK_SET as libc::c_int;
        }
        flock.l_start = 0;
        flock.l_len = 0;

        unsafe {
            libc::fcntl(self.file.as_raw_fd(), libc::F_SETLK, &flock);
        }
    }
}
