use serde::{Deserialize, Serialize};
use uint::construct_uint;
construct_uint! {
    #[derive(Serialize, Deserialize)]
    pub struct U256(4);
}
pub mod crypto;
pub mod sha256;
pub mod types;
pub mod utils;

pub mod signkey_serde {
    use ecdsa::{SigningKey};
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
