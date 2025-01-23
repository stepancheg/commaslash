use crate::lockf::Lockf;
use crate::sha256_command::Sha256Command;

pub(crate) enum Os {
    Linux,
    Macos,
}

impl Os {
    /// Shell expression pointing to cache dir.
    pub(crate) fn cache_dir_expr(&self) -> &'static str {
        match self {
            Os::Linux => "${XDG_CACHE_HOME:-$HOME/.cache}",
            Os::Macos => "$HOME/Library/Caches",
        }
    }

    /// Path to the command to lock the file.
    pub(crate) fn flock(&self) -> Lockf {
        match self {
            Os::Linux => Lockf::Flock,
            Os::Macos => Lockf::Lockf,
        }
    }

    pub(crate) fn sha256_command(&self) -> Sha256Command {
        match self {
            Os::Linux => Sha256Command::Sha256sum,
            Os::Macos => Sha256Command::Shasum,
        }
    }
}
