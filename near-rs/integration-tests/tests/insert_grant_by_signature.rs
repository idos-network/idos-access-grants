use borsh::BorshSerialize;
use near_workspaces::{result::ExecutionFinalResult, types::SecretKey};
use rand::random;
use ring::digest::{digest, SHA256};
use serde::Serialize;
use serde_json::json;
use std::str::FromStr;

mod helpers;
use helpers::{create_public_key, create_secret_key, extract_public_key, scenario_base, Grant};

fn assert_transaction_success(result: ExecutionFinalResult) {
    assert!(
        result.is_success(),
        "{}",
        result.into_result().unwrap_err().to_string()
    );
}

fn assert_transaction_failure(result: ExecutionFinalResult, expected_error: &str) {
    assert!(result.is_failure());
    assert_eq!(
        result.into_result().unwrap_err().to_string(),
        expected_error
    );
}

#[derive(BorshSerialize, Serialize)]
struct Nep413Payload {
    pub message: String,
    pub nonce: [u8; 32],
    pub recipient: String,
    #[serde(rename = "callbackUrl")]
    pub callback_url: Option<String>,
}

const NEP413_TAG: u32 = 2147484061; // 2**31 + 413
fn nep413_hashed_payload(payload: &Nep413Payload) -> Vec<u8> {
    let mut writer = vec![];

    borsh::to_writer(&mut writer, &NEP413_TAG).expect("Can't borsh encode NEP413_TAG");
    borsh::to_writer(&mut writer, payload).expect("Can't borsh encode payload");

    digest(&SHA256, writer.as_slice()).as_ref().into()
}

fn generate_nonce<const N: usize>() -> [u8; N] {
    let mut result = [0; N];
    for x in result.iter_mut() {
        *x = random()
    }
    result
}

fn sign(secret_key: SecretKey, data: Vec<u8>) -> Vec<u8> {
    if let near_crypto::Signature::ED25519(signature) =
        near_crypto::SecretKey::from_str(secret_key.to_string().as_str())
            .unwrap()
            .sign(data.as_slice())
    {
        IntoIterator::into_iter(signature.to_bytes()).collect()
    } else {
        unreachable!("Not an ED25519 signature")
    }
}

#[tokio::test]
async fn happy_path() -> anyhow::Result<()> {
    let (_, contract, test_account) = scenario_base().await?;

    let owner_sk = create_secret_key();
    let owner = extract_public_key(&owner_sk);

    let grantee = create_public_key();
    let data_id: String = "DATA_ID".into();
    let locked_until = 0;
    let nonce: [u8; 32] = generate_nonce();

    assert_eq!(
        test_account
            .call(contract.id(), "find_grants")
            .args_json(json!({"owner": owner, "grantee": grantee}))
            .view()
            .await?
            .json::<Vec<Grant>>()
            .unwrap(),
        vec![]
    );

    let recipient = test_account
        .call(contract.id(), "grant_message_recipient")
        .args_json(json!({}))
        .view()
        .await?
        .json::<String>()
        .unwrap();

    let message = test_account
        .call(contract.id(), "insert_grant_by_signature_message")
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

    let hashed_payload = nep413_hashed_payload(&Nep413Payload {
        message,
        nonce,
        recipient,
        callback_url: None,
    });

    let signature = sign(owner_sk, hashed_payload);

    assert_transaction_success(
        test_account
            .call(contract.id(), "insert_grant_by_signature")
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
        test_account
            .call(contract.id(), "find_grants")
            .args_json(json!({"owner": owner, "grantee": grantee}))
            .view()
            .await?
            .json::<Vec<Grant>>()
            .unwrap(),
        vec![Grant {
            owner,
            grantee,
            data_id,
            locked_until,
        }]
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
    let nonce: [u8; 32] = generate_nonce();

    assert_transaction_failure(
        test_account
            .call(contract.id(), "insert_grant_by_signature")
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
