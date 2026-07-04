use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use crate::{
    formats::{targz::TarGz, zip::Zip, zstd::Zstd},
    utils,
};

mod targz;
mod zip;
mod zstd;

pub struct Options {
    compression_level: Option<i64>,
}
impl Options {
    pub fn new(compression_level: Option<i64>) -> Self {
        Self { compression_level }
    }
}

pub trait ArchiveFormat {
    fn compress(&self, sources: &[PathBuf], archive: &Path, options: Options)
        -> Result<(), String>;
    fn extract(&self, archive: &Path, target: &Path) -> Result<(), String>;
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum Format {
    Zip(Zip),
    TarGz(TarGz),
    Zstd(Zstd),
}
impl ToString for Format {
    fn to_string(&self) -> String {
        match self {
            Self::Zip(_) => "zip",
            Self::TarGz(_) => "tar.gz",
            Self::Zstd(_) => "zstd",
        }
        .to_string()
    }
}
impl FromStr for Format {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "zip" => Ok(Self::Zip(Zip {})),
            "tar.gz" | "targz" => Ok(Self::TarGz(TarGz {})),
            "zstd" => Ok(Self::Zstd(Zstd {})),
            _ => Err(format!(
                "{s} is not a valid archive format ( zip tar.gz zstd )"
            )),
        }
    }
}
impl ArchiveFormat for Format {
    fn compress(
        &self,
        sources: &[PathBuf],
        archive: &Path,
        options: Options,
    ) -> Result<(), String> {
        match self {
            Format::Zip(zip) => zip.compress(sources, archive, options),
            Format::TarGz(targz) => targz.compress(sources, archive, options),
            Format::Zstd(zstd) => zstd.compress(sources, archive, options),
        }
    }

    fn extract(&self, archive: &Path, target: &Path) -> Result<(), String> {
        match self {
            Format::Zip(zip) => zip.extract(archive, target),
            Format::TarGz(targz) => targz.extract(archive, target),
            Format::Zstd(zstd) => zstd.extract(archive, target),
        }
    }
}
impl Format {
    pub fn is_format_ext(ext: &str) -> bool {
        Self::from_str(ext).is_ok()
    }
    pub fn get_extension(&self) -> String {
        self.to_string()
    }
    pub fn format_from_path(p: &Path) -> Option<Self> {
        utils::extract_file_extension(p)
            .map(|ext| Format::from_str(&ext).ok())
            .unwrap_or(None)
    }
    pub fn default() -> Self {
        Self::Zip(Zip)
    }
}
