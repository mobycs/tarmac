use std::{
    collections::BTreeMap,
    fs, io,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use snafu::{ResultExt, Snafu};

static MANIFEST_FILENAME: &str = "tarmac-manifest.toml";

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Manifest {
    pub assets: BTreeMap<String, ManifestAsset>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ManifestAsset {
    pub uploaded_id: Option<u64>,
    pub uploaded_hash: Option<String>,
}

#[derive(Debug, Snafu)]
pub enum ManifestError {
    DeserializeToml {
        file_path: PathBuf,
        source: toml::de::Error,
    },

    SerializeToml {
        source: toml::ser::Error,
    },

    Io {
        file_path: PathBuf,
        source: io::Error,
    },
}

impl Manifest {
    /// Constructs a new manifest from an iterator of asset entries that should
    /// be present in the manifest.
    pub fn from_assets<I>(assets: I) -> Self
    where
        I: IntoIterator<Item = (String, ManifestAsset)>,
    {
        Self {
            assets: assets.into_iter().collect(),
        }
    }

    pub fn read_from_folder<P: AsRef<Path>>(folder_path: P) -> Result<Option<Self>, ManifestError> {
        let folder_path = folder_path.as_ref();
        let file_path = &folder_path.join(MANIFEST_FILENAME);

        let contents = match fs::read(file_path) {
            Ok(contents) => contents,
            Err(ref err) if err.kind() == io::ErrorKind::NotFound => {
                return Ok(None);
            }
            other => other.context(Io { file_path })?,
        };

        let config = toml::from_slice(&contents).context(DeserializeToml { file_path })?;

        Ok(Some(config))
    }

    pub fn write_to_folder<P: AsRef<Path>>(&self, folder_path: P) -> Result<(), ManifestError> {
        let folder_path = folder_path.as_ref();
        let file_path = &folder_path.join(MANIFEST_FILENAME);

        let serialized = toml::to_vec(self).context(SerializeToml)?;
        fs::write(file_path, serialized).context(Io { file_path })?;

        Ok(())
    }
}
