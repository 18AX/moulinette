use cgroup::MemoryCgroup;

mod cgroup;

fn main() {
    let pid: u64 = 21335;

    let memory_cgroup = MemoryCgroup::open("virli").expect("Failed to open virli crgoup");

    memory_cgroup.add_pid(pid).expect("Failed to add pid");
    memory_cgroup
        .set_memory_limit(4194304)
        .expect("Failed to set limit");

    println!("PID {} added to cgroup and limit set", pid)
}
