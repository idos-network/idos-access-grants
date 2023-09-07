use std::{env, fs};
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use near_units::parse_near;
use serde::Deserialize;
use serde_json::json;
use workspaces::{Account, Contract};
use workspaces::result::{Result, ExecutionFinalResult};

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

pub struct ContractConnection<'a> {
    user: &'a Account,
    contract: &'a Contract,
}

impl <'a> ContractConnection<'_> {
    async fn transact(&self, method: &str, json_args: serde_json::Value) -> Result<ExecutionFinalResult> {
        self.user
            .call(self.contract.id(), method)
            .args_json(json_args)
            .transact()
            .await
    }

    async fn insert_grant(&self, json_args: serde_json::Value) -> Result<ExecutionFinalResult> {
        self.transact("insert_grant", json_args).await
    }

    async fn delete_grant(&self, json_args: serde_json::Value) -> Result<ExecutionFinalResult> {
        self.transact("delete_grant", json_args).await
    }

    async fn find_grants(&self, json_args: serde_json::Value) -> anyhow::Result<Vec<Grant>> {
        Ok(
            self.user
                .call(self.contract.id(), "find_grants")
                .args_json(json_args)
                .view()
                .await?
                .json::<Vec<Grant>>().unwrap()
        )
    }
}

trait FallibleResult {
    fn fails_with(&self, message: &str) -> bool;
}

impl FallibleResult for ExecutionFinalResult {
    fn fails_with(&self, message: &str) -> bool {
        self.is_failure() && self.clone().into_result().unwrap_err().to_string().contains(message)
    }
}

async fn test_everything(
    user: &Account,
    contract: &Contract,
) -> anyhow::Result<()> {
    let owner = user.id().as_str();
    let (mut grants, mut result);
    let connection = ContractConnection{ user, contract };

    grants = connection.find_grants(json!({
        "grantee": "bob.near",
        "data_id": "A1",
    })).await?;
    assert_eq!(grants.len(), 0);

    grants = connection.find_grants(json!({
        "grantee": "bob.near",
        "data_id": "A1",
    })).await?;
    assert_eq!(grants.len(), 0);

    result = connection.insert_grant(json!({
        "grantee": "bob.near",
        "data_id": "A1",
    })).await?;
    assert!(result.is_success());

    result = connection.insert_grant(json!({
        "grantee": "bob.near",
        "data_id": "A1",
    })).await?;
    assert!(result.fails_with("Grant already exists"));

    result = connection.insert_grant(json!({
        "grantee": "bob.near",
        "data_id": "A2",
    })).await?;
    assert!(result.is_success());

    result = connection.insert_grant(json!({
        "grantee": "charlie.near",
        "data_id": "A2",
    })).await?;
    assert!(result.is_success());

    grants = connection.find_grants(json!({
        "owner": owner,
    })).await?;
    assert_eq!(grants.len(), 3);

    grants = connection.find_grants(json!({
        "grantee": "bob.near",
    })).await?;
    assert_eq!(grants.len(), 2);

    grants = connection.find_grants(json!({
        "owner": owner,
        "grantee": "bob.near",
    })).await?;
    assert_eq!(grants.len(), 2);

    grants = connection.find_grants(json!({
        "owner": owner,
        "data_id": "A2",
    })).await?;
    assert_eq!(grants.len(), 2);

    grants = connection.find_grants(json!({
        "grantee": "bob.near",
        "data_id": "A1",
    })).await?;
    assert_eq!(grants.len(), 1);

    grants = connection.find_grants(json!({
        "grantee": "charlie.near",
        "data_id": "A1",
    })).await?;
    assert_eq!(grants.len(), 0);

    result = connection.delete_grant(json!({
        "grantee": "bob.near",
        "data_id": "A1",
    })).await?;
    assert!(result.is_success());

    grants = connection.find_grants(json!({
        "grantee": "bob.near",
    })).await?;
    assert_eq!(grants.len(), 1);

    grants = connection.find_grants(json!({
        "grantee": "bob.near",
        "data_id": "A1",
    })).await?;
    assert_eq!(grants.len(), 0);

    grants = connection.find_grants(json!({
        "owner": owner,
    })).await?;
    assert_eq!(grants.len(), 2);

    let in_the_future = (SystemTime::now().duration_since(UNIX_EPOCH)? + Duration::from_secs(3600)).as_nanos();
    let in_the_past = (SystemTime::now().duration_since(UNIX_EPOCH)? - Duration::from_secs(3600)).as_nanos();
    let in_the_paster = (SystemTime::now().duration_since(UNIX_EPOCH)? - 2 * Duration::from_secs(3600)).as_nanos();
    let in_the_pastest = (SystemTime::now().duration_since(UNIX_EPOCH)? - 3 * Duration::from_secs(3600)).as_nanos();

    result = connection.insert_grant(json!({
        "grantee": "dave.near",
        "data_id": "A2",
        "locked_until": in_the_future,
    })).await?;
    assert!(result.is_success());

    result = connection.delete_grant(json!({
        "grantee": "dave.near",
        "data_id": "A2",
    })).await?;
    assert!(result.fails_with("Grant is timelocked"));

    result = connection.insert_grant(json!({
        "grantee": "eve.near",
        "data_id": "A3",
        "locked_until": in_the_past,
    })).await?;
    assert!(result.is_success());

    result = connection.delete_grant(json!({
        "grantee": "eve.near",
        "data_id": "A3",
        "locked_until": in_the_past,
    })).await?;
    assert!(result.is_success());

    grants = connection.find_grants(json!({
        "grantee": "eve.near",
    })).await?;
    assert_eq!(grants.len(), 0);

    result = connection.insert_grant(json!({
        "grantee": "eve.near",
        "data_id": "A3",
        "locked_until": in_the_past,
    })).await?;
    assert!(result.is_success());

    result = connection.insert_grant(json!({
        "grantee": "eve.near",
        "data_id": "A3",
        "locked_until": in_the_paster,
    })).await?;
    assert!(result.is_success());

    result = connection.insert_grant(json!({
        "grantee": "eve.near",
        "data_id": "A3",
        "locked_until": in_the_pastest,
    })).await?;
    assert!(result.is_success());

    result = connection.delete_grant(json!({
        "grantee": "eve.near",
        "data_id": "A3",
        "locked_until": in_the_past,
    })).await?;
    assert!(result.is_success());

    grants = connection.find_grants(json!({
        "grantee": "eve.near",
        "data_id": "A3",
    })).await?;
    assert_eq!(grants.len(), 2);

    result = connection.delete_grant(json!({
        "grantee": "eve.near",
        "data_id": "A3",
        "locked_until": 0,
    })).await?;
    assert!(result.is_success());

    grants = connection.find_grants(json!({
        "grantee": "eve.near",
        "data_id": "A3",
    })).await?;
    assert_eq!(grants.len(), 0);

    println!("      Passed ✅ test_everything");
    Ok(())
}
