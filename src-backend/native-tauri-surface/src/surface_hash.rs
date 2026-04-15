use std::{
    fmt::{Display, Formatter, Result},
    ops::Deref,
};
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SurfaceHash(u64);

impl Deref for SurfaceHash {
    type Target = u64;

    fn deref(&self) -> &u64 {
        &self.0
    }
}

impl SurfaceHash {
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}
impl Display for SurfaceHash {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.0)
    }
}

impl From<u64> for SurfaceHash {
    fn from(value: u64) -> Self {
        SurfaceHash(value)
    }
}

impl From<String> for SurfaceHash {
    fn from(value: String) -> Self {
        Self(value.parse::<u64>().expect("parse failed"))
    }
}
