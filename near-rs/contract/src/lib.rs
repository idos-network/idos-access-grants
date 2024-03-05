// The `#[near_bindgen]` for `impl FractalRegistry` was triggering this, and I couldn't find a way to suppress it.
#![allow(clippy::too_many_arguments)]
extern crate near_sdk;

use std::convert::TryInto;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::Serialize;
use near_sdk::store::LookupMap;
use near_sdk::{env, near_bindgen, require, CurveType, EpochHeight, PublicKey};

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct FractalRegistry {
    pub grants_by_id: LookupMap<String, Grant>,

    pub grant_ids_by_owner: LookupMap<PublicKey, Vec<String>>,
    pub grant_ids_by_grantee: LookupMap<PublicKey, Vec<String>>,
    pub grant_ids_by_data_id: LookupMap<String, Vec<String>>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Grant {
    owner: PublicKey,
    grantee: PublicKey,
    data_id: String,
    locked_until: EpochHeight,
}

#[cfg(test)]
#[test]
fn derive_grant_id_example() {
    // Just to make sure we don't accidentally change the way we derive grant_ids.

    let grant = Grant {
        owner: "ed25519:BCUg4havhRURACQAFK48e6ScqcJgPbeqHbfcmNoWp3fZ"
            .parse()
            .unwrap(),
        grantee: "ed25519:mrjrfx8wSA9pYyMEeMm2QnFe9ct1P8CRkmU55h8MxEi"
            .parse()
            .unwrap(),
        data_id: "some data".into(),
        locked_until: 1337,
    };

    assert_eq!(
        "8031eff696fa15a7e4c69530a1d8b634faab8d512fde219b92aae0082adb8606",
        derive_grant_id(&grant)
    );
}

pub fn derive_grant_id(grant: &Grant) -> String {
    let id = format!(
        "{}{}{}{}",
        Into::<String>::into(&grant.owner),
        Into::<String>::into(&grant.grantee),
        grant.data_id,
        grant.locked_until,
    );

    hex::encode(env::keccak256(id.as_bytes()))
}

impl Default for FractalRegistry {
    fn default() -> Self {
        let grants_by_id = LookupMap::new(b"g");
        let grant_ids_by_owner = LookupMap::new(b"h");
        let grant_ids_by_grantee = LookupMap::new(b"i");
        let grant_ids_by_data_id = LookupMap::new(b"j");

        Self {
            grants_by_id,
            grant_ids_by_owner,
            grant_ids_by_grantee,
            grant_ids_by_data_id,
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Clone)]
#[serde(crate = "near_sdk::serde")]
struct Nep413Payload {
    pub message: String,
    pub nonce: [u8; 32],
    pub recipient: String,
    #[serde(rename = "callbackUrl")]
    pub callback_url: Option<String>,
}

const NEP413_TAG: u32 = 2147484061; // 2**31 + 413
fn nep413_hashed_payload(payload: &Nep413Payload) -> [u8; 32] {
    let mut writer = vec![];

    borsh::to_writer(&mut writer, &NEP413_TAG).expect("Can't borsh encode NEP413_TAG");
    borsh::to_writer(&mut writer, payload).expect("Can't borsh encode payload");

    env::sha256_array(&writer)
}

// Just so people don't pass in the wrong name for the message.
macro_rules! u8_to_fixed_length_array {
    ( $value:expr ) => {
        __u8_to_fixed_length_array($value, stringify!($value))
    };
}

fn __u8_to_fixed_length_array<'a, const N: usize>(
    source: &'a [u8],
    name: &'static str,
) -> &'a [u8; N] {
    source.try_into().unwrap_or_else(|_| {
        panic!(
            "{} doesn't seem to have exactly {} bytes: {:?}",
            name, N, source
        )
    })
}

#[cfg(test)]
#[test]
fn u8_to_fixed_length_array_example_good() {
    assert_eq!(
        [1, 2, 3],
        *u8_to_fixed_length_array!(vec![1, 2, 3].as_slice())
    );
}

#[cfg(test)]
#[test]
#[should_panic(expected = "original.as_slice() doesn't seem to have exactly 1 bytes: [1, 2, 3]")]
fn u8_to_fixed_length_array_example_bad() {
    let original = vec![1, 2, 3];
    let _var_name: [u8; 1] = *u8_to_fixed_length_array!(original.as_slice());
}

pub fn public_key_bytes_ref(public_key: &PublicKey) -> &[u8; 32] {
    // First byte is the curve type.
    u8_to_fixed_length_array!(&public_key.as_bytes()[1..])
}

#[near_bindgen(event_json(standard = "FractalRegistry"))]
pub enum FractalRegistryEvents {
    #[event_version("0")]
    GrantInserted {
        owner: PublicKey,
        grantee: PublicKey,
        data_id: String,
        locked_until: EpochHeight,
    },

    #[event_version("0")]
    GrantDeleted {
        owner: PublicKey,
        grantee: PublicKey,
        data_id: String,
        locked_until: EpochHeight,
    },
}

#[near_bindgen]
impl FractalRegistry {
    pub fn insert_grant(
        &mut self,
        grantee: PublicKey,
        data_id: String,
        locked_until: Option<EpochHeight>,
    ) {
        self._insert_grant(env::signer_account_pk(), grantee, data_id, locked_until)
    }

    pub fn insert_grant_by_signature_message(
        &self,
        owner: PublicKey,
        grantee: PublicKey,
        data_id: String,
        locked_until: Option<EpochHeight>,
    ) -> String {
        format!(
            "operation: insertGrant\n\
            owner: {}\n\
            grantee: {}\n\
            dataId: {}\n\
            lockedUntil: {}",
            Into::<String>::into(&owner),
            Into::<String>::into(&grantee),
            data_id,
            locked_until.unwrap_or(0)
        )
    }

    pub fn grant_message_recipient(&self) -> String {
        "idos.network".into()
    }

    pub fn insert_grant_by_signature(
        &mut self,
        owner: PublicKey,
        grantee: PublicKey,
        data_id: String,
        locked_until: Option<EpochHeight>,
        nonce: Vec<u8>,
        signature: Vec<u8>,
    ) {
        require!(
            owner.curve_type() == CurveType::ED25519,
            "Only ed25519 keys are supported",
        );

        // Serde didn't have [u8; 64] implemented, only up to 32. So, I've decided to convert them inside the function.
        let nonce: [u8; 32] = *u8_to_fixed_length_array!(nonce.as_slice());
        let signature: [u8; 64] = *u8_to_fixed_length_array!(signature.as_slice());

        let message = self.insert_grant_by_signature_message(
            owner.clone(),
            grantee.clone(),
            data_id.clone(),
            locked_until,
        );

        let hashed_payload = nep413_hashed_payload(&Nep413Payload {
            message,
            nonce,
            recipient: self.grant_message_recipient(),
            callback_url: None,
        });

        require!(
            env::ed25519_verify(
                &signature,
                &hashed_payload,
                public_key_bytes_ref(&owner),
            ),
            "Signature doesn't match"
        );

        self._insert_grant(owner, grantee, data_id, locked_until)
    }

    fn _insert_grant(
        &mut self,
        owner: PublicKey,
        grantee: PublicKey,
        data_id: String,
        locked_until: Option<EpochHeight>,
    ) {
        let grant = Grant {
            owner: owner.clone(),
            grantee: grantee.clone(),
            data_id: data_id.clone(),
            locked_until: locked_until.unwrap_or(0),
        };

        let grant_id = derive_grant_id(&grant);

        require!(
            !self.grants_by_id.contains_key(&grant_id),
            "Grant already exists"
        );

        self.grants_by_id.insert(grant_id.clone(), grant);

        self.grant_ids_by_owner
            .entry(owner.clone())
            .or_default()
            .push(grant_id.clone());

        self.grant_ids_by_grantee
            .entry(grantee.clone())
            .or_default()
            .push(grant_id.clone());

        self.grant_ids_by_data_id
            .entry(data_id.clone())
            .or_default()
            .push(grant_id.clone());

        let locked_until = locked_until.unwrap_or(0);

        FractalRegistryEvents::GrantInserted {
            owner,
            grantee,
            data_id,
            locked_until,
        }
        .emit();
    }

    pub fn delete_grant(
        &mut self,
        grantee: PublicKey,
        data_id: String,
        locked_until: Option<EpochHeight>,
    ) {
        let owner = env::signer_account_pk();

        self.find_grants(
            Some(owner.clone()),
            Some(grantee.clone()),
            Some(data_id.clone()),
        )
        .iter()
        .filter(|grant| match locked_until {
            None => true,
            Some(0) => true,
            Some(locked_until_) => grant.locked_until == locked_until_,
        })
        .for_each(|grant| {
            require!(
                grant.locked_until < env::block_timestamp(),
                "Grant is timelocked"
            );

            let grant_id = derive_grant_id(grant);

            self.grants_by_id.remove(&grant_id);

            self.grant_ids_by_owner
                .get_mut(&owner)
                .unwrap_or(&mut vec![])
                .retain(|id| *id != *grant_id);

            self.grant_ids_by_grantee
                .get_mut(&grantee)
                .unwrap_or(&mut vec![])
                .retain(|id| *id != *grant_id);

            self.grant_ids_by_data_id
                .get_mut(&data_id)
                .unwrap_or(&mut vec![])
                .retain(|id| *id != *grant_id);
        });

        let locked_until = locked_until.unwrap_or(0);

        FractalRegistryEvents::GrantDeleted {
            owner,
            grantee,
            data_id,
            locked_until,
        }
        .emit();
    }

    pub fn grants_for(&self, grantee: PublicKey, data_id: String) -> Vec<Grant> {
        self.find_grants(None, Some(grantee), Some(data_id))
    }

    pub fn find_grants(
        &self,
        owner: Option<PublicKey>,
        grantee: Option<PublicKey>,
        data_id: Option<String>,
    ) -> Vec<Grant> {
        let mut grant_id_searches = Vec::new();

        require!(
            owner.is_some() || grantee.is_some(),
            "Required argument: `owner` and/or `grantee`",
        );

        let empty = vec![];
        if let Some(owner) = owner {
            grant_id_searches.push(self.grant_ids_by_owner.get(&owner).unwrap_or(&empty));
        }

        if let Some(grantee) = grantee {
            grant_id_searches.push(self.grant_ids_by_grantee.get(&grantee).unwrap_or(&empty));
        }

        if let Some(data_id) = data_id {
            grant_id_searches.push(self.grant_ids_by_data_id.get(&data_id).unwrap_or(&empty));
        }

        let Some((head, tail)) = grant_id_searches.split_first() else {
            return vec![];
        };

        head.iter()
            .filter(|id| tail.iter().all(|s| s.contains(id)))
            .map(|id| self.grants_by_id.get(id).unwrap().clone())
            .collect()
    }
}
