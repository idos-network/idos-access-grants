use std::{env, fs};
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use near_units::parse_near;
use serde::Deserialize;
use serde_json::json;
use workspaces::{Account, Contract};
use workspaces::result::ExecutionFinalResult;

#[derive(Deserialize)]
pub struct Grant {
    grantee: String,
    data_id: String,
}

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
    test_everything(&alice, &contract).await?;

    Ok(())
}

async fn test_everything(
    user: &Account,
    contract: &Contract,
) -> anyhow::Result<()> {
    let owner = user.id().as_str();

    let contract_call = |method, args|
        user.call(contract.id(), method).args_json(args).transact();

    let parse_grants = |result: ExecutionFinalResult|
        result.json::<Vec<Grant>>().unwrap();

    let mut result;

    result = contract_call("grants_by", json!({
        "grantee": "bob.near",
        "data_id": "A1",
    })).await?;
    assert!(result.is_success());
    assert_eq!(parse_grants(result).len(), 0);

    result = contract_call("insert_grant", json!({
        "grantee": "bob.near",
        "data_id": "A1",
    })).await?;
    assert!(result.is_success());

    result = contract_call("insert_grant", json!({
        "grantee": "bob.near",
        "data_id": "A1",
    })).await?;
    assert!(result.is_failure());
    // TODO assert that error is "Grant already exists"

    result = contract_call("insert_grant", json!({
        "grantee": "bob.near",
        "data_id": "A2",
    })).await?;
    assert!(result.is_success());

    result = contract_call("insert_grant", json!({
        "grantee": "charlie.near",
        "data_id": "A2",
    })).await?;
    assert!(result.is_success());

    result = contract_call("grants_by", json!({
        "owner": owner,
    })).await?;
    assert!(result.is_success());
    assert_eq!(parse_grants(result).len(), 3);

    result = contract_call("grants_by", json!({
        "grantee": "bob.near",
    })).await?;
    assert!(result.is_success());
    assert_eq!(parse_grants(result).len(), 2);

    result = contract_call("grants_by", json!({
        "owner": owner,
        "grantee": "bob.near",
    })).await?;
    assert!(result.is_success());
    assert_eq!(parse_grants(result).len(), 2);

    result = contract_call("grants_by", json!({
        "owner": owner,
        "data_id": "A2",
    })).await?;
    assert!(result.is_success());
    assert_eq!(parse_grants(result).len(), 2);

    result = contract_call("grants_by", json!({
        "grantee": "bob.near",
        "data_id": "A1",
    })).await?;
    assert!(result.is_success());
    assert_eq!(parse_grants(result).len(), 1);

    result = contract_call("grants_by", json!({
        "grantee": "charlie.near",
        "data_id": "A1"
    })).await?;
    assert!(result.is_success());
    assert_eq!(parse_grants(result).len(), 0);

    result = contract_call("delete_grant", json!({
        "grantee": "bob.near",
        "data_id": "A1",
    })).await?;
    assert!(result.is_success());

    result = contract_call("grants_by", json!({
        "grantee": "bob.near",
    })).await?;
    assert!(result.is_success());
    assert_eq!(parse_grants(result).len(), 1);

    result = contract_call("grants_by", json!({
        "grantee": "bob.near",
        "data_id": "A1",
    })).await?;
    assert!(result.is_success());
    assert_eq!(parse_grants(result).len(), 0);

    result = contract_call("grants_by", json!({
        "owner": owner,
    })).await?;
    assert!(result.is_success());
    assert_eq!(parse_grants(result).len(), 2);

    let in_one_minute = SystemTime::now().duration_since(UNIX_EPOCH)? + Duration::from_secs(60);

    result = contract_call("insert_grant", json!({
        "grantee": "dave.near",
        "data_id": "A2",
        "locked_until": in_one_minute.as_nanos(),
    })).await?;
    assert!(result.is_success());

    result = contract_call("delete_grant", json!({
        "grantee": "dave.near",
        "data_id": "A2",
    })).await?;
    assert!(result.is_failure());
    // TODO assert that error is "Grant is timelocked"

    println!("      Passed ✅ test_everything");
    Ok(())
}
