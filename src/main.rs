mod cgroup;
use cgroup::Memory;
use cgroup::MemoryField;
use std::env;
use std::process;

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

fn main() {
    let args: Arguments = parse_arguments();

    println!("{:?}", args);
}
