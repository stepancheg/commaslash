mod archive;
mod gen;
mod genpy;
mod lockf;
mod os;
mod rel_path;
mod sh;
mod sha256;
pub(crate) mod spec;
mod target_platform;
mod sha256_command;
mod testutil;
mod github;
mod shx;

use std::fs;
use std::fs::Permissions;
use std::os::unix::fs::PermissionsExt;
use crate::gen::gen;
use crate::target_platform::TargetPlatform;
use anyhow::Context;
use clap::Parser;
use ordinal_map::map::OrdinalMap;
use spec::resolve::{ResolvedSpec, ResolvedTargetSpec};

/// Generate a script which downloads an archive and runs a binary from it.
#[derive(clap::Parser)]
struct Args {
    /// Spec for macos-aarch64.
    #[clap(long, value_name = "spec")]
    macos_aarch64: Option<String>,
    /// Spec for macos-x86_64
    #[clap(long, value_name = "spec")]
    macos_x86_64: Option<String>,
    /// Spec for linux-x86_64.
    #[clap(long, value_name = "spec")]
    linux_x86_64: Option<String>,
    /// Where to write the resulting script; `-` for stdout.
    #[clap(long, value_name = "output")]
    output: String,
}

pub fn commaslash_main() -> anyhow::Result<()> {
    let args = Args::parse();

    if args.macos_aarch64.is_none() && args.macos_x86_64.is_none() && args.linux_x86_64.is_none() {
        return Err(anyhow::anyhow!(
            "Must specify at least one spec, e.g. --macos-aarch64=..."
        ));
    }

    let specs = OrdinalMap::from_iter([
        (TargetPlatform::LinuxX86_64, args.linux_x86_64),
        (TargetPlatform::MacosAarch64, args.macos_aarch64),
        (TargetPlatform::MacosX86_64, args.macos_x86_64),
    ]);

    let mut resolved_spec = ResolvedSpec {
        specs: OrdinalMap::new(),
    };

    for (target_platform, spec) in specs {
        let Some(spec) = spec else {
            continue;
        };
        let spec = ResolvedTargetSpec::parse_and_resolve(&spec)
            .with_context(|| format!("Failed to parse target spec for {target_platform}"))?;
        resolved_spec.specs.insert(target_platform, spec);
    }

    let script = gen(&resolved_spec)?;

    if args.output == "-" {
        print!("{}", script);
    } else {
        fs::write(&args.output, script)?;
        fs::set_permissions(&args.output, Permissions::from_mode(0o755))?;
    }

    Ok(())
}
