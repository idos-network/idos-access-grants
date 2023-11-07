use std::{
    env, fs,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use serde::Deserialize;
use serde_json::json;
use workspaces::{Account, Contract};

#[derive(Deserialize, Debug, PartialEq)]
pub struct Grant {
    owner: String,
    grantee: String,
    data_id: String,
    locked_until: u128,
}

async fn create_subaccount(new_account_id: &str, account: &Account) -> anyhow::Result<Account> {
    Ok(account
        .create_subaccount(new_account_id)
        .transact()
        .await?
        .into_result()?)
}

fn extract_id(account: &Account) -> String {
    account.id().clone().into()
}

async fn create_id(new_account_id: &str, account: &Account) -> anyhow::Result<String> {
    Ok(extract_id(&create_subaccount(new_account_id, account).await?))
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
    let test_account = create_subaccount("test", &account).await?;
    // begin tests
    test_everything(
        &contract,
        &test_account,
        &extract_id(&test_account),
        &create_id("bob", &account).await?,
        &create_id("charlie", &account).await?,
        &create_id("dave", &account).await?,
        &create_id("eve", &account).await?,
    )
    .await?;

    Ok(())
}

async fn test_everything(
    contract: &Contract,
    test_account: &Account,
    test: &str,
    bob: &str,
    charlie: &str,
    dave: &str,
    eve: &str,
) -> anyhow::Result<()> {
    let mut result;
    let mut grants;

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
    assert!(result.is_success());

    grants = test_account
        .call(contract.id(), "find_grants")
        .args_json(json!({ "owner": test }))
        .view()
        .await?
        .json::<Vec<Grant>>()
        .unwrap();
    assert_eq!(
        grants,
        vec![
            Grant {
                owner: test.into(),
                grantee: bob.into(),
                data_id: "A1".into(),
                locked_until: 0
            },
            Grant {
                owner: test.into(),
                grantee: bob.into(),
                data_id: "A2".into(),
                locked_until: 0
            },
            Grant {
                owner: test.into(),
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
                owner: test.into(),
                grantee: bob.into(),
                data_id: "A1".into(),
                locked_until: 0
            },
            Grant {
                owner: test.into(),
                grantee: bob.into(),
                data_id: "A2".into(),
                locked_until: 0
            },
        ]
    );

    grants = test_account
        .call(contract.id(), "find_grants")
        .args_json(json!({"owner": test, "grantee": bob}))
        .view()
        .await?
        .json::<Vec<Grant>>()
        .unwrap();
    assert_eq!(
        grants,
        vec![
            Grant {
                owner: test.into(),
                grantee: bob.into(),
                data_id: "A1".into(),
                locked_until: 0
            },
            Grant {
                owner: test.into(),
                grantee: bob.into(),
                data_id: "A2".into(),
                locked_until: 0
            },
        ]
    );

    grants = test_account
        .call(contract.id(), "find_grants")
        .args_json(json!({"owner": test, "data_id": "A2"}))
        .view()
        .await?
        .json::<Vec<Grant>>()
        .unwrap();
    assert_eq!(
        grants,
        vec![
            Grant {
                owner: test.into(),
                grantee: bob.into(),
                data_id: "A2".into(),
                locked_until: 0
            },
            Grant {
                owner: test.into(),
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
            owner: test.into(),
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
            owner: test.into(),
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
        .args_json(json!({ "owner": test }))
        .view()
        .await?
        .json::<Vec<Grant>>()
        .unwrap();
    assert_eq!(
        grants,
        vec![
            Grant {
                owner: test.into(),
                grantee: bob.into(),
                data_id: "A2".into(),
                locked_until: 0
            },
            Grant {
                owner: test.into(),
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
                owner: test.into(),
                grantee: eve.into(),
                data_id: "A3".into(),
                locked_until: in_the_paster
            },
            Grant {
                owner: test.into(),
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

    assert!(test_account
        .call(contract.id(), "find_grants")
        .args_json(json!({"data_id": "A2"}))
        .view()
        .await
        .unwrap_err()
        .to_string()
        .contains("Required argument: `owner` and/or `grantee`"));

    println!("      Passed ✅ test_everything");
    Ok(())
}
