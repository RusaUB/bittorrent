use super::{hashes::Hashes};

use crate::download::Downloaded;

use super::download;
use anyhow::Context;
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use std::path::Path;

/// Metainfo files (also known as .torrent files) 
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Torrent {
    /// URL to a "tracker", which is a central server that keeps
    /// track of peers participating in the sharing of a torrent
    pub announce : String,
    /// Maps to a dictionary information about torrent file
    pub info: Info,
}


#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Info {
    /// The name key maps to a UTF-8 encoded string which is the suggested name to save the file (or directory) as. 
    /// It is purely advisory.
    pub name: String,

    /// Piece length maps to the number of bytes in each piece the file is split into. 
    /// For the purposes of transfer, files are split into fixed-size pieces which are all 
    /// the same length except for possibly the last one which may be truncated. 
    /// Piece length is almost always a power of two, most commonly 2 18 = 256 K (BitTorrent prior to version 3.2 uses 2 20 = 1 M as default).
    #[serde(rename = "piece length")]
    pub plength: usize,

    /// Pieces maps to a string whose length is a multiple of 20. It is to be subdivided into strings of length 20, each of which is the SHA1 hash of the piece at the corresponding index.
    pub pieces: Hashes,

    /// There is also a key length or a key files, but not both or neither. If length is present then the download represents a single file, otherwise it represents a set of files which go in a directory structure.
    #[serde(flatten)]
    pub keys: Keys
}

/// There is a key `length` or a key `files`, but not both or neither.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Keys {
    /// If `length` is present then the download represents a single file.
    SingleFile {
        /// The length of the file in bytes.
        length: usize,
    },
    /// Otherwise it represents a set of files which go in a directory structure.
    ///
    /// For the purposes of the other keys in `Info`, the multi-file case is treated as only having
    /// a single file by concatenating the files in the order they appear in the files list.
    MultiFile { files: Vec<File> },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct File {
    /// The length of the file, in bytes.
    pub length: usize,

    /// Subdirectory names for this file, the last of which is the actual file name
    /// (a zero length list is an error case).
    pub path: Vec<String>,
}

impl Torrent {
    pub fn info_hash(&self) -> [u8; 20] {
        let info_encoded =
            serde_bencode::to_bytes(&self.info).expect("re-encode info section should be fine");
        let mut hasher = Sha1::new();
        hasher.update(&info_encoded);
        hasher
            .finalize()
            .try_into()
            .expect("GenericArray<_, 20> == [_; 20]")
    }

    pub async fn read(file: impl AsRef<Path>) -> anyhow::Result<Self> {
        let dot_torrent = tokio::fs::read(file).await.context("read torrent file")?;
        let t: Torrent = serde_bencode::from_bytes(&dot_torrent).context("parse torrent file")?;
        Ok(t)
    }

    pub fn print_tree(&self) {
        match &self.info.keys {
            Keys::SingleFile { .. } => {
                eprintln!("{}", self.info.name);
            }
            Keys::MultiFile { files } => {
                for file in files {
                    eprintln!("{}", file.path.join(std::path::MAIN_SEPARATOR_STR));
                }
            }
        }
    }

    pub fn length(&self) -> usize {
        match &self.info.keys {
            Keys::SingleFile { length } => *length,
            Keys::MultiFile { files } => files.iter().map(|file| file.length).sum(),
        }
    }

    pub async fn download_all(&self) -> anyhow::Result<Downloaded> {
        download::all(self).await
    }
}
