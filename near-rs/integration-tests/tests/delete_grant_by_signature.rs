use serde_json::json;

mod helpers;
use helpers::{create_public_key, scenario_base, Grant};

mod assert;

mod nep413;

#[tokio::test]
async fn happy_path() -> anyhow::Result<()> {
    let (worker, contract, some_other_account) = scenario_base().await?;

    let (owner_id, owner_sk) = worker.dev_generate().await;
    let owner_account = worker
        .create_tla(owner_id.clone(), owner_sk.clone())
        .await?
        .unwrap();
    let owner = owner_sk.public_key();

    let grantee = create_public_key();
    let data_id: String = "DATA_ID".into();
    let locked_until = 0;
    let nonce = nep413::generate_nonce();

    assert::transaction_success(
        owner_account
            .call(contract.id(), "insert_grant")
            .args_json(json!({"grantee": grantee, "data_id": data_id}))
            .transact()
            .await?,
    );

    let recipient = some_other_account
        .call(contract.id(), "grant_message_recipient")
        .args_json(json!({}))
        .view()
        .await?
        .json::<String>()
        .unwrap();

    let message = some_other_account
        .call(contract.id(), "delete_grant_by_signature_message")
        .args_json(json!({
            "owner": owner,
            "grantee": grantee,
            "data_id": data_id,
            "locked_until": locked_until,
        }))
        .view()
        .await?
        .json::<String>()
        .unwrap();

    let signature = nep413::sign(
        owner_sk,
        nep413::Payload {
            message,
            nonce,
            recipient,
            callback_url: None,
        },
    );

    assert::transaction_success(
        some_other_account
            .call(contract.id(), "delete_grant_by_signature")
            .args_json(json!({
                "owner": owner,
                "grantee": grantee,
                "data_id": data_id,
                "locked_until": locked_until,
                "nonce": nonce,
                "signature": signature,
            }))
            .transact()
            .await?,
    );

    assert_eq!(
        some_other_account
            .call(contract.id(), "find_grants")
            .args_json(json!({"owner": owner, "grantee": grantee}))
            .view()
            .await?
            .json::<Vec<Grant>>()
            .unwrap(),
        vec![],
    );

    Ok(())
}

#[tokio::test]
async fn wrong_signature() -> anyhow::Result<()> {
    let (_, contract, test_account) = scenario_base().await?;

    let owner = create_public_key();
    let grantee = create_public_key();
    let data_id: String = "DATA_ID".into();
    let locked_until = 0;
    let nonce = nep413::generate_nonce();

    assert::transaction_failure(
        test_account
            .call(contract.id(), "delete_grant_by_signature")
            .args_json(json!({
                "owner": owner,
                "grantee": grantee,
                "data_id": data_id,
                "locked_until": locked_until,
                "nonce": nonce,
                "signature": IntoIterator::into_iter([0u8; 64]).collect::<Vec<u8>>(),
            }))
            .transact()
            .await?,
        r#"Action #0: ExecutionError("Smart contract panicked: Signature doesn't match")"#,
    );

    Ok(())
}
