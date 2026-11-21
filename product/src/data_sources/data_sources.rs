use bincode_next::{Decode, config, decode_from_slice};
use common::{data_source::DataSource, data_source_kind::DataSourceKind};
use std::{collections::BTreeSet, error::Error};

#[derive(Debug, Decode)]
pub(crate) struct DataSources {
    sources: Vec<DataSource>,
}

impl DataSources {
    pub(crate) fn load() -> Result<Self, Box<dyn Error>> {
        let bytes = include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/data/data_sources.bin"
        ));

        let (data, consumed): (Self, usize) = decode_from_slice(bytes, config::standard())?;

        if consumed != bytes.len() {
            return Err("trailing data in data sources binary".into());
        }

        data.validate()?;
        Ok(data)
    }

    fn validate(&self) -> Result<(), Box<dyn Error>> {
        let mut known = BTreeSet::new();

        if self.sources.is_empty() {
            return Err("no data sources configured".into());
        }

        for source in &self.sources {
            if !known.insert(source.kind()) {
                return Err("duplicate data source".into());
            }
        }

        if !self.sources.iter().any(DataSource::enabled) {
            return Err("no enabled data sources".into());
        }

        Ok(())
    }

    pub(crate) fn enabled_sources(&self) -> impl Iterator<Item = DataSourceKind> + '_ {
        self.sources
            .iter()
            .filter(|source| source.enabled())
            .map(DataSource::kind)
    }
}
