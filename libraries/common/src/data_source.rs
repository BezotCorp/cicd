use crate::data_source_kind::DataSourceKind;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "source", derive(serde::Deserialize))]
#[cfg_attr(feature = "encode", derive(bincode_next::Encode))]
#[cfg_attr(feature = "decode", derive(bincode_next::Decode))]
pub struct DataSource {
    kind: DataSourceKind,
    enabled: bool,
}

impl DataSource {
    pub fn kind(&self) -> DataSourceKind {
        self.kind
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }
}
