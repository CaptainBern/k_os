mod arch;
mod flags;
mod build;
mod run;

use std::path::{PathBuf, Path};
use xshell::{cmd, Shell};

fn main() -> anyhow::Result<()> {
    let sh = &Shell::new()?;
    sh.change_dir(project_root());

    let flags = flags::Xtask::from_env()?;
    match flags.subcommand {
        flags::XtaskCmd::Help(_) => {
            println!("{}", flags::Xtask::HELP);
            Ok(())
        }
        flags::XtaskCmd::Build(cmd) => cmd.run(sh),
        flags::XtaskCmd::Run(cmd) => cmd.run(sh)
    }
}

fn project_root() -> PathBuf {
    Path::new(&env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(1)
        .unwrap()
        .to_path_buf()
}