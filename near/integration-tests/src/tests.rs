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
    test_something(&alice, &contract).await?;
    Ok(())
}

async fn test_something(
    user: &Account,
    contract: &Contract,
) -> anyhow::Result<()> {
    user
        .call( contract.id(), "grants_for")
        .args_json(json!({"grantee": "julio.near", "data_id": "42"}))
        .transact()
        .await?;

    assert_eq!(0, 0);
    println!("      Passed ✅ test something");
    Ok(())
}
