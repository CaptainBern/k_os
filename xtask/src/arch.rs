use std::{str::FromStr};

#[derive(Debug, Copy, Clone)]
pub enum Arch {
    X86_64,
}

impl Arch {
    #[inline]
    pub fn name(&self) -> &'static str {
        match self {
            Arch::X86_64 => "x86_64"
        }
    }
}

impl Default for Arch {
    fn default() -> Self {
        Arch::X86_64
    }
}

impl FromStr for Arch {
    type Err = anyhow::Error;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "x86_64" => Ok(Arch::X86_64),
            _ => Err(anyhow::format_err!("Unsupported architecture: {}", s))
        }
    }
}