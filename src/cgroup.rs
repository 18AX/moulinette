use std::{fs, io::Write, path::Path};

pub struct MemoryCgroup {
    name: String,
}

impl MemoryCgroup {
    pub fn open(name: &str) -> Result<MemoryCgroup, &'static str> {
        let mut path = String::from("/sys/fs/cgroup/memory/");
        path.push_str(name);

        if !Path::new(&path).is_dir() {
            return Err("Cgroup does not exists");
        }

        Ok(MemoryCgroup { name: path })
    }

    pub fn create(parent: &str, name: &str) -> Result<MemoryCgroup, &'static str> {
        let mut path = String::from("/sys/fs/cgroup/memory/");
        path.push_str(parent);
        path.push_str(name);

        if fs::create_dir(&path).is_err() {
            return Err("Failed to create new memory cgroup");
        }

        Ok(MemoryCgroup { name: path })
    }

    pub fn add_pid(&self, pid: u64) -> Result<(), &'static str> {
        let mut path = String::from(&self.name);
        path.push_str("/cgroup.procs");

        let mut file = match fs::File::create(path) {
            Ok(f) => f,
            Err(_) => return Err("failed to create file"),
        };

        if file.write(pid.to_string().as_bytes()).is_err() {
            return Err("Failed to write pid");
        }

        Ok(())
    }
}
