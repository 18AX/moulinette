use std::{error::Error, fmt::Display};

use seccomp_sys::{scmp_filter_ctx, seccomp_init, seccomp_load, seccomp_release, seccomp_rule_add};

pub struct Context {
    ctx: *mut scmp_filter_ctx,
}

#[derive(Debug)]
pub enum SeccompError {
    InitFailed,
    LoadFailed,
    RuleAddFailed,
}

impl Display for SeccompError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "seccomp {:?}", self)
    }
}

impl Error for SeccompError {}

type Result<T> = std::result::Result<T, SeccompError>;

impl Context {
    pub fn new(action: u32) -> Result<Context> {
        let ctx: *mut scmp_filter_ctx = unsafe { seccomp_init(action) };

        if ctx.is_null() {
            return Err(SeccompError::InitFailed);
        }

        Ok(Context { ctx })
    }

    pub fn add_simple_rule(&self, syscall: i32, action: u32) -> Result<()> {
        let res: i32 = unsafe { seccomp_rule_add(self.ctx, action, syscall, 0) };

        if res != 0 {
            return Err(SeccompError::RuleAddFailed);
        }

        Ok(())
    }

    pub fn add_simple_array(&self, syscalls: Vec<i32>, action: u32) -> Result<()> {
        for syscall in syscalls {
            self.add_simple_rule(syscall, action)?;
        }

        Ok(())
    }

    pub fn load(&self) -> Result<()> {
        let res: i32 = unsafe { seccomp_load(self.ctx) };

        if res != 0 {
            return Err(SeccompError::LoadFailed);
        }

        Ok(())
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            seccomp_release(self.ctx);
        }
    }
}
