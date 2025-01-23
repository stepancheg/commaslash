use crate::sha256::Sha256Digest;

#[derive(ordinal_map::Ordinal)]
pub(crate) enum Sha256Command {
    Sha256sum,
    Shasum,
}

impl Sha256Command {
    pub(crate) fn command(&self) -> &'static str {
        match self {
            Sha256Command::Sha256sum => "sha256sum",
            Sha256Command::Shasum => "shasum",
        }
    }

    pub(crate) fn test_command(&self) -> String {
        format!("{} --version", self.command())
    }

    pub(crate) fn check_command(&self, sha256_digest: Sha256Digest, file_path: &str) -> String {
        match self {
            Sha256Command::Sha256sum => {
                format!("echo \"{sha256_digest}  {file_path}\" | sha256sum --check -")
            }
            Sha256Command::Shasum => {
                format!("echo \"{sha256_digest}  {file_path}\" | shasum -a 256 --check -")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::sha256::Sha256Digest;
    use crate::sha256_command::Sha256Command;
    use crate::testutil::{assert_shell_err, assert_shell_ok};
    use ordinal_map::Ordinal;
    use std::fs;
    use tempfile::TempDir;

    fn commands_available() -> Vec<Sha256Command> {
        let mut commands_available = Vec::new();
        for command in Sha256Command::all_values() {
            if which::which(command.command()).is_ok() {
                commands_available.push(command);
            }
        }
        assert!(!commands_available.is_empty());
        commands_available
    }

    #[test]
    fn test_test_command() {
        for command in commands_available() {
            assert_shell_ok(command.test_command())
        }
    }

    #[test]
    fn test_check_command() {
        let tempfile = TempDir::new().unwrap();

        let path = tempfile.path().join("hello.txt");
        fs::write(&path, "hello").unwrap();
        let expected = Sha256Digest::from_hex(
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824",
        )
        .unwrap();
        let wrong = Sha256Digest::from_hex(
            "1717171717171717171717171717171717171717171717171717171717171717",
        )
        .unwrap();

        for command in commands_available() {
            assert_shell_ok(command.check_command(expected, path.to_str().unwrap()));
            assert_shell_err(command.check_command(wrong, path.to_str().unwrap()));
        }
    }
}
