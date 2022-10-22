use cgroup::MemoryCgroup;

mod cgroup;

fn main() {
    let pid: u64 = 24846;

    let memory_cgroup = MemoryCgroup::open("virli").expect("Failed to open virli crgoup");

    memory_cgroup
        .write_value(cgroup::MemoryField::PROCS, pid)
        .expect("Failed to add pid");
    memory_cgroup
        .write_value(cgroup::MemoryField::MEMORY_LIMIT, 4194304)
        .expect("Failed to set limit");

    println!(
        "PID {} added to cgroup and limit set {}",
        pid,
        memory_cgroup
            .read_value(cgroup::MemoryField::MEMORY_LIMIT)
            .expect("Failed to read value")
    );
}
