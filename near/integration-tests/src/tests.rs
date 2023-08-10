use std::{env, fs};
use near_units::parse_near;
use serde_json::json;
use workspaces::{Account, Contract};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let wasm_arg: &str = &(env::args().nth(1).unwrap());
    let wasm_filepath = fs::canonicalize(env::current_dir()?.join(wasm_arg))?;

    let worker = workspaces::sandbox().await?;
    let wasm = std::fs::read(wasm_filepath)?;
    let contract = worker.dev_deploy(&wasm).await?;

    // create accounts
    let account = worker.dev_create_account().await?;
    let alice = account
        .create_subaccount( "alice")
        .initial_balance(parse_near!("30 N"))
        .transact()
        .await?
        .into_result()?;

    // begin tests
    test_grants_for(&alice, &contract).await?;
    Ok(())
}

async fn test_grants_for(
    user: &Account,
    contract: &Contract,
) -> anyhow::Result<()> {
    let mut grants;

    grants = user
        .call( contract.id(), "grants_for")
        .args_json(json!({"grantee": "julio.near", "data_id": "42"}))
        .transact()
        .await?;

    assert!(grants.is_success());

    grants = user
        .call( contract.id(), "grants_for")
        .args_json(json!({"grantee": "julio.near"}))
        .transact()
        .await?;

    assert!(grants.is_failure());

    println!("      Passed ✅ test grants_for");
    Ok(())
}
