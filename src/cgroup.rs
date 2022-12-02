use log::{error, info};
use std::{fs, path::PathBuf};

#[derive(Debug)]
pub enum CgroupError {
    InvalidName(&'static str),
    IOError(std::io::Error),
}

type Result<T> = std::result::Result<T, CgroupError>;

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

        if let Err(e) = fs::create_dir_all(&cgroup_path) {
            return Err(CgroupError::IOError(e));
        }

        // Add cpu, cpuset, memory and pids controllers
        if let Err(e) = fs::write(cgroup_path.join("cgroup.subtree_control"), "+cpu") {
            return Err(CgroupError::IOError(e));
        }

        if let Err(e) = fs::write(cgroup_path.join("cgroup.subtree_control"), "+cpuset") {
            return Err(CgroupError::IOError(e));
        }

        if let Err(e) = fs::write(cgroup_path.join("cgroup.subtree_control"), "+memory") {
            return Err(CgroupError::IOError(e));
        }

        if let Err(e) = fs::write(cgroup_path.join("cgroup.subtree_control"), "+pids") {
            return Err(CgroupError::IOError(e));
        }

        let new_group_path: PathBuf = cgroup_path.join(&self.name);

        if new_group_path.exists() {
            if !new_group_path.is_dir() {
                error!(target:"cgroup", "{:?} exists but is not a directory", new_group_path);
                return Err(CgroupError::InvalidName(
                    "Path exist but is not a directory",
                ));
            }
        } else {
            if let Err(e) = fs::create_dir(&new_group_path) {
                return Err(CgroupError::IOError(e));
            }
        }

        info!(target:"cgroup", "{:?} created", new_group_path);

        // Let's add all the pids in the cgroup
        for pid in &self.pids {
            if let Err(e) = fs::write(
                new_group_path.join("cgroup.procs"),
                pid.to_string().as_str(),
            ) {
                error!(target:"cgroup_procs", "{}", e);
                return Err(CgroupError::IOError(e));
            }
        }

        // Set the cpu limits
        if let Some(n_cpus) = self.cpus {
            if let Err(e) = fs::write(
                new_group_path.join("cpuset.cpus"),
                n_cpus.to_string().as_bytes(),
            ) {
                error!(target:"cgroup_cpus", "{}", e);
                return Err(CgroupError::IOError(e));
            }
        }

        // Set the memory limit
        if let Some(max_mem) = self.max_mem {
            if let Err(e) = fs::write(
                new_group_path.join("memory.max"),
                max_mem.to_string().as_str(),
            ) {
                error!(target:"cgroup_mem", "{}", e);
                return Err(CgroupError::IOError(e));
            }
        }

        // Set the pids limit
        if let Some(max_pids) = self.max_pids {
            if let Err(e) = fs::write(
                new_group_path.join("pids.max"),
                max_pids.to_string().as_bytes(),
            ) {
                error!(target:"cgroup_pids", "{}", e);
                return Err(CgroupError::IOError(e));
            }
        }

        Ok(CgroupV2 {
            name: String::from(&self.name),
            path: new_group_path,
        })
    }
}

impl CgroupV2 {
    pub fn destroy(&self) -> Result<()> {
        if let Err(e) = fs::remove_dir_all(&self.path) {
            error!(target:"cgroup_destroy", "{}", e);
            return Err(CgroupError::IOError(e));
        }

        Ok(())
    }
}

impl CgroupV2 {}
