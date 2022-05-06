use crate::*;
use crate::internal::hash_account_id;

const STORAGE_PRICE_CONTRACT_ID:u128 = 10 * STORAGE_PRICE_PER_BYTE;

#[near_bindgen]
impl Contract{

	#[payable]
	pub fn add_contract_for_account(&mut self, nft_contract_id: AccountId){
        // Need to deposit 0.001 N for storing these values 
        assert!(env::attached_deposit() >= STORAGE_PRICE_CONTRACT_ID, "Requires minimum deposit of {}", STORAGE_PRICE_CONTRACT_ID );

		let account_id = env::predecessor_account_id();

		let mut contract_ids = self.contract_ids_by_account_id.get(&account_id).unwrap_or_else(|| {
            UnorderedSet::new(
                StorageKey::ContractIdsInner {
                    //we get a new unique prefix for the collection by hashing the owner
                    account_id_hash: hash_account_id(&account_id),
                }
                .try_to_vec()
                .unwrap(),
            )
        });

        contract_ids.insert(&nft_contract_id);

		self.contract_ids_by_account_id.insert(&account_id, &contract_ids);
	}

	#[payable]
	pub fn remove_contract_for_account(&mut self, nft_contract_id: AccountId){
		assert_one_yocto();

		let account_id = env::predecessor_account_id();

        // Finding the collection of contract ids for the account
		let mut contract_ids = self.contract_ids_by_account_id.get(&account_id).expect("Couldn't find account");

        // Finding the contract id
        assert_eq!(contract_ids.contains(&nft_contract_id), true, "Couldn't find the contract being removed");
        
        contract_ids.remove(&nft_contract_id);
        
        if contract_ids.is_empty() {
            self.contract_ids_by_account_id.remove(&account_id);
        } 
        else {
            self.contract_ids_by_account_id.insert(&account_id, &contract_ids);
        }
	}

	pub fn get_contract_ids_for_account(&self, account_id:AccountId) -> Vec<AccountId>{

		let contract_ids = self.contract_ids_by_account_id.get(&account_id);
        
        let contracts = if let Some(contract_ids) = contract_ids {
            contract_ids
        } else {
            return vec![];
        };

        let keys=contracts.as_vector();
        keys.iter().collect()
	}
}

