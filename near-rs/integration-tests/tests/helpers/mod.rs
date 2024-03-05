use lazy_static::lazy_static;

use std::{env, fs};

use near_workspaces::{network::Sandbox, types::SecretKey, Account, Contract, Worker};
use serde::Deserialize;

#[derive(Deserialize, Debug, PartialEq)]
pub struct Grant {
    pub owner: String,
    pub grantee: String,
    pub data_id: String,
    pub locked_until: u128,
}

pub fn create_secret_key() -> SecretKey {
    SecretKey::from_random(near_workspaces::types::KeyType::ED25519)
}

pub fn extract_public_key(secret_key: &SecretKey) -> String {
    secret_key.public_key().to_string()
}

pub fn create_public_key() -> String {
    extract_public_key(&create_secret_key())
}

lazy_static! {
    static ref WASM: Vec<u8> = {
        let wasm_arg: String = env::var("CONTRACT_LOCATION").unwrap_or(
            "../contract/target/wasm32-unknown-unknown/release/access_grants.wasm".into(),
        );
        let wasm_filepath = fs::canonicalize(env::current_dir().unwrap().join(wasm_arg)).unwrap();
        std::fs::read(wasm_filepath).unwrap()
    };
}

pub async fn scenario_base() -> anyhow::Result<(Worker<Sandbox>, Contract, Account)> {
    let worker = near_workspaces::sandbox().await?;
    let contract = worker.dev_deploy(&WASM).await?;
    let test_account = worker
        .dev_create_account()
        .await?
        .create_subaccount("test")
        .transact()
        .await?
        .into_result()?;
    Ok((worker, contract, test_account))
}
