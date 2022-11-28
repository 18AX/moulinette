use anyhow::{anyhow, Result};
use fs_extra::dir::CopyOptions;
use libc::SYS_pivot_root;
use std::{ffi::CString, fs, os::unix, path::PathBuf, ptr::null};
use tempdir::TempDir;

pub fn create_environment(workdir: Option<&String>, rootfs: Option<&String>) -> Result<()> {
    // Create a temp dir to be used as root file system
    let tmp_dir: TempDir = TempDir::new("moulinette")?;

    // Copy the workdir and the rootfs
    if let Some(w) = workdir {
        fs_extra::dir::copy(w, tmp_dir.path(), &CopyOptions::default())?;
    }

    let cpy_options: CopyOptions = CopyOptions {
        overwrite: true,
        skip_exist: false,
        buffer_size: 64000,
        copy_inside: true,
        content_only: true,
        depth: 0,
    };

    if let Some(r) = rootfs {
        fs_extra::dir::copy(r, tmp_dir.path(), &cpy_options)?;
    }

    // Mount the tmpfs directory
    let src_string = CString::new(tmp_dir.path().as_os_str().to_str().unwrap())?;

    let res = unsafe {
        libc::mount(
            std::ptr::null(),
            CString::new("/")?.as_ptr(),
            std::ptr::null(),
            libc::MS_PRIVATE | libc::MS_REC,
            std::ptr::null(),
        )
    };

    if res != 0 {
        return Err(anyhow!("mount failed {}", res));
    }

    let res: i32 = unsafe {
        libc::mount(
            src_string.as_ptr(),
            src_string.as_ptr(),
            std::ptr::null(),
            libc::MS_BIND | libc::MS_REC,
            std::ptr::null(),
        )
    };

    if res != 0 {
        return Err(anyhow!("mount failed {}", res));
    }

    let path_buf = PathBuf::from(tmp_dir.path());
    let oldroot = path_buf.join("oldrootfs");

    std::env::set_current_dir(tmp_dir.path())?;
    fs::create_dir_all(&oldroot)?;

    // pivot root
    let pivot_new: CString = CString::new(path_buf.as_os_str().to_str().unwrap())?;
    let pivot_old: CString = CString::new(oldroot.as_os_str().to_str().unwrap())?;

    let pivot_root_res: i64 =
        unsafe { libc::syscall(SYS_pivot_root, pivot_new.as_ptr(), pivot_old.as_ptr()) };

    if pivot_root_res != 0 {
        return Err(anyhow!(
            "Failed to pivot root {}",
            std::io::Error::last_os_error()
        ));
    }

    // chroot the directory
    unix::fs::chroot("/")?;
    std::env::set_current_dir(".")?;

    unsafe {
        libc::umount(CString::new("oldrootfs")?.as_ptr());
    }

    Ok(())
}
