use std::{
    io::{Read, Write},
    path::{self, Path, PathBuf},
};

pub trait CgroupField {
    fn as_str(&self) -> &'static str;
}

pub trait Cgroup {
    fn open(name: &str) -> Result<Box<Self>, &'static str>;
    fn create(name: &str) -> Result<Box<Self>, &'static str>;
    fn get_path(&self) -> &Path;

    fn write_value<T: std::string::ToString>(
        &self,
        field: &dyn CgroupField,
        value: T,
    ) -> Result<(), ()> {
        let mut path: PathBuf = PathBuf::from(self.get_path());
        path.push(field.as_str());

        self.write_to_file(&path, &value.to_string())
    }

    fn read_value(&self, field: &dyn CgroupField) -> Result<String, ()> {
        let mut path: PathBuf = PathBuf::from(self.get_path());
        path.push(field.as_str());

        self.read_from_file(&path)
    }

    fn write_to_file(&self, p: &PathBuf, value: &str) -> Result<(), ()> {
        let mut file = match std::fs::File::create(p) {
            Ok(f) => f,
            Err(_) => return Err(()),
        };

        if file.write(value.as_bytes()).is_err() {
            return Err(());
        }

        Ok(())
    }

    fn read_from_file(&self, filename: &PathBuf) -> Result<String, ()> {
        let mut file = match std::fs::File::open(filename) {
            Ok(f) => f,
            Err(_) => return Err(()),
        };

        let mut result: String = String::new();

        if file.read_to_string(&mut result).is_err() {
            return Err(());
        }

        Ok(result)
    }
}

pub enum MemoryField {
    PROCS_LIST,
    MEMORY_LIMIT,
}

pub struct Memory {
    path: PathBuf,
}

impl CgroupField for MemoryField {
    fn as_str(&self) -> &'static str {
        match self {
            MemoryField::PROCS_LIST => "/cgroup.procs",
            MemoryField::MEMORY_LIMIT => "/memory.limit_in_bytes",
        }
    }
}

const MEMORY_CGROUP_PATH: &'static str = "/sys/fs/cgroup/memory/";

impl Cgroup for Memory {
    fn open(name: &str) -> Result<Box<Memory>, &'static str> {
        let mut path: PathBuf = PathBuf::from(MEMORY_CGROUP_PATH);
        path.push(name);

        if !path.is_dir() {
            return Err("Path is not a directory");
        }

        Ok(Box::new(Memory { path: path }))
    }

    fn create(name: &str) -> Result<Box<Memory>, &'static str> {
        let mut path: PathBuf = PathBuf::from(MEMORY_CGROUP_PATH);
        path.push(name);

        if std::fs::create_dir(&path).is_err() {
            return Err("Failed to create new memory cgroup");
        }

        Ok(Box::new(Memory { path: path }))
    }

    fn get_path(&self) -> &Path {
        &self.path
    }
}
