use std::{
    fs,
    io::{Read, Write},
    path::Path,
};

pub struct MemoryCgroup {
    path: String,
}

pub enum MemoryField {
    PROCS,
    MEMORY_LIMIT,
}

impl MemoryField {
    pub fn as_str(&self) -> &'static str {
        match self {
            MemoryField::PROCS => "/cgroup.procs",
            MemoryField::MEMORY_LIMIT => "/memory.limit_in_bytes",
        }
    }
}

impl MemoryCgroup {
    pub fn open(name: &str) -> Result<MemoryCgroup, &'static str> {
        let mut path = String::from("/sys/fs/cgroup/memory/");
        path.push_str(name);

        if !Path::new(&path).is_dir() {
            return Err("Cgroup does not exists");
        }

        Ok(MemoryCgroup { path })
    }

    pub fn create(parent: &str, name: &str) -> Result<MemoryCgroup, &'static str> {
        let mut path = String::from("/sys/fs/cgroup/memory/");
        path.push_str(parent);
        path.push_str(name);

        if fs::create_dir(&path).is_err() {
            return Err("Failed to create new memory cgroup");
        }

        Ok(MemoryCgroup { path })
    }

    fn write_to_file(&self, filename: &str, value: &str) -> Result<(), &'static str> {
        let mut path = String::from(&self.path);
        path.push_str(filename);

        let mut file = match fs::File::create(path) {
            Ok(f) => f,
            Err(_) => return Err("failed to create file"),
        };

        if file.write(value.as_bytes()).is_err() {
            return Err("Failed to write pid");
        }

        Ok(())
    }

    fn read_from_file(&self, filename: &str) -> Result<String, &'static str> {
        let mut path = String::from(&self.path);
        path.push_str(filename);

        let mut file = match fs::File::open(path) {
            Ok(f) => f,
            Err(_) => return Err("failed to create file"),
        };

        let mut result: String = String::new();

        if file.read_to_string(&mut result).is_err() {
            return Err("Failed to read file contents");
        }

        Ok(result)
    }

    pub fn write_value<T: std::string::ToString>(
        &self,
        field: MemoryField,
        value: T,
    ) -> Result<(), &'static str> {
        self.write_to_file(field.as_str(), &value.to_string())
    }

    pub fn read_value(&self, field: MemoryField) -> Result<String, &'static str> {
        self.read_from_file(field.as_str())
    }
}
