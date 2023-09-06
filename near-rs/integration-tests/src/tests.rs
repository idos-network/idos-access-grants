use std::{env, fs};
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use near_units::parse_near;
use serde::Deserialize;
use serde_json::json;
use workspaces::{Account, Contract};

#[derive(Deserialize, Debug, PartialEq)]
pub struct Grant {
    grantee: String,
    data_id: String,
    locked_until: u128,
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

    let mut result;
    let mut grants;

    grants = user
        .call(contract.id(), "find_grants")
        .args_json(json!({"grantee": "bob.near", "data_id": "A1"}))
        .view()
        .await?
        .json::<Vec<Grant>>().unwrap();
    assert_eq!(grants, vec![
    ]);

    result = user
        .call(contract.id(), "insert_grant")
        .args_json(json!({"grantee": "bob.near", "data_id": "A1"}))
        .transact()
        .await?;
    assert!(result.is_success());

    result = user
        .call(contract.id(), "insert_grant")
        .args_json(json!({"grantee": "bob.near", "data_id": "A1"}))
        .transact()
        .await?;
    assert!(result.is_failure());
    assert!(result.into_result().unwrap_err().to_string().contains("Grant already exists"));

    result = user
        .call(contract.id(), "insert_grant")
        .args_json(json!({"grantee": "bob.near", "data_id": "A2"}))
        .transact()
        .await?;
    assert!(result.is_success());

    result = user
        .call(contract.id(), "insert_grant")
        .args_json(json!({"grantee": "charlie.near", "data_id": "A2"}))
        .transact()
        .await?;
    assert!(result.is_success());

    grants = user
        .call(contract.id(), "find_grants")
        .args_json(json!({"owner": owner}))
        .view()
        .await?
        .json::<Vec<Grant>>().unwrap();
    assert_eq!(grants, vec![
        Grant { grantee: "bob.near".into(), data_id: "A1".into(), locked_until: 0 },
        Grant { grantee: "bob.near".into(), data_id: "A2".into(), locked_until: 0 },
        Grant { grantee: "charlie.near".into(), data_id: "A2".into(), locked_until: 0 },
    ]);

    grants = user
        .call(contract.id(), "find_grants")
        .args_json(json!({"grantee": "bob.near"}))
        .view()
        .await?
        .json::<Vec<Grant>>().unwrap();
    assert_eq!(grants, vec![
        Grant { grantee: "bob.near".into(), data_id: "A1".into(), locked_until: 0 },
        Grant { grantee: "bob.near".into(), data_id: "A2".into(), locked_until: 0 },
    ]);

    grants = user
        .call(contract.id(), "find_grants")
        .args_json(json!({"owner": owner, "grantee": "bob.near"}))
        .view()
        .await?
        .json::<Vec<Grant>>().unwrap();
    assert_eq!(grants, vec![
        Grant { grantee: "bob.near".into(), data_id: "A1".into(), locked_until: 0 },
        Grant { grantee: "bob.near".into(), data_id: "A2".into(), locked_until: 0 },
    ]);

    grants = user
        .call(contract.id(), "find_grants")
        .args_json(json!({"owner": owner, "data_id": "A2"}))
        .view()
        .await?
        .json::<Vec<Grant>>().unwrap();
    assert_eq!(grants, vec![
        Grant { grantee: "bob.near".into(), data_id: "A2".into(), locked_until: 0 },
        Grant { grantee: "charlie.near".into(), data_id: "A2".into(), locked_until: 0 },
    ]);

    grants = user
        .call(contract.id(), "find_grants")
        .args_json(json!({"grantee": "bob.near", "data_id": "A1"}))
        .view()
        .await?
        .json::<Vec<Grant>>().unwrap();
    assert_eq!(grants, vec![
        Grant { grantee: "bob.near".into(), data_id: "A1".into(), locked_until: 0 },
    ]);

    grants = user
        .call(contract.id(), "find_grants")
        .args_json(json!({"grantee": "charlie.near", "data_id": "A1"}))
        .view()
        .await?
        .json::<Vec<Grant>>().unwrap();
    assert_eq!(grants, vec![
    ]);

    result = user
        .call(contract.id(), "delete_grant")
        .args_json(json!({"grantee": "bob.near", "data_id": "A1"}))
        .transact()
        .await?;
    assert!(result.is_success());

    grants = user
        .call(contract.id(), "find_grants")
        .args_json(json!({"grantee": "bob.near"}))
        .view()
        .await?
        .json::<Vec<Grant>>().unwrap();
    assert_eq!(grants, vec![
        Grant { grantee: "bob.near".into(), data_id: "A2".into(), locked_until: 0 },
    ]);

    grants = user
        .call(contract.id(), "find_grants")
        .args_json(json!({"grantee": "bob.near", "data_id": "A1"}))
        .view()
        .await?
        .json::<Vec<Grant>>().unwrap();
    assert_eq!(grants, vec![
    ]);

    grants = user
        .call(contract.id(), "find_grants")
        .args_json(json!({"owner": owner}))
        .view()
        .await?
        .json::<Vec<Grant>>().unwrap();
    assert_eq!(grants, vec![
        Grant { grantee: "bob.near".into(), data_id: "A2".into(), locked_until: 0 },
        Grant { grantee: "charlie.near".into(), data_id: "A2".into(), locked_until: 0 },
    ]);

    let in_the_future = (SystemTime::now().duration_since(UNIX_EPOCH)? + Duration::from_secs(3600)).as_nanos();
    let in_the_past = (SystemTime::now().duration_since(UNIX_EPOCH)? - Duration::from_secs(3600)).as_nanos();
    let in_the_paster = (SystemTime::now().duration_since(UNIX_EPOCH)? - 2 * Duration::from_secs(3600)).as_nanos();
    let in_the_pastest = (SystemTime::now().duration_since(UNIX_EPOCH)? - 3 * Duration::from_secs(3600)).as_nanos();

    result = user
        .call(contract.id(), "insert_grant")
        .args_json(json!({"grantee": "dave.near", "data_id": "A2", "locked_until": in_the_future}))
        .transact()
        .await?;
    assert!(result.is_success());

    result = user
        .call(contract.id(), "delete_grant")
        .args_json(json!({"grantee": "dave.near", "data_id": "A2"}))
        .transact()
        .await?;
    assert!(result.is_failure());
    assert!(result.into_result().unwrap_err().to_string().contains("Grant is timelocked"));

    result = user
        .call(contract.id(), "insert_grant")
        .args_json(json!({"grantee": "eve.near", "data_id": "A3", "locked_until": in_the_past}))
        .transact()
        .await?;
    assert!(result.is_success());

    result = user
        .call(contract.id(), "delete_grant")
        .args_json(json!({"grantee": "eve.near", "data_id": "A3", "locked_until": in_the_past}))
        .transact()
        .await?;
    assert!(result.is_success());

    grants = user
        .call(contract.id(), "find_grants")
        .args_json(json!({"grantee": "eve.near"}))
        .view()
        .await?
        .json::<Vec<Grant>>().unwrap();
    assert_eq!(grants, vec![
    ]);

    result = user
        .call(contract.id(), "insert_grant")
        .args_json(json!({"grantee": "eve.near", "data_id": "A3", "locked_until": in_the_past}))
        .transact()
        .await?;
    assert!(result.is_success());

    result = user
        .call(contract.id(), "insert_grant")
        .args_json(json!({"grantee": "eve.near", "data_id": "A3", "locked_until": in_the_paster}))
        .transact()
        .await?;
    assert!(result.is_success());

    result = user
        .call(contract.id(), "insert_grant")
        .args_json(json!({"grantee": "eve.near", "data_id": "A3", "locked_until": in_the_pastest}))
        .transact()
        .await?;
    assert!(result.is_success());

    result = user
        .call(contract.id(), "delete_grant")
        .args_json(json!({"grantee": "eve.near", "data_id": "A3", "locked_until": in_the_past}))
        .transact()
        .await?;
    assert!(result.is_success());

    grants = user
        .call(contract.id(), "find_grants")
        .args_json(json!({"grantee": "eve.near", "data_id": "A3"}))
        .view()
        .await?
        .json::<Vec<Grant>>().unwrap();
    assert_eq!(grants, vec![
        Grant { grantee: "eve.near".into(), data_id: "A3".into(), locked_until: in_the_paster },
        Grant { grantee: "eve.near".into(), data_id: "A3".into(), locked_until: in_the_pastest },
    ]);

    result = user
        .call(contract.id(), "delete_grant")
        .args_json(json!({"grantee": "eve.near", "data_id": "A3", "locked_until": 0}))
        .transact()
        .await?;
    assert!(result.is_success());

    grants = user
        .call(contract.id(), "find_grants")
        .args_json(json!({"grantee": "eve.near", "data_id": "A3"}))
        .view()
        .await?
        .json::<Vec<Grant>>().unwrap();
    assert_eq!(grants, vec![
    ]);

    println!("      Passed ✅ test_everything");
    Ok(())
}
