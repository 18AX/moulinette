use anyhow::Result;
use caps::errors::CapsError;
use caps::CapSet;
use log::info;
use nix::sched::unshare;
use nix::sched::CloneFlags;
use nix::unistd::sethostname;
use rand::distributions::Alphanumeric;
use rand::Rng;
use seccomp::Context;
use seccomp_sys::SCMP_ACT_ALLOW;
use seccomp_sys::SCMP_ACT_ERRNO;
use std::env;
use std::process;
use std::process::Command;
use std::process::Stdio;
use syscall_numbers::x86_64::{SYS_nfsservctl, SYS_personality, SYS_pivot_root};

mod cgroup;
mod docker_image;
mod safe_env;
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
    env_logger::init();

    info!(target:"main", "parsing arguments");
    let args: Arguments = parse_arguments();

    unshare(CloneFlags::CLONE_NEWNS).expect("Failed to unshare");

    let hostname: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(10)
        .map(char::from)
        .collect();

    info!(target:"main", "generated random hostname {}", hostname);

    // Creating the cgroup
    let cgroup = cgroup::CgroupV2Builder::new(&hostname)
        .add_pid(process::id())
        .set_cpus_number(1)
        .set_mem_max(1073741824)
        .set_pids_max(100)
        .create()
        .expect("Failed to create cgroup");

    info!(target:"main", "process added to cgroup");

    safe_env::create_environment(args.workdir.as_ref(), args.rootfs.as_ref())
        .expect("Failed to create environment");

    info!(target:"main", "safe environment created");

    unshare(
        CloneFlags::CLONE_NEWCGROUP
            | CloneFlags::CLONE_NEWIPC
            | CloneFlags::CLONE_NEWNET
            | CloneFlags::CLONE_NEWPID
            | CloneFlags::CLONE_NEWUTS,
    )
    .expect("Failed to unshare");

    info!(target:"main", "unshare NEWCGROUP NEWIPC NEWNET NEWPID NEWUTS");

    sethostname(&hostname).expect("Failed to set hostname");

    drop_capabilities().expect("Failed to drop capabilities");

    info!(target:"main", "capabilities dropped");

    set_allowed_syscalls().expect("Failed to set up syscalls");

    info!(target:"main", "syscall filtered");

    let mut proc = Command::new(args.binary_name)
        .args(&args.binary_args)
        .stdout(Stdio::inherit())
        .stdin(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("Failed to execute process");

    let exit = proc.wait().expect("Failed to wait");

    let exit_code = exit.code().expect("Failed to retrieve exit code");

    info!(target:"exit_code", "{}", exit_code);

    std::process::exit(exit_code);
}
