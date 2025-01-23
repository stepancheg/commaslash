use crate::rel_path::RelPathBuf;
use crate::sha256::Sha256Digest;
use crate::spec::parse::ParsedTargetSpec;
use crate::target_platform::TargetPlatform;
use anyhow::Context;
use ordinal_map::map::OrdinalMap;
use crate::archive::ArchiveFormat;

pub(crate) struct ResolvedTargetSpec {
    pub(crate) url: String,
    pub(crate) size: u64,
    pub(crate) sha256: Sha256Digest,
    pub(crate) path: RelPathBuf,
    pub(crate) archive_format: ArchiveFormat,
}

impl ResolvedTargetSpec {
    pub(crate) fn parse_and_resolve(spec: &str) -> anyhow::Result<ResolvedTargetSpec> {
        let ParsedTargetSpec {
            url,
            size,
            sha256,
            path,
        } = ParsedTargetSpec::parse(spec)?;
        let archive_format = ArchiveFormat::from_file_path(&url)?;
        Ok(ResolvedTargetSpec {
            url,
            size: size.context("inferring size is not implemented")?,
            sha256: sha256.context("inferring sha256 is not implemented")?,
            path,
            archive_format,
        })
    }
}

pub(crate) struct ResolvedSpec {
    pub(crate) specs: OrdinalMap<TargetPlatform, ResolvedTargetSpec>,
}

impl ResolvedSpec {
    pub(crate) fn exe_name(&self) -> anyhow::Result<&str> {
        let target_spec = self
            .specs
            .values()
            .next()
            .context("must specify at least one spec")?;
        target_spec
            .path
            .file_name()
            .context("path must have a file name")
    }
}
