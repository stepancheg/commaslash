use anyhow::Context;
use std::fmt::{Display, Formatter};

#[derive(Copy, Clone)]
pub(crate) struct Sha256Digest {
    bytes: [u8; 32],
}

impl Display for Sha256Digest {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let encoded = hex::encode(self.bytes);
        write!(f, "{encoded}")
    }
}

impl Sha256Digest {
    pub(crate) fn from_hex(hex: &str) -> anyhow::Result<Sha256Digest> {
        let bytes = hex::decode(hex).context("Failed to parse sha-256 from hex string")?;
        Ok(Sha256Digest {
            bytes: bytes.try_into().ok().context("Incorrect sha-256 length")?,
        })
    }
}
