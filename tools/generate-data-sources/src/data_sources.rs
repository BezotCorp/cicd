use bincode_next::{Encode, config, encode_to_vec};
use common::data_source::DataSource;
use serde::Deserialize;
use std::{collections::BTreeSet, error::Error, fs, path::Path};

#[derive(Deserialize, Encode)]
struct DataSources {
    sources: Vec<DataSource>,
}

impl DataSources {
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
}

pub(crate) fn generate() -> Result<(), Box<dyn Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .ok_or("invalid generator directory")?;

    let source = fs::read_to_string(root.join("product/data/data_sources.ron"))?;

    let data: DataSources = ron::from_str(&source)?;
    data.validate()?;

    let encoded = encode_to_vec(&data, config::standard())?;

    fs::write(root.join("product/data/data_sources.bin"), encoded)?;

    Ok(())
}
