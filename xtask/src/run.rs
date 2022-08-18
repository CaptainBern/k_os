use xshell::{Shell, cmd};

use crate::{flags, project_root};

impl flags::Run {
    pub fn run(&self, sh: &Shell) -> anyhow::Result<()> {
        let build = flags::Build {
            arch: self.arch,
            release: self.release,
        };

        // Build target first.
        build.run(sh)?;

        let graftpoints = &[
            format!("boot/kernel={}", build.target_binary().to_str().unwrap()),
            format!("boot/grub/grub.cfg={}", build.src_dir().join("boot/grub/grub.cfg").to_str().unwrap()),
        ];

        let build_dir = project_root().join("build").join(self.arch.unwrap_or_default().name());
        let iso_path = build_dir.join("image.iso");

        sh.create_dir(&build_dir)?;

        // TODO: These are hard-coded for x86 at the moment.
        cmd!(sh, "grub-mkrescue -o {iso_path} -graft-points {graftpoints...}").run()?;
        cmd!(sh, "qemu-system-x86_64 -enable-kvm -cpu host -m 4G -display sdl -serial stdio -drive file={iso_path},media=cdrom").run()?;
        
        Ok(())
    }
}