mod cgroup;
use cgroup::Memory;
use cgroup::MemoryField;

use crate::cgroup::Cgroup;

fn main() {
    let pid: u64 = 19424;

    let memory_cgroup = Memory::create("virli").expect("Failed to open virli crgoup");

    memory_cgroup
        .write_value(&MemoryField::PROCS_LIST, pid)
        .expect("Failed to add pid");
    memory_cgroup
        .write_value(&MemoryField::MEMORY_LIMIT, 4194304)
        .expect("Failed to set limit");

    println!(
        "PID {} added to cgroup and limit set {}",
        pid,
        memory_cgroup
            .read_value(&MemoryField::MEMORY_LIMIT)
            .expect("Failed to read value")
    );
}
