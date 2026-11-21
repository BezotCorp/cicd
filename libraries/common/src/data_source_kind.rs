#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "source", derive(serde::Deserialize))]
#[cfg_attr(feature = "encode", derive(bincode_next::Encode))]
#[cfg_attr(feature = "decode", derive(bincode_next::Decode))]
pub enum DataSourceKind {
    Git,
    Ci,
}

impl DataSourceKind {
    pub const ALL: [Self; 2] = [Self::Git, Self::Ci];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Git => "git",
            Self::Ci => "ci",
        }
    }
}
