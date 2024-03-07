use borsh::BorshSerialize;
use rand::random;
use ring::digest::{digest, SHA256};
use serde::Serialize;

use near_workspaces::types::SecretKey;
use std::str::FromStr;

#[derive(BorshSerialize, Serialize)]
pub struct Payload {
    pub message: String,
    pub nonce: [u8; 32],
    pub recipient: String,
    #[serde(rename = "callbackUrl")]
    pub callback_url: Option<String>,
}

const NEP413_TAG: u32 = 2147484061; // 2**31 + 413
pub fn hashed_payload(payload: &Payload) -> Vec<u8> {
    let mut writer = vec![];

    borsh::to_writer(&mut writer, &NEP413_TAG).expect("Can't borsh encode NEP413_TAG");
    borsh::to_writer(&mut writer, payload).expect("Can't borsh encode payload");

    digest(&SHA256, writer.as_slice()).as_ref().into()
}

pub fn generate_nonce() -> [u8; 32] {
    let mut result = [0; 32];
    for x in result.iter_mut() {
        *x = random()
    }
    result
}

fn raw_sign(secret_key: SecretKey, data: &[u8]) -> [u8; 64] {
    let crypto_secret_key =
        near_crypto::SecretKey::from_str(secret_key.to_string().as_str()).unwrap();

    match crypto_secret_key.sign(data) {
        near_crypto::Signature::ED25519(signature) => signature.to_bytes(),
        _ => panic!("Only ED25519 keys supported"),
    }
}

pub fn sign(secret_key: SecretKey, payload: Payload) -> Vec<u8> {
    let bytes = raw_sign(secret_key, hashed_payload(&payload).as_slice());

    IntoIterator::into_iter(bytes).collect()
}
