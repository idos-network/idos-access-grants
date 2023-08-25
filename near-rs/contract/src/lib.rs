use near_sdk::serde::Serialize;
use near_sdk::{env, AccountId, EpochHeight, near_bindgen, require};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use hex;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct FractalRegistry {
    pub grants_by_id: LookupMap<String, Grant>,

    pub grant_ids_by_owner: LookupMap<AccountId, Vec<String>>,
    pub grant_ids_by_grantee: LookupMap<AccountId, Vec<String>>,
    pub grant_ids_by_data_id: LookupMap<String, Vec<String>>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Grant {
    owner: AccountId,
    grantee: AccountId,
    data_id: String,
    locked_until: EpochHeight,
}

pub fn derive_grant_id(grant: &Grant) -> String {
    let id = format!(
        "{}{}{}{}",
        grant.owner,
        grant.grantee,
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

#[near_bindgen]
impl FractalRegistry {
    pub fn insert_grant(
        &mut self,
        grantee: AccountId,
        data_id: String,
        locked_until: Option<u64>
    ) {
        let owner = env::predecessor_account_id();

        let grant = Grant {
            owner: owner.clone(),
            grantee: grantee.clone(),
            data_id: data_id.clone(),
            locked_until: locked_until.unwrap_or(0),
        };

        let grant_id = derive_grant_id(&grant);

        require!(!self.grants_by_id.contains_key(&grant_id), "Grant already exists");

        self.grants_by_id.insert(&grant_id, &grant);

        let mut grant_ids_owner = self.grant_ids_by_owner.get(&owner.clone()).unwrap_or(vec!{});
        grant_ids_owner.push(grant_id.clone());
        self.grant_ids_by_owner.insert(&owner.clone(), &grant_ids_owner);

        let mut grant_ids_grantee = self.grant_ids_by_grantee.get(&grantee.clone()).unwrap_or(vec!{});
        grant_ids_grantee.push(grant_id.clone());
        self.grant_ids_by_grantee.insert(&grantee.clone(), &grant_ids_grantee);

        let mut grant_ids_data_id = self.grant_ids_by_data_id.get(&data_id.clone()).unwrap_or(vec!{});
        grant_ids_data_id.push(grant_id.clone());
        self.grant_ids_by_data_id.insert(&data_id.clone(), &grant_ids_data_id);
    }

    pub fn delete_grant(
        &mut self,
        grantee: AccountId,
        data_id: String,
        locked_until: Option<u64>
    ) {
        let owner = env::predecessor_account_id();

        self
            .find_grants(Some(owner.clone()), Some(grantee.clone()), Some(data_id.clone()))
            .iter()
            .filter(|grant| [0, grant.locked_until].contains(&locked_until.unwrap_or(0)))
            .for_each(|grant| {
                require!(grant.locked_until < env::block_timestamp(), "Grant is timelocked");

                let grant_id = derive_grant_id(&grant);

                self.grants_by_id.remove(&grant_id);

                let mut grant_ids_owner = self.grant_ids_by_owner.get(&owner.clone()).unwrap();
                grant_ids_owner.retain(|id| *id != grant_id);
                self.grant_ids_by_owner.insert(&owner.clone(), &grant_ids_owner);

                let mut grant_ids_grantee = self.grant_ids_by_grantee.get(&grantee.clone()).unwrap();
                grant_ids_grantee.retain(|id| *id != grant_id);
                self.grant_ids_by_grantee.insert(&grantee.clone(), &grant_ids_grantee);

                let mut grant_ids_data_id = self.grant_ids_by_data_id.get(&data_id.clone()).unwrap();
                grant_ids_data_id.retain(|id| *id != grant_id);
                self.grant_ids_by_data_id.insert(&data_id.clone(), &grant_ids_data_id);
            });
    }

    pub fn grants_for(
        &mut self,
        grantee: AccountId,
        data_id: String
    ) -> Vec<Grant> {
        self.find_grants(None, Some(grantee), Some(data_id))
    }

    pub fn find_grants(
        &mut self,
        owner: Option<AccountId>,
        grantee: Option<AccountId>,
        data_id: Option<String>
    ) -> Vec<Grant> {
        let mut grant_id_searches = Vec::new();

        if let Some(owner) = owner {
            grant_id_searches.push(self.grant_ids_by_owner.get(&owner.clone()).unwrap_or(vec!{}));
        }

        if let Some(grantee) = grantee {
            grant_id_searches.push(self.grant_ids_by_grantee.get(&grantee.clone()).unwrap_or(vec!{}));
        }

        if let Some(data_id) = data_id {
            grant_id_searches.push(self.grant_ids_by_data_id.get(&data_id.clone()).unwrap_or(vec!{}));
        }

        grant_id_searches[0]
            .iter()
            .filter(|id| grant_id_searches.iter().all(|s| s.contains(id)))
            .map(|id| self.grants_by_id.get(&id).unwrap())
            .collect()
    }
}
