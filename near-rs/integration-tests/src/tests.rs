use std::{
    env, fs,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use near_workspaces::{types::SecretKey, Account, Contract};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize, Debug, PartialEq)]
pub struct Grant {
    owner: String,
    grantee: String,
    data_id: String,
    locked_until: u128,
}

fn extract_public_key(secret_key: &SecretKey) -> String {
    secret_key.public_key().to_string()
}

async fn create_public_key() -> anyhow::Result<String> {
    Ok(extract_public_key(&SecretKey::from_random(
        near_workspaces::types::KeyType::ED25519,
    )))
}

const EVENT_JSON_PREFIX: &'static str = "EVENT_JSON";
const EVENT_JSON_SEPARATOR: &'static str = ":";
fn extract_event(s: &str) -> serde_json::Value {
    if let Some((EVENT_JSON_PREFIX, json_str)) = s.split_once(EVENT_JSON_SEPARATOR) {
        if let Ok(json_value) = json_str.parse::<serde_json::Value>() {
            return json_value;
        }
    }

    panic!(
        "Expected {:?} to start with {:?}, followed by {:?} and a valid JSON value.",
        s, EVENT_JSON_PREFIX, EVENT_JSON_SEPARATOR
    )
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let wasm_arg: &str = &(env::args().nth(1).unwrap());
    let wasm_filepath = fs::canonicalize(env::current_dir()?.join(wasm_arg))?;

    let worker = near_workspaces::sandbox().await?;
    let wasm = std::fs::read(wasm_filepath)?;
    let contract = worker.dev_deploy(&wasm).await?;

    // create accounts
    let test_account = worker
        .dev_create_account()
        .await?
        .create_subaccount("test")
        .transact()
        .await?
        .into_result()?;

    // begin tests
    test_everything(
        &contract,
        &test_account,
        &create_public_key().await?,
        &create_public_key().await?,
        &create_public_key().await?,
        &create_public_key().await?,
    )
    .await?;

    Ok(())
}

async fn test_everything(
    contract: &Contract,
    test_account: &Account,
    bob: &str,
    charlie: &str,
    dave: &str,
    eve: &str,
) -> anyhow::Result<()> {
    let mut result;
    let mut grants;
    let test_public_key: String = test_account.secret_key().public_key().to_string();

    grants = test_account
        .call(contract.id(), "find_grants")
        .args_json(json!({"grantee": bob, "data_id": "A1"}))
        .view()
        .await?
        .json::<Vec<Grant>>()
        .unwrap();
    assert_eq!(grants, vec![]);

    result = test_account
        .call(contract.id(), "insert_grant")
        .args_json(json!({"grantee": bob, "data_id": "A1"}))
        .transact()
        .await?;
    assert!(result.is_success());
    assert_eq!(result.logs().len(), 1);
    assert_eq!(
        extract_event(result.logs()[0]),
        json!({
            "standard": "FractalRegistry",
            "version": "0",
            "event": "grant_inserted",
            "data": {
                "owner": test_public_key,
                "grantee": bob,
                "data_id": "A1",
                "locked_until": 0,
            },
        }),
    );

    result = test_account
        .call(contract.id(), "insert_grant")
        .args_json(json!({"grantee": bob, "data_id": "A1"}))
        .transact()
        .await?;
    assert!(result.is_failure());
    assert!(result
        .into_result()
        .unwrap_err()
        .to_string()
        .contains("Grant already exists"));

    result = test_account
        .call(contract.id(), "insert_grant")
        .args_json(json!({"grantee": bob, "data_id": "A2"}))
        .transact()
        .await?;
    assert!(result.is_success());

    result = test_account
        .call(contract.id(), "insert_grant")
        .args_json(json!({"grantee": charlie, "data_id": "A2"}))
        .transact()
        .await?;
    assert!(
        result.is_success(),
        "{}",
        result.into_result().unwrap_err().to_string()
    );

    grants = test_account
        .call(contract.id(), "find_grants")
        .args_json(json!({ "owner": test_public_key }))
        .view()
        .await?
        .json::<Vec<Grant>>()
        .unwrap();
    assert_eq!(
        grants,
        vec![
            Grant {
                owner: test_public_key.clone(),
                grantee: bob.into(),
                data_id: "A1".into(),
                locked_until: 0
            },
            Grant {
                owner: test_public_key.clone(),
                grantee: bob.into(),
                data_id: "A2".into(),
                locked_until: 0
            },
            Grant {
                owner: test_public_key.clone(),
                grantee: charlie.into(),
                data_id: "A2".into(),
                locked_until: 0
            },
        ]
    );

    grants = test_account
        .call(contract.id(), "find_grants")
        .args_json(json!({ "grantee": bob }))
        .view()
        .await?
        .json::<Vec<Grant>>()
        .unwrap();
    assert_eq!(
        grants,
        vec![
            Grant {
                owner: test_public_key.clone(),
                grantee: bob.into(),
                data_id: "A1".into(),
                locked_until: 0
            },
            Grant {
                owner: test_public_key.clone(),
                grantee: bob.into(),
                data_id: "A2".into(),
                locked_until: 0
            },
        ]
    );

    grants = test_account
        .call(contract.id(), "find_grants")
        .args_json(json!({"owner": test_public_key, "grantee": bob}))
        .view()
        .await?
        .json::<Vec<Grant>>()
        .unwrap();
    assert_eq!(
        grants,
        vec![
            Grant {
                owner: test_public_key.clone(),
                grantee: bob.into(),
                data_id: "A1".into(),
                locked_until: 0
            },
            Grant {
                owner: test_public_key.clone(),
                grantee: bob.into(),
                data_id: "A2".into(),
                locked_until: 0
            },
        ]
    );

    grants = test_account
        .call(contract.id(), "find_grants")
        .args_json(json!({"owner": test_public_key, "data_id": "A2"}))
        .view()
        .await?
        .json::<Vec<Grant>>()
        .unwrap();
    assert_eq!(
        grants,
        vec![
            Grant {
                owner: test_public_key.clone(),
                grantee: bob.into(),
                data_id: "A2".into(),
                locked_until: 0
            },
            Grant {
                owner: test_public_key.clone(),
                grantee: charlie.into(),
                data_id: "A2".into(),
                locked_until: 0
            },
        ]
    );

    grants = test_account
        .call(contract.id(), "find_grants")
        .args_json(json!({"grantee": bob, "data_id": "A1"}))
        .view()
        .await?
        .json::<Vec<Grant>>()
        .unwrap();
    assert_eq!(
        grants,
        vec![Grant {
            owner: test_public_key.clone(),
            grantee: bob.into(),
            data_id: "A1".into(),
            locked_until: 0
        },]
    );

    grants = test_account
        .call(contract.id(), "find_grants")
        .args_json(json!({"grantee": charlie, "data_id": "A1"}))
        .view()
        .await?
        .json::<Vec<Grant>>()
        .unwrap();
    assert_eq!(grants, vec![]);

    result = test_account
        .call(contract.id(), "delete_grant")
        .args_json(json!({"grantee": bob, "data_id": "A1"}))
        .transact()
        .await?;
    assert!(result.is_success());
    assert_eq!(result.logs().len(), 1);
    assert_eq!(
        extract_event(result.logs()[0]),
        json!({
            "standard": "FractalRegistry",
            "version": "0",
            "event": "grant_deleted",
            "data": {
                "owner": test_public_key,
                "grantee": bob,
                "data_id": "A1",
                "locked_until": 0,
            },
        })
    );

    grants = test_account
        .call(contract.id(), "find_grants")
        .args_json(json!({ "grantee": bob }))
        .view()
        .await?
        .json::<Vec<Grant>>()
        .unwrap();
    assert_eq!(
        grants,
        vec![Grant {
            owner: test_public_key.clone(),
            grantee: bob.into(),
            data_id: "A2".into(),
            locked_until: 0
        },]
    );

    grants = test_account
        .call(contract.id(), "find_grants")
        .args_json(json!({"grantee": bob, "data_id": "A1"}))
        .view()
        .await?
        .json::<Vec<Grant>>()
        .unwrap();
    assert_eq!(grants, vec![]);

    grants = test_account
        .call(contract.id(), "find_grants")
        .args_json(json!({ "owner": test_public_key }))
        .view()
        .await?
        .json::<Vec<Grant>>()
        .unwrap();
    assert_eq!(
        grants,
        vec![
            Grant {
                owner: test_public_key.clone(),
                grantee: bob.into(),
                data_id: "A2".into(),
                locked_until: 0
            },
            Grant {
                owner: test_public_key.clone(),
                grantee: charlie.into(),
                data_id: "A2".into(),
                locked_until: 0
            },
        ]
    );

    let in_the_future =
        (SystemTime::now().duration_since(UNIX_EPOCH)? + Duration::from_secs(3600)).as_nanos();
    let in_the_past =
        (SystemTime::now().duration_since(UNIX_EPOCH)? - Duration::from_secs(3600)).as_nanos();
    let in_the_paster =
        (SystemTime::now().duration_since(UNIX_EPOCH)? - 2 * Duration::from_secs(3600)).as_nanos();
    let in_the_pastest =
        (SystemTime::now().duration_since(UNIX_EPOCH)? - 3 * Duration::from_secs(3600)).as_nanos();

    result = test_account
        .call(contract.id(), "insert_grant")
        .args_json(json!({"grantee": dave, "data_id": "A2", "locked_until": in_the_future}))
        .transact()
        .await?;
    assert!(result.is_success());

    result = test_account
        .call(contract.id(), "delete_grant")
        .args_json(json!({"grantee": dave, "data_id": "A2"}))
        .transact()
        .await?;
    assert!(result.is_failure());
    assert!(result
        .into_result()
        .unwrap_err()
        .to_string()
        .contains("Grant is timelocked"));

    result = test_account
        .call(contract.id(), "insert_grant")
        .args_json(json!({"grantee": eve, "data_id": "A3", "locked_until": in_the_past}))
        .transact()
        .await?;
    assert!(result.is_success());

    result = test_account
        .call(contract.id(), "delete_grant")
        .args_json(json!({"grantee": eve, "data_id": "A3", "locked_until": in_the_past}))
        .transact()
        .await?;
    assert!(result.is_success());

    grants = test_account
        .call(contract.id(), "find_grants")
        .args_json(json!({ "grantee": eve }))
        .view()
        .await?
        .json::<Vec<Grant>>()
        .unwrap();
    assert_eq!(grants, vec![]);

    result = test_account
        .call(contract.id(), "insert_grant")
        .args_json(json!({"grantee": eve, "data_id": "A3", "locked_until": in_the_past}))
        .transact()
        .await?;
    assert!(result.is_success());

    result = test_account
        .call(contract.id(), "insert_grant")
        .args_json(json!({"grantee": eve, "data_id": "A3", "locked_until": in_the_paster}))
        .transact()
        .await?;
    assert!(result.is_success());

    result = test_account
        .call(contract.id(), "insert_grant")
        .args_json(json!({"grantee": eve, "data_id": "A3", "locked_until": in_the_pastest}))
        .transact()
        .await?;
    assert!(result.is_success());

    result = test_account
        .call(contract.id(), "delete_grant")
        .args_json(json!({"grantee": eve, "data_id": "A3", "locked_until": in_the_past}))
        .transact()
        .await?;
    assert!(result.is_success());

    grants = test_account
        .call(contract.id(), "find_grants")
        .args_json(json!({"grantee": eve, "data_id": "A3"}))
        .view()
        .await?
        .json::<Vec<Grant>>()
        .unwrap();
    assert_eq!(
        grants,
        vec![
            Grant {
                owner: test_public_key.clone(),
                grantee: eve.into(),
                data_id: "A3".into(),
                locked_until: in_the_paster
            },
            Grant {
                owner: test_public_key.clone(),
                grantee: eve.into(),
                data_id: "A3".into(),
                locked_until: in_the_pastest
            },
        ]
    );

    result = test_account
        .call(contract.id(), "delete_grant")
        .args_json(json!({"grantee": eve, "data_id": "A3", "locked_until": 0}))
        .transact()
        .await?;
    assert!(result.is_success());

    grants = test_account
        .call(contract.id(), "find_grants")
        .args_json(json!({"grantee": eve, "data_id": "A3"}))
        .view()
        .await?
        .json::<Vec<Grant>>()
        .unwrap();
    assert_eq!(grants, vec![]);

    assert!(format!(
        "{:?}",
        test_account
            .view(contract.id(), "find_grants")
            .args_json(json!({"data_id": "A2"}))
            .await
            .expect_err("find_grants should have panicked")
    )
    .contains("Required argument: `owner` and/or `grantee`"));

    println!("      Passed ✅ test_everything");
    Ok(())
}
