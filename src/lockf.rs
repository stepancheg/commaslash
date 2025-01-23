use std::fmt::{Display, Formatter};

#[derive(ordinal_map::Ordinal)]
pub(crate) enum Lockf {
    /// macOS.
    Lockf,
    /// Linux.
    Flock,
}

impl Display for Lockf {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.command())
    }
}

impl Lockf {
    pub(crate) fn command(&self) -> &'static str {
        match self {
            Lockf::Flock => "flock",
            Lockf::Lockf => "lockf",
        }
    }

    pub(crate) fn test_command(&self) -> Option<&'static str> {
        match self {
            Lockf::Lockf => {
                // No easy way to check command exists.
                None
            }
            Lockf::Flock => Some("flock --version"),
        }
    }

    pub(crate) fn lock_fs(&self, timeout_seconds: u32, fd: u32) -> String {
        match self {
            Lockf::Lockf => format!("lockf -t {timeout_seconds} {fd}"),
            Lockf::Flock => format!("flock -w {timeout_seconds} {fd}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::lockf::Lockf;
    use crate::testutil::assert_shell_ok;
    use ordinal_map::Ordinal;
    use crate::github::is_github_actions;

    #[test]
    fn test_display() {
        assert_eq!("flock", Lockf::Flock.to_string());
    }

    fn available_commands() -> Vec<Lockf> {
        let mut available_commands = Vec::new();
        for command in Lockf::all_values() {
            if which::which(command.command()).is_ok() {
                available_commands.push(command);
            }
        }
        if !cfg!(target_os = "macos") || !is_github_actions() {
            // There's no either `flock` or `lockf` on github macos.
            assert!(!available_commands.is_empty());
        }
        available_commands
    }

    #[test]
    fn test_test_command() {
        for command in available_commands() {
            if let Some(test_command) = command.test_command() {
                assert_shell_ok(test_command);
            }
        }
    }
}
