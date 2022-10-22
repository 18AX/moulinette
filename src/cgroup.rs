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

    fn write_to_file(&self, filename: &str, value: &str) -> Result<(), &'static str> {
        let mut path = String::from(&self.name);
        path.push_str(filename);

        println!("path {} value: {}", path, value);

        let mut file = match fs::File::create(path) {
            Ok(f) => f,
            Err(_) => return Err("failed to create file"),
        };

        if file.write(value.as_bytes()).is_err() {
            return Err("Failed to write pid");
        }

        Ok(())
    }

    pub fn add_pid(&self, pid: u64) -> Result<(), &'static str> {
        self.write_to_file("/cgroup.procs", &pid.to_string())
    }

    pub fn set_memory_limit(&self, limit: u64) -> Result<(), &'static str> {
        self.write_to_file("/memory.limit_in_bytes", &limit.to_string())
    }
}
