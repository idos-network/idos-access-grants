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

    result = contract_call("find_grants", json!({
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
    assert!(result.into_result().unwrap_err().to_string().contains("Grant already exists"));

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

    result = contract_call("find_grants", json!({
        "owner": owner,
    })).await?;
    assert!(result.is_success());
    assert_eq!(parse_grants(result).len(), 3);

    result = contract_call("find_grants", json!({
        "grantee": "bob.near",
    })).await?;
    assert!(result.is_success());
    assert_eq!(parse_grants(result).len(), 2);

    result = contract_call("find_grants", json!({
        "owner": owner,
        "grantee": "bob.near",
    })).await?;
    assert!(result.is_success());
    assert_eq!(parse_grants(result).len(), 2);

    result = contract_call("find_grants", json!({
        "owner": owner,
        "data_id": "A2",
    })).await?;
    assert!(result.is_success());
    assert_eq!(parse_grants(result).len(), 2);

    result = contract_call("find_grants", json!({
        "grantee": "bob.near",
        "data_id": "A1",
    })).await?;
    assert!(result.is_success());
    assert_eq!(parse_grants(result).len(), 1);

    result = contract_call("find_grants", json!({
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

    result = contract_call("find_grants", json!({
        "grantee": "bob.near",
    })).await?;
    assert!(result.is_success());
    assert_eq!(parse_grants(result).len(), 1);

    result = contract_call("find_grants", json!({
        "grantee": "bob.near",
        "data_id": "A1",
    })).await?;
    assert!(result.is_success());
    assert_eq!(parse_grants(result).len(), 0);

    result = contract_call("find_grants", json!({
        "owner": owner,
    })).await?;
    assert!(result.is_success());
    assert_eq!(parse_grants(result).len(), 2);

    let in_the_future = SystemTime::now().duration_since(UNIX_EPOCH)? + Duration::from_secs(3600);
    let in_the_past = SystemTime::now().duration_since(UNIX_EPOCH)? - Duration::from_secs(3600);
    let in_the_paster = SystemTime::now().duration_since(UNIX_EPOCH)? - 2 * Duration::from_secs(3600);
    let in_the_pastest = SystemTime::now().duration_since(UNIX_EPOCH)? - 3 * Duration::from_secs(3600);

    result = contract_call("insert_grant", json!({
        "grantee": "dave.near",
        "data_id": "A2",
        "locked_until": in_the_future.as_nanos(),
    })).await?;
    assert!(result.is_success());

    result = contract_call("delete_grant", json!({
        "grantee": "dave.near",
        "data_id": "A2",
    })).await?;
    assert!(result.is_failure());
    assert!(result.into_result().unwrap_err().to_string().contains("Grant is timelocked"));

    result = contract_call("insert_grant", json!({
        "grantee": "eve.near",
        "data_id": "A3",
        "locked_until": in_the_past.as_nanos(),
    })).await?;
    assert!(result.is_success());

    result = contract_call("delete_grant", json!({
        "grantee": "eve.near",
        "data_id": "A3",
        "locked_until": in_the_past.as_nanos(),
    })).await?;
    assert!(result.is_success());

    result = contract_call("find_grants", json!({
        "grantee": "eve.near",
    })).await?;
    assert!(result.is_success());
    assert_eq!(parse_grants(result).len(), 0);

    result = contract_call("insert_grant", json!({
        "grantee": "eve.near",
        "data_id": "A3",
        "locked_until": in_the_past.as_nanos(),
    })).await?;
    assert!(result.is_success());

    result = contract_call("insert_grant", json!({
        "grantee": "eve.near",
        "data_id": "A3",
        "locked_until": in_the_paster.as_nanos(),
    })).await?;
    assert!(result.is_success());

    result = contract_call("insert_grant", json!({
        "grantee": "eve.near",
        "data_id": "A3",
        "locked_until": in_the_pastest.as_nanos(),
    })).await?;
    assert!(result.is_success());

    result = contract_call("delete_grant", json!({
        "grantee": "eve.near",
        "data_id": "A3",
        "locked_until": in_the_past.as_nanos(),
    })).await?;
    assert!(result.is_success());

    result = contract_call("find_grants", json!({
        "grantee": "eve.near",
        "data_id": "A3",
    })).await?;
    assert!(result.is_success());
    assert_eq!(parse_grants(result).len(), 2);

    result = contract_call("delete_grant", json!({
        "grantee": "eve.near",
        "data_id": "A3",
        "locked_until": 0,
    })).await?;
    assert!(result.is_success());

    result = contract_call("find_grants", json!({
        "grantee": "eve.near",
        "data_id": "A3",
    })).await?;
    assert!(result.is_success());
    assert_eq!(parse_grants(result).len(), 0);

    println!("      Passed ✅ test_everything");
    Ok(())
}
