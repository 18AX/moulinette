use anyhow::{anyhow, Result};
use std::{ffi::CString, fs, path::PathBuf};

pub struct CgroupV2Builder {
    name: String,
    pids: Vec<u32>,
    max_mem: Option<u64>,
    max_pids: Option<u32>,
    cpus: Option<u32>,
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
            max_pids: Option::None,
            cpus: Option::None,
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

    pub fn set_pids_max(&mut self, max: u32) -> &mut Self {
        self.max_pids = Some(max);
        self
    }

    pub fn set_cpus_number(&mut self, n: u32) -> &mut Self {
        self.cpus = Some(n);

        self
    }

    pub fn create(&mut self) -> Result<CgroupV2> {
        let cgroup_path: PathBuf = PathBuf::from("/sys/fs/cgroup/");

        fs::create_dir_all(&cgroup_path)?;

        // Add cpu, cpuset, memory and pids controllers
        fs::write(cgroup_path.join("cgroup.subtree_control"), "+cpu")?;
        fs::write(cgroup_path.join("cgroup.subtree_control"), "+cpuset")?;
        fs::write(cgroup_path.join("cgroup.subtree_control"), "+memory")?;
        fs::write(cgroup_path.join("cgroup.subtree_control"), "+pids")?;

        let new_group_path: PathBuf = cgroup_path.join(&self.name);

        if new_group_path.exists() {
            if !new_group_path.is_dir() {
                return Err(anyhow!("Path exist but is not a directory"));
            }
        } else {
            fs::create_dir(&new_group_path)?;
        }

        // Let's add all the pids in the cgroup
        for pid in &self.pids {
            fs::write(
                new_group_path.join("cgroup.procs"),
                pid.to_string().as_str(),
            )?;
        }

        // Set the cpu limits
        if let Some(n_cpus) = self.cpus {
            fs::write(
                new_group_path.join("cpuset.cpus"),
                n_cpus.to_string().as_bytes(),
            )?;
        }

        // Set the memory limit
        if let Some(max_mem) = self.max_mem {
            fs::write(
                new_group_path.join("memory.max"),
                max_mem.to_string().as_str(),
            )?;
        }

        // Set the pids limit
        if let Some(max_pids) = self.max_pids {
            fs::write(
                new_group_path.join("pids.max"),
                max_pids.to_string().as_bytes(),
            )?;
        }

        Ok(CgroupV2 {
            name: String::from(&self.name),
            path: new_group_path,
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
