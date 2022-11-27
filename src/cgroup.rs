use anyhow::{anyhow, Result};
use std::{fs, path::PathBuf};

pub struct CgroupV2Builder {
    name: String,
    pids: Vec<u32>,
    max_mem: Option<u64>,
}

pub struct CgroupV2 {
    name: String,
    path: PathBuf,
}

impl CgroupV2Builder {
    pub fn new(name: &str) -> Self {
        CgroupV2Builder {
            name: String::from(name),
            pids: Vec::new(),
            max_mem: Option::None,
        }
    }

    pub fn add_pid(&mut self, pid: u32) -> &mut Self {
        self.pids.push(pid);

        self
    }

    pub fn set_mem_max(&mut self, max: u64) -> &mut Self {
        self.max_mem = Some(max);

        self
    }

    pub fn create(&mut self) -> Result<CgroupV2> {
        let path_builder: PathBuf = PathBuf::from("/sys/fs/cgroup").join(&self.name);

        if path_builder.exists() {
            if !path_builder.is_dir() {
                return Err(anyhow!("Path exist but is not a directory"));
            }
        } else {
            fs::create_dir(&path_builder)?;
        }

        // Let's add all the pids in the cgroup
        for pid in &self.pids {
            fs::write(path_builder.join("cgroup.procs"), pid.to_string().as_str())?;
        }

        // Add memory controller at the root dir
        fs::write("/sys/fs/cgroup/cgroup.subtree_control", "+memory")?;

        // Set the memory limit
        if let Some(max_mem) = self.max_mem {
            fs::write(
                path_builder.join("memory.max"),
                max_mem.to_string().as_str(),
            )?;
        }

        Ok(CgroupV2 {
            name: String::from(&self.name),
            path: path_builder,
        })
    }
}

impl CgroupV2 {
    pub fn destroy(&self) -> Result<()> {
        fs::remove_dir_all(&self.path)?;

        Ok(())
    }
}

impl CgroupV2 {}
