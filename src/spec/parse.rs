use crate::rel_path::RelPathBuf;
use crate::sha256::Sha256Digest;
use anyhow::Context;

pub(crate) struct ParsedTargetSpec {
    pub(crate) url: String,
    pub(crate) size: Option<u64>,
    pub(crate) sha256: Option<Sha256Digest>,
    pub(crate) path: RelPathBuf,
}

impl ParsedTargetSpec {
    pub(crate) fn parse(spec: &str) -> anyhow::Result<ParsedTargetSpec> {
        if spec.is_empty() {
            return Err(anyhow::anyhow!("empty spec"));
        }

        let mut url = None;
        let mut size = None;
        let mut sha256 = None;
        let mut path = None;

        for part in spec.split_whitespace() {
            if part.is_empty() {
                continue;
            }

            let Some((key, value)) = part.split_once('=') else {
                return Err(anyhow::anyhow!("Spec item must be key=value; got: {part}"));
            };

            if key.is_empty() {
                return Err(anyhow::anyhow!("Empty key"));
            }

            match key {
                "url" => {
                    if url.is_some() {
                        return Err(anyhow::anyhow!("Duplicate url"));
                    }
                    if value.is_empty() {
                        return Err(anyhow::anyhow!("Empty url"));
                    }
                    url = Some(value.to_owned());
                }
                "size" => {
                    if size.is_some() {
                        return Err(anyhow::anyhow!("Duplicate size"));
                    }
                    let value = value.parse().context("Could not parse size")?;
                    size = Some(value);
                }
                "sha256" => {
                    if sha256.is_some() {
                        return Err(anyhow::anyhow!("Duplicate sha256"));
                    }
                    if value.is_empty() {
                        return Err(anyhow::anyhow!("Empty sha256"));
                    }
                    let value = Sha256Digest::from_hex(value)?;
                    sha256 = Some(value);
                }
                "path" => {
                    if path.is_some() {
                        return Err(anyhow::anyhow!("Duplicate path"));
                    }
                    if value.is_empty() {
                        return Err(anyhow::anyhow!("Empty path"));
                    }
                    let value = RelPathBuf::new(value.to_owned())?;

                    path = Some(value);
                }
                key => {
                    return Err(anyhow::anyhow!("Unknown key: {key}"));
                }
            }
        }

        let url = url.context("Missing url")?;
        let path = path.context("Missing path")?;

        Ok(ParsedTargetSpec {
            url,
            size,
            sha256,
            path,
        })
    }
}
