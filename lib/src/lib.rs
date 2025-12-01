use core::fmt;
use serde::{Deserialize, Serialize};
use std::error::Error;
use uint::construct_uint;
construct_uint! {
    #[derive(Serialize, Deserialize)]
    pub struct U256(4);
}
// initial reward in bitcoin - multiply by 10^8 to get satoshis
pub const INITIAL_REWARD: u64 = 50;
// halving interval in blocks
pub const HALVING_INTERVAL: u64 = 210;
// ideal block time in seconds
pub const IDEAL_BLOCK_TIME: u64 = 10;
// minimum target
pub const MINIMUM_TARGET: U256 = U256([
    0xFFFF_FFFF_FFFF_FFFF,
    0xFFFF_FFFF_FFFF_FFFF,
    0xFFFF_FFFF_FFFF_FFFF,
    0x0000_FFFF_FFFF_FFFF,
]);
// difficulty update intervals in blocks
pub const DIFFICULTY_UPDATE_INTERVALS: u64 = 50;


pub mod crypto;
pub mod error;
pub mod sha256;
pub mod types;
pub mod utils;
pub mod signkey_serde {
    use ecdsa::SigningKey;
    use k256::Secp256k1;
    use serde::Deserialize;
    pub fn serialize<S>(key: &SigningKey<Secp256k1>, seriliazer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        seriliazer.serialize_bytes(&key.to_bytes())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<SigningKey<Secp256k1>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let bytes: Vec<u8> = Vec::<u8>::deserialize(deserializer)?;
        Ok(SigningKey::from_slice(&bytes).unwrap())
    }
}

#[derive(Debug)]
pub enum MyError {
    Io(std::io::Error),
    Parse(std::num::ParseIntError),
    NotFound(String),
}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MyError::Io(e) => {
                write!(f, "I/O Error: {}", e)
            }
            MyError::Parse(e) => {
                write!(f, "Parse error {}", e)
            }
            MyError::NotFound(msg) => {
                write!(f, "Not Found {}", msg)
            }
        }
    }
}

impl Error for MyError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            MyError::Io(e) => Some(e),
            MyError::Parse(e) => Some(e),
            MyError::NotFound(_) => None,
        }
    }
}

impl From<std::io::Error> for MyError {
    fn from(error: std::io::Error) -> Self {
        MyError::Io(error)
    }
}

impl From<std::num::ParseIntError> for MyError {
    fn from(error: std::num::ParseIntError) -> Self {
        MyError::Parse(error)
    }
}
