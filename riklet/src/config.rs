use serde::{Deserialize, Serialize};
use cri::container::RuncConfiguration;
use std::time::Duration;
use std::path::{PathBuf, Path};
use snafu::{ResultExt, Snafu};
use log::{info, debug};
use std::fs::File;
use std::io::Write;
use std::alloc::dealloc;
use oci::skopeo::SkopeoConfiguration;
use oci::image_manager::ImageManagerConfiguration;
use oci::umoci::UmociConfiguration;
use shared::utils::{create_file_with_parent_folders, create_directory_if_not_exists};

use crate::constants::DEFAULT_COMMAND_TIMEOUT;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Unable to load the configuration file. Error {}", source))]
    LoadError { source: std::io::Error },
    #[snafu(display("Unable to parse the configuration file. Error {}", source))]
    ParseError { source: toml::de::Error },
    #[snafu(display("Unable to encode the configuration in TOML format. Error {}", source))]
    TomlEncodeError { source: toml::ser::Error },
    #[snafu(display("Unable to create the configuration. Error {}",  source))]
    ConfigFileCreationError { source: std::io::Error },
    #[snafu(display("An error occured when trying to write the configuration. Error {}",  source))]
    ConfigFileWriteError { source: std::io::Error },
    #[snafu(display("An error occured when trying to create the {} directory. Error {}", path.display(), source))]
    CreateDirectoryError { source: std::io::Error, path: PathBuf },
}

#[derive(Deserialize, Debug, Serialize)]
pub struct Configuration {
    pub runner: RuncConfiguration,
    pub manager: ImageManagerConfiguration,
}

impl Configuration {

    /// Create the configuration file and store the default config into it
    fn create(path: &PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        info!("No configuration file found at {}. Creating a new configuration file with the default configuration.", path.display());
        let configuration = Configuration::default();

        let toml = toml::to_string(&configuration)
            .map_err(|source| Error::TomlEncodeError { source })?;

        let mut file = create_file_with_parent_folders(path)
            .map_err(|source| Error::ConfigFileCreationError { source })?;

        file.write_all(&toml.into_bytes())
            .map_err(|source| Error::ConfigFileWriteError { source })?;

        Ok(configuration)
    }

    /// Read the configuration file from the path provided.
    fn read(path: &PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = std::fs::read(path)
            .map_err(|source| Error::LoadError { source })?;

        Ok(
            toml::from_slice(&contents)
            .map_err(|source| Error::ParseError { source })?
        )
    }

    /// Load the configuration file
    /// If not exists, create it and return the default configuration
    pub fn load(path: &PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let configuration = if !path.exists() {
            Configuration::create(path)
        } else {
            Configuration::read(path)
        }?;

        debug!("Loaded configuration from file {}.", path.display());

        Ok(configuration)
    }

    /// Create all directories and files used by Riklet to work properly
    pub fn bootstrap(&self) -> Result<(), Error> {

        let bundles_dir = self.manager.oci_manager.bundles_directory.clone();
        let images_dir = self.manager.image_puller.images_directory.clone();

        create_directory_if_not_exists(&bundles_dir)
            .map_err(|source| Error::CreateDirectoryError {
                source,
                path: bundles_dir.unwrap()
            })?;

        create_directory_if_not_exists(&images_dir)
            .map_err(|source| Error::CreateDirectoryError {
                source,
                path: images_dir.unwrap()
            })?;

        Ok(())
    }
}

/// Default implementation
impl Default for Configuration {

    fn default() -> Self {
        Self {
            runner: RuncConfiguration {
                debug: false,
                rootless: false,
                root: None,
                command: None,
                timeout: Some(Duration::from_millis(DEFAULT_COMMAND_TIMEOUT)),
            },
            manager: ImageManagerConfiguration {
                image_puller: SkopeoConfiguration {
                    images_directory: Some(PathBuf::from("/var/lib/riklet/images")),
                    timeout: Some(Duration::from_millis(DEFAULT_COMMAND_TIMEOUT)),
                    debug: false,
                    insecure_policy: false,
                    ..Default::default()
                },
                oci_manager: UmociConfiguration {
                    timeout: Some(Duration::from_millis(DEFAULT_COMMAND_TIMEOUT)),
                    bundles_directory: Some(PathBuf::from("/var/lib/riklet/bundles")),
                    debug: false,
                    ..Default::default()
                }
            }
        }
    }
}

