extern crate near_sdk;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::serde::Serialize;
use near_sdk::{env, near_bindgen, require, EpochHeight, PublicKey};

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
    assert_eq!(
        "76191fb1c82d02aba62433cec21abb5a5ee93eb713736bc32abf802f46c3b17d",
        derive_grant_id(&Grant{
            owner: "ed25519:6E8sCci9badyRkXb3JoRpBj5p8C6Tw41ELDZoiihKEtp".parse().unwrap(),
            grantee: "secp256k1:qMoRgcoXai4mBPsdbHi1wfyxF9TdbPCF4qSDQTRP3TfescSRoUdSx6nmeQoN3aiwGzwMyGXAb1gUjBTv5AY8DXj".parse().unwrap(),
            data_id: "some data".into(),
            locked_until: 1337,
        })
    )
}

pub fn derive_grant_id(grant: &Grant) -> String {
    let id = format!(
        "{:?}{:?}{}{}",
        grant.owner, grant.grantee, grant.data_id, grant.locked_until,
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

fn get_push_insert<K: BorshSerialize, V: BorshDeserialize + BorshSerialize + Clone>(
    collection: &mut LookupMap<K, Vec<V>>,
    key: &K,
    value: &V,
) {
    let mut value_vec = collection.get(key).unwrap_or(vec![]);
    value_vec.push(value.clone());
    collection.insert(key, &value_vec);
}

fn remove_values<
    K: BorshSerialize,
    V: BorshDeserialize + BorshSerialize + Clone + std::cmp::PartialEq<V>,
>(
    collection: &mut LookupMap<K, Vec<V>>,
    key: &K,
    value: &V,
) {
    let mut value_vec = collection.get(key).unwrap();
    value_vec.retain(|id| *id != *value);
    collection.insert(key, &value_vec);
}

#[near_bindgen]
impl FractalRegistry {
    pub fn insert_grant(
        &mut self,
        grantee: PublicKey,
        data_id: String,
        locked_until: Option<EpochHeight>,
    ) {
        let owner = env::signer_account_pk();

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

        self.grants_by_id.insert(&grant_id, &grant);

        get_push_insert(&mut self.grant_ids_by_owner, &owner, &grant_id);
        get_push_insert(&mut self.grant_ids_by_grantee, &grantee, &grant_id);
        get_push_insert(&mut self.grant_ids_by_data_id, &data_id, &grant_id);
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

            remove_values(&mut self.grant_ids_by_owner, &owner, &grant_id);
            remove_values(&mut self.grant_ids_by_grantee, &grantee, &grant_id);
            remove_values(&mut self.grant_ids_by_data_id, &data_id, &grant_id);
        });
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

        if let Some(owner) = owner {
            grant_id_searches.push(self.grant_ids_by_owner.get(&owner).unwrap_or(vec![]));
        }

        if let Some(grantee) = grantee {
            grant_id_searches.push(self.grant_ids_by_grantee.get(&grantee).unwrap_or(vec![]));
        }

        if let Some(data_id) = data_id {
            grant_id_searches.push(self.grant_ids_by_data_id.get(&data_id).unwrap_or(vec![]));
        }

        let Some((head, tail)) = grant_id_searches.split_first() else {
            return vec![];
        };

        head.iter()
            .filter(|id| tail.iter().all(|s| s.contains(id)))
            .map(|id| self.grants_by_id.get(id).unwrap())
            .collect()
    }
}
