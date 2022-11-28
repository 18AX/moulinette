mod cgroup;
use anyhow::Result;
use caps::errors::CapsError;
use caps::CapSet;
use fs_extra::dir::CopyOptions;
use seccomp::Context;
use seccomp_sys::SCMP_ACT_ALLOW;
use seccomp_sys::SCMP_ACT_ERRNO;
use std::env;
use std::os::unix;
use std::process;
use std::process::Command;
use std::process::Stdio;
use syscall_numbers::x86_64::{SYS_nfsservctl, SYS_personality, SYS_pivot_root};
use tempdir::TempDir;

mod seccomp;

#[derive(Debug)]
struct Arguments {
    binary_name: String,
    binary_args: Vec<String>,
    workdir: Option<String>,
    rootfs: Option<String>,
}

fn print_help() {
    println!("MyMoulette, the students' nightmare, now highly secured");
    println!("Usage: ./mymoulette [-v student_workdir] <-I docker-img|rootfs-path> moulette_prog [moulette_arg [...]]");
    println!("\trootfs-path is the path to the directory containing the new rootfs (exclusive with -Ioption)");
    println!("\tdocker-img is an image available on hub.docker.com (exclusive with rootfs-path)");
    println!("\tmoulette_prog will be the first program to be launched, must already be in the environment");
    println!("\tstudent_workdir is the directory containing the code to grade");
}

/// Returns the parsed arguments from the command line
fn parse_arguments() -> Arguments {
    let args: Vec<String> = env::args().collect();

    let mut binary_name: String = String::new();
    let mut binary_args: Vec<String> = Vec::new();
    let mut workdir: Option<String> = Option::None;
    let mut rootfs: Option<String> = Option::None;

    let mut expecting_workdir: bool = false;
    let mut expecting_rootfs: bool = false;
    let mut is_binary_name_set: bool = false;

    for item in args.iter().skip(1) {
        match item.as_str() {
            "-v" => expecting_workdir = true,
            "-I" => expecting_rootfs = true,
            s => {
                if expecting_workdir {
                    expecting_workdir = false;
                    workdir = Some(String::from(s));
                } else if expecting_rootfs {
                    expecting_rootfs = false;
                    rootfs = Some(String::from(s));
                } else if is_binary_name_set {
                    binary_args.push(String::from(s));
                } else {
                    binary_name = String::from(s);
                    is_binary_name_set = true;
                }
            }
        }
    }

    if !is_binary_name_set {
        print_help();
        process::exit(1);
    }

    Arguments {
        binary_name,
        binary_args,
        workdir,
        rootfs,
    }
}

fn drop_capabilities() -> Result<(), CapsError> {
    caps::clear(None, CapSet::Bounding)?;
    caps::clear(None, CapSet::Inheritable)?;

    Ok(())
}

fn create_environment(workdir: Option<&String>, rootfs: Option<&String>) -> Result<()> {
    // Create a temp dir to be used as root file system
    let tmp_dir: TempDir = TempDir::new("moulinette")?;

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

    // chroot the directory
    unix::fs::chroot(tmp_dir.path())?;

    std::env::set_current_dir("/")?;

    Ok(())
}

fn set_allowed_syscalls() -> Result<()> {
    // We allow everything
    let seccomp: Context = Context::new(SCMP_ACT_ALLOW)?;

    // Let's create a blocklist

    seccomp.add_simple_array(
        vec![
            SYS_nfsservctl as i32,
            SYS_personality as i32,
            SYS_pivot_root as i32,
        ],
        SCMP_ACT_ERRNO(1),
    )?;

    seccomp.load()?;

    Ok(())
}
fn main() {
    let args: Arguments = parse_arguments();

    println!("[*] Adding pid to cgroup");

    let cgroup = cgroup::CgroupV2Builder::new("moulinette")
        .add_pid(process::id())
        .set_cpus_number(1)
        .set_mem_max(1048576)
        .set_pids_max(100)
        .create()
        .expect("Failed to create cgroup");

    println!("[*] Creating environment directory");

    create_environment(args.workdir.as_ref(), args.rootfs.as_ref())
        .expect("Failed to create environment");

    println!("[*] Dropping capabalities...");

    drop_capabilities().expect("Failed to drop capabilities");

    println!("[*] Setting allowed syscalls");
    set_allowed_syscalls().expect("Failed to set up syscalls");

    println!("[*] Running the binary...");

    let mut proc = Command::new(args.binary_name)
        .args(&args.binary_args)
        .stdout(Stdio::inherit())
        .stdin(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("Failed to execute process");

    let exit = proc.wait().expect("Failed to wait");

    println!(
        "process exited {}",
        exit.code().expect("Failed to retrieve exit code")
    );

    cgroup.destroy().expect("Failed to destroy cgroup");
}
