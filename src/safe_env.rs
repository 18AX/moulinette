use fs_extra::dir::CopyOptions;
use log::{error, info, warn};
use nix::{
    errno::Errno,
    mount::{mount, umount2, MntFlags, MsFlags},
    unistd::pivot_root,
};
use std::{
    fs,
    path::{Path, PathBuf},
};
use tempdir::TempDir;

use crate::docker_image;

#[derive(Debug)]
pub enum SafeEnvError {
    IOError(std::io::Error),
    Mount(Errno),
    Umount(Errno),
    PivotRoot(Errno),
}

type Result<T> = std::result::Result<T, SafeEnvError>;

fn mount_workdir(src: &Path) -> Result<()> {
    let workdir_path: PathBuf = PathBuf::from("/").join("home").join("student");

    info!(target:"workdir", "{:?}", src);

    if let Err(e) = fs::create_dir_all(&workdir_path) {
        error!(target:"workdir", "create_dir failed");
        return Err(SafeEnvError::IOError(e));
    }

    if let Err(e) = mount(
        Some(src),
        &workdir_path,
        Option::Some("rw"),
        MsFlags::MS_BIND,
        Option::<&str>::None,
    ) {
        error!(target:"workdir", "mount failed");
        return Err(SafeEnvError::Mount(e));
    }

    info!(target:"workdir", "mounted");

    Ok(())
}

fn switch_root(rootfs: &Path) -> Result<&Path> {
    // Check if we can mout /
    if let Err(e) = mount(
        Option::<&str>::None,
        "/",
        Option::<&str>::None,
        MsFlags::MS_PRIVATE | MsFlags::MS_REC,
        Option::<&str>::None,
    ) {
        error!(target:"/", "mount failed");
        return Err(SafeEnvError::Mount(e));
    }

    info!(target:"/", "mounted");

    // Mount tmpdir for pivot root
    if let Err(e) = mount(
        Option::Some(rootfs),
        rootfs,
        Option::<&str>::None,
        MsFlags::MS_BIND,
        Option::<&str>::None,
    ) {
        error!(target:"tmpdir", "mount failed");
        return Err(SafeEnvError::Mount(e));
    }

    info!(target:"tmpdir", "mounted");

    // Create the directory where pivot root will mount the oldrootfs
    let oldroot = PathBuf::from(rootfs).join("oldrootfs");

    if let Err(e) = std::env::set_current_dir(rootfs) {
        return Err(SafeEnvError::IOError(e));
    }

    if let Err(e) = fs::create_dir_all(&oldroot) {
        return Err(SafeEnvError::IOError(e));
    }

    if let Err(e) = pivot_root(rootfs, &oldroot) {
        error!(target:"pivot_root", "failed");
        return Err(SafeEnvError::PivotRoot(e));
    }

    info!(target:"pivot_root", "done");

    if let Err(e) = std::env::set_current_dir("/") {
        return Err(SafeEnvError::IOError(e));
    }

    Ok(Path::new("/oldrootfs"))
}

fn mount_os_fs(oldroot: &Path) -> Result<()> {
    if let Err(e) = mount(
        Option::<&str>::None,
        "/proc",
        Option::Some("proc"),
        MsFlags::empty(),
        Option::<&str>::None,
    ) {
        error!(target:"proc", "mount failed");
        return Err(SafeEnvError::Mount(e));
    }

    info!(target:"proc", "mounted");

    if let Err(e) = mount(
        Option::Some(&oldroot.join("dev")),
        "/dev",
        Option::<&str>::None,
        MsFlags::MS_MOVE,
        Option::<&str>::None,
    ) {
        error!(target:"dev", "mount failed");
        return Err(SafeEnvError::Mount(e));
    }

    info!(target:"dev", "mounted");

    Ok(())
}

fn clean_oldrootfs(oldroot: &Path) -> Result<()> {
    if let Err(e) = umount2(oldroot, MntFlags::MNT_DETACH) {
        error!(target:"oldroot", "unmount failed");
        return Err(SafeEnvError::Umount(e));
    }

    if let Err(e) = fs::remove_dir(oldroot) {
        error!(target:"oldroot", "remove_dir failed");
        return Err(SafeEnvError::IOError(e));
    }

    info!(target:"oldroot", "cleaned");

    Ok(())
}

pub fn create_environment(workdir: Option<&String>, rootfs: Option<&String>) -> Result<()> {
    // Create a temp dir to be used as root file system
    let tmp_dir: TempDir = match TempDir::new("moulinette") {
        Ok(t) => t,
        Err(e) => return Err(SafeEnvError::IOError(e)),
    };

    info!(target:"safe_env", "env path {:?}", tmp_dir);

    let cpy_options: CopyOptions = CopyOptions {
        overwrite: true,
        skip_exist: false,
        buffer_size: 64000,
        copy_inside: true,
        content_only: true,
        depth: 0,
    };

    if let Some(rfs) = rootfs {
        // If we cannot pull the docker image we try to copy the rootfs from the host
        if let Err(e) = docker_image::download(rfs, tmp_dir.path()) {
            warn!(target:"rootfs", "{:?}", e);
            if let Err(_) = fs_extra::dir::copy(rfs, tmp_dir.path(), &cpy_options) {
                return Err(SafeEnvError::IOError(std::io::Error::last_os_error()));
            }
        }
    }

    // Update the path of the oldroot
    let oldroot = switch_root(tmp_dir.path())?;

    mount_os_fs(&oldroot)?;

    // Mount student files
    if let Some(w) = workdir {
        let p = format!("{}/{}", oldroot.as_os_str().to_str().unwrap(), w);
        mount_workdir(Path::new(&p))?;
    }

    clean_oldrootfs(&oldroot)?;

    Ok(())
}
