extern crate near_sdk;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::Serialize;
use near_sdk::store::LookupMap;
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
