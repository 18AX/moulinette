use cgroup::MemoryCgroup;

mod cgroup;

fn main() {
    let pid: u64 = 16276;

    let memory_cgroup = MemoryCgroup::open("virli").expect("Failed to open virli crgoup");

    memory_cgroup.add_pid(pid).expect("Failed to add pid");

    println!("PID {} added to cgroup", pid)
}
