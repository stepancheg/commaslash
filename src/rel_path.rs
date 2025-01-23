use std::mem;
use std::ops::Deref;

#[derive(derive_more::Display)]
#[repr(transparent)]
pub(crate) struct RelPath(str);

#[derive(derive_more::Display)]
pub(crate) struct RelPathBuf(String);

impl RelPath {
    fn unchecked_new(path: &str) -> &RelPath {
        // SAFETY: repr(transparent).
        unsafe { mem::transmute::<&str, &RelPath>(path) }
    }

    pub(crate) fn new(path: &str) -> anyhow::Result<&RelPath> {
        if path.is_empty() {
            return Ok(RelPath::unchecked_new(""));
        }
        if path.starts_with('/') {
            return Err(anyhow::anyhow!("path must not start with /: {path}"));
        }
        if path.ends_with('/') {
            return Err(anyhow::anyhow!("path must not end with /: {path}"));
        }
        for component in path.split('/') {
            if component.is_empty() {
                return Err(anyhow::anyhow!("empty component in path: {path}"));
            }
            if component == "." {
                return Err(anyhow::anyhow!("component . in path: {path}"));
            }
            if component == ".." {
                return Err(anyhow::anyhow!("component .. in path: {path}"));
            }
        }
        Ok(RelPath::unchecked_new(path))
    }

    pub(crate) fn components(&self) -> impl DoubleEndedIterator<Item = &str> {
        self.0.split('/')
    }

    pub(crate) fn file_name(&self) -> Option<&str> {
        self.components().last()
    }
}

impl RelPathBuf {
    pub(crate) fn new(path: String) -> anyhow::Result<RelPathBuf> {
        RelPath::new(&path)?;
        Ok(RelPathBuf(path))
    }
}

impl Deref for RelPathBuf {
    type Target = RelPath;

    fn deref(&self) -> &Self::Target {
        RelPath::unchecked_new(&self.0)
    }
}
