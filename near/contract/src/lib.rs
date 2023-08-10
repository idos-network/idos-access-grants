use near_sdk::serde::Serialize;
use near_sdk::{env, AccountId, EpochHeight, require, near_bindgen};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct FractalRegistry {
    // Mapping of grantee and dataId to an array of grants.
    pub grants: UnorderedMap<AccountId, UnorderedMap<String, Vec<Grant>>>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Grant {
    owner: AccountId,
    grantee: AccountId,
    data_id: String,
    locked_until: EpochHeight,
}

// This implementation is terrible Rust and probably doesn't work right. ONLY
// USE IT TO TEST the interface. The grants_for method is also a dummy, always
// returning the same result regardless of input, because I couldn't make it a
// read-only method despite only using &self (not mut):
// https://docs.near.org/sdk/rust/contract-interface/contract-mutability

impl Default for FractalRegistry {
    fn default() -> Self {
        let def_acct = &AccountId::new_unchecked("jchappelow.testnet".to_string());
        let mut grants = UnorderedMap::new(b"o");
        grants.insert(
            def_acct,
            &UnorderedMap::new(b"i"),
        );
        grants
            .get(def_acct)
            .unwrap()
            .insert(
                &"blah".to_string(),
                &vec![Grant {
                    owner: def_acct.clone(),
                    grantee: def_acct.clone(),
                    data_id: "blah".to_string(),
                    locked_until: 0,
                }],
            );
        Self { grants }
    }
}

#[near_bindgen]
impl FractalRegistry {
    pub fn insert_grant(&mut self, grantee: AccountId, data_id: String) {
        let new_grant = Grant {
            owner: env::predecessor_account_id(),
            grantee: grantee.clone(),
            data_id: data_id.clone(),
            locked_until: 0,
        };
        self.grants.get(&grantee).as_mut().unwrap()
            .insert(&data_id, &vec![new_grant]);
    }

    pub fn delete_grant(&mut self, grantee: AccountId, data_id: String) {
        let binding = self.grants.get(&grantee);
        let grants = binding.unwrap().get(&data_id);
        let mut grants_for_data_id = grants.unwrap();

        require!(grants_for_data_id.len() > 0, "No grants found for this grantee and dataId");

        for (i, grant) in grants_for_data_id.iter_mut().enumerate() {
            if grant.owner == env::predecessor_account_id() {
                grants_for_data_id.swap_remove(i);
                return;
            }
        }

        panic!("Grant not found");
    }

    pub fn grants_for(grantee: AccountId, data_id: String) -> Vec<Grant> {
        vec!{
            Grant {
                owner: AccountId::new_unchecked("jchappelow.testnet".to_string()),
                grantee: grantee.clone(),
                data_id: data_id.clone(),
                locked_until: 2690839560,
            }
        }
        // self.grants
        //     .get(&grantee)
        //     .unwrap()
        //     .get(&data_id)
        //     .unwrap().clone()
    }
}

/*
 * The rest of this file holds the inline tests for the code above
 * Learn more about Rust tests: https://doc.rust-lang.org/book/ch11-01-writing-tests.html
 */
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_something() {
        let contract = FractalRegistry::default();
        assert_eq!(0, 0);
    }
}
