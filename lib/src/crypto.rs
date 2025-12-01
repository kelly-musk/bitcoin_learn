use crate::{sha256::Hash, signkey_serde};
use ecdsa::signature::Verifier;
use ecdsa::{
    Signature as ECDSASignature, SigningKey, VerifyingKey,
    signature::{SignerMut, rand_core},
};
use k256::Secp256k1;
use rand_core::OsRng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Signature(ECDSASignature<Secp256k1>);

impl Signature {
    // pub fn sign_output(output_hash: &Hash, private_key: &PrivateKey) -> Self {
    //     let signing_key = &private_key.0;
    //     let signature = signing_key.sign(&output_hash.as_bytes());
    //     Signature(signature)
    // }

    pub fn sign_out(output_hash: &Hash, private_key: &mut PrivateKey) -> Self {
        let signnig_key = &mut private_key.0;
        let signature = signnig_key.sign(&output_hash.as_bytes());
        Signature(signature)
    }

    // verify a signature
    pub fn verify(&self, output_hash: &Hash, public_key: &PublicKey) -> bool {
        public_key
            .0
            .verify(&output_hash.as_bytes(), &self.0)
            .is_ok()
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone)]
pub struct PublicKey(VerifyingKey<Secp256k1>);

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PrivateKey(#[serde(with = "signkey_serde")] pub SigningKey<Secp256k1>);

// pub fn serializer<S>(key: &SigningKey<Secp256k1>, seriliazer: S) -> Result<S::Ok, S::Error>
// where
//     S: serde::Serializer,
// {
//     seriliazer.serialize_bytes(&key.to_bytes())
// }

// pub fn deserialize<'de, D>(deserializer: D) -> Result<SigningKey<Secp256k1>, D::Error>
// where
//     D: serde::Deserializer<'de>,
// {
//     let bytes: Vec<u8> = Vec::<u8>::deserialize(deserializer)?;
//     Ok(SigningKey::from_slice(&bytes).unwrap())
// }

impl PrivateKey {
    pub fn new_key() -> Self {
        let mut rng = OsRng;
        PrivateKey(SigningKey::random(&mut rng))
    }

    pub fn public_key(&self) -> PublicKey {
        PublicKey(self.0.verifying_key().clone())
    }
}
