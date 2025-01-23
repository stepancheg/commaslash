use std::fmt::{Display, Formatter};

#[derive(ordinal_map::Ordinal)]
pub(crate) enum ArchiveFormat {
    Zip,
}

impl Display for ArchiveFormat {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.command())
    }
}

impl ArchiveFormat {
    pub(crate) fn from_file_path(path: &str) -> anyhow::Result<Self> {
        if path.ends_with(".zip") {
            Ok(ArchiveFormat::Zip)
        } else {
            Err(anyhow::anyhow!(
                "Cannot determine archive format for `{path}`"
            ))
        }
    }

    pub(crate) fn command(&self) -> &'static str {
        match self {
            ArchiveFormat::Zip => "unzip",
        }
    }

    pub(crate) fn test_command(&self) -> &'static str {
        match self {
            ArchiveFormat::Zip => "unzip -v",
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::archive::ArchiveFormat;
    use crate::testutil::assert_shell_ok;
    use ordinal_map::Ordinal;

    #[test]
    fn test_test_command() {
        for archive in ArchiveFormat::all_values() {
            assert_shell_ok(archive.test_command());
        }
    }
}
