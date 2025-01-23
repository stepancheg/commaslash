use crate::os::Os;

#[derive(ordinal_map::Ordinal, derive_more::Display)]
pub(crate) enum TargetPlatform {
    #[display("linux-x86_64")]
    LinuxX86_64,
    #[display("macos-x86_64")]
    MacosX86_64,
    #[display("macos-aarch64")]
    MacosAarch64,
}

impl TargetPlatform {
    pub(crate) fn uname_sm(&self) -> &'static str {
        match self {
            TargetPlatform::LinuxX86_64 => "Linux x86_64",
            TargetPlatform::MacosX86_64 => "Darwin x86_64",
            TargetPlatform::MacosAarch64 => "Darwin arm64",
        }
    }

    pub(crate) fn os(&self) -> Os {
        match self {
            TargetPlatform::LinuxX86_64 => Os::Linux,
            TargetPlatform::MacosX86_64 => Os::Macos,
            TargetPlatform::MacosAarch64 => Os::Macos,
        }
    }
}
