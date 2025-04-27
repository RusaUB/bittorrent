use std::path::PathBuf;
use anyhow::{Context, Ok};
use bittorrent::{parse};
use clap::{Parser, Subcommand};
use serde_bencode;
use sha1::{Sha1, Digest};
use bittorrent::torrent::{Torrent};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug,Subcommand)]
enum Command {
    Decode {
        value: String,
    },
    Info {
        torrent: PathBuf
    }
}

fn main() -> anyhow::Result<()>{
    let args = Args::parse();

    match args.command {
        Command::Decode { value } => {
            let v: serde_json::Value = parse::decode_bencoded_value(&value);
            println!("{v}");
        }
        Command::Info { torrent } => {
            let dot_torrent = std::fs::read(torrent).context("open torrent file")?;
            let t: Torrent = serde_bencode::from_bytes(&dot_torrent).context("parse torrent file")?;
            eprintln!("{t:?}");

            // turn the structure back into a bencoded byte string, as it should look like in a .torrent file
            let info_encoded = serde_bencode::to_bytes(&t.info).context("re-encode info section")?;

            let mut hasher = Sha1::new();

            // feed the data, just serialised into the hasher 
            hasher.update(&info_encoded);
            let info_hash = hasher.finalize();

            println!("Info Hash: {}", hex::encode(&info_hash));
            println!("Piece Length: {}", t.info.plength);


            let pieces = &t.info.pieces;
            println!("Piece Hashes:");            
            for i in pieces.0.iter(){
                let piece_hash = hex::encode(i);
                println!("{}", piece_hash)
            }

        }
    }
    Ok(())
}