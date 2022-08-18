use std::path::PathBuf;

use xshell::{Shell, cmd};

use crate::{flags, project_root};

impl flags::Build {
    #[inline]
    pub fn src_dir(&self) -> PathBuf {
        project_root()
            .join(format!("src/arch/{}", self.arch.unwrap_or_default().name()))
    }

    #[inline]
    pub fn target_spec(&self) -> PathBuf {
        self.src_dir()
            .join(format!("kernel-{}.json", self.arch.unwrap_or_default().name()))
    }

    #[inline]
    pub fn target_binary(&self) -> PathBuf {
        project_root()
            .join("target")
            .join(format!("kernel-{}", self.arch.unwrap_or_default().name()))
            .join(if self.release { "release" } else { "debug" })
            .join("kernel")
    }

    pub fn run(&self, sh: &Shell) -> anyhow::Result<()> {
        let flags = [
            "-Zbuild-std=core,alloc",
            "-Zbuild-std-features=compiler-builtins-mem"
        ];
        let release = if self.release {
            Some("--release")
        } else {
            None
        };

        let target_spec = self.target_spec();
        let _d = sh.push_dir(self.src_dir());

        cmd!(sh, "cargo build {flags...} {release...} --target {target_spec}").run()?;

        Ok(())
    }
}
