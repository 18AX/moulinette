mod cgroup;
use caps::errors::CapsError;
use caps::CapSet;
use fs_extra::dir::CopyOptions;
use std::env;
use std::fs;
use std::os::unix;
use std::path::PathBuf;
use std::process;
use std::process::Command;
use std::process::Stdio;
use tempdir::TempDir;

use crate::cgroup::Cgroup;

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

    for i in 1..args.len() {
        match args[i].as_str() {
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
        binary_name: binary_name,
        binary_args: binary_args,
        workdir: workdir,
        rootfs: rootfs,
    }
}

fn drop_capabilities() -> Result<(), CapsError> {
    caps::clear(None, CapSet::Bounding)?;
    caps::clear(None, CapSet::Inheritable)?;

    Ok(())
}

fn create_environment(
    binary: &str,
    workdir: Option<&String>,
    rootfs: Option<&String>,
) -> Result<String, &'static str> {
    // Create a temp dir to be used as root file system
    let tmp_dir: TempDir = match TempDir::new("moulinette") {
        Ok(dir) => dir,
        Err(_) => return Err("Failed to create temp directory"),
    };

    let binary_path: PathBuf = PathBuf::from(binary);

    let binary_filename = match binary_path.file_name() {
        Some(f) => f,
        None => return Err("Invalid binary name"),
    };

    let binary_cpy_path = tmp_dir.path().join(binary_filename);

    // Move everything to the directory
    if let Err(e) = fs::copy(binary, &binary_cpy_path) {
        println!("{}", e);
        return Err("Failed to copy binary to tmp fs");
    }

    if let Some(w) = workdir {
        if fs_extra::dir::copy(w, tmp_dir.path(), &CopyOptions::default()).is_err() {
            return Err("Failed to copy rootfs");
        }
    }

    if let Some(r) = rootfs {
        if fs_extra::dir::copy(r, tmp_dir.path(), &CopyOptions::default()).is_err() {
            return Err("Failed to copy workdir");
        }
    }

    // chroot the directory
    if unix::fs::chroot(tmp_dir.path()).is_err() {
        return Err("Failed to chroot tmp directory");
    }

    if std::env::set_current_dir("/").is_err() {
        return Err("Failed to change env dir");
    }

    Ok(String::from(binary_filename.to_str().unwrap()))
}

fn main() {
    let args: Arguments = parse_arguments();

    println!("[*] Creating environment directory");

    let binary_filename: String = create_environment(
        &args.binary_name,
        args.workdir.as_ref(),
        args.rootfs.as_ref(),
    )
    .expect("Failed to create environment");

    println!("[*] Dropping capabalities...");

    drop_capabilities().expect("Failed to drop capabilities");

    println!("[*] Running the binary...");

    let exec_path: PathBuf = PathBuf::new().join("/").join(binary_filename);

    let mut proc = Command::new(&exec_path)
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
}
