use serde::de::{self, Deserialize, Deserializer, Visitor};
use serde::ser::{Serialize, Serializer};
use std::fmt;


/// The pieces field is one long string of bytes that consists of many, many 20-byte hashes.
/// SHA-1 hash of the corresponding fragment (piece) of the file.
/// e.g 
/// 
/// file = "Hello WorldHello WorldHello WorldHello World"
/// 
/// piece length = 10 bytes
/// 
/// SHA1("Hello wor") = 0a1b2c3d4e5f6789abcdef1234567890abcdef12
/// 
/// SHA1("ld! This ") = 1b2c3d4e5f6789abcdef1234567890abcdef123
/// 
/// ...
#[derive(Debug, Clone)]
pub struct Hashes(pub Vec<[u8; 20]>);
struct HashesVisitor;

impl<'de> Visitor<'de> for HashesVisitor {
    type Value = Hashes;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a byte string whose length is a multiple of 20")
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if v.len() % 20 != 0 {
            return Err(E::custom(format!("length is {}", v.len())));
        }
        // TODO: use array_chunks when stable
        Ok(Hashes(
            v.chunks_exact(20)
                .map(|slice_20| slice_20.try_into().expect("guaranteed to be length 20"))
                .collect(),
        ))
    }
}

impl<'de> Deserialize<'de> for Hashes {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_bytes(HashesVisitor)
    }
}

impl Serialize for Hashes {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let single_slice = self.0.concat();
        serializer.serialize_bytes(&single_slice)
    }
}
