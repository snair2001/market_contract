use crate::*;

pub(crate) fn hash_account_id(account_id: &AccountId) -> CryptoHash {
    let mut hash = CryptoHash::default();
    hash.copy_from_slice(&env::sha256(account_id.as_bytes()));
    hash
}

impl Contract {
    //internal method for removing a sale from the market. This returns the previously removed sale object
    pub(crate) fn internal_remove_sale(
        &mut self,
        nft_contract_id: AccountId,
        token_id: TokenId,
    ) -> Sale {

        //get the unique sale ID (contract + DELIMITER + token ID)
        let contract_and_token_id = format!("{}{}{}", &nft_contract_id, DELIMETER, token_id);
        //get the sale object by removing the unique sale ID. If there was no sale, panic
        let sale = self.sales.remove(&contract_and_token_id).expect("No sale");
        
        //get the set of sales for the sale's owner. If there's no sale, panic. 
        let mut by_owner_id = self.by_owner_id.get(&sale.owner_id).expect("No sale by_owner_id");
        //remove the unique sale ID from the set of sales
        by_owner_id.remove(&contract_and_token_id);
        
        //if the set of sales is now empty after removing the unique sale ID, we simply remove that owner from the map
        if by_owner_id.is_empty() {
            self.by_owner_id.remove(&sale.owner_id);
        //if the set of sales is not empty after removing, we insert the set back into the map for the owner
        } else {
            self.by_owner_id.insert(&sale.owner_id, &by_owner_id);
        }

        //get the set of token IDs for sale for the nft contract ID. If there's no sale, panic. 
        let mut by_nft_contract_id = self
            .by_nft_contract_id
            .get(&nft_contract_id)
            .expect("No sale by nft_contract_id");
        
        //remove the token ID from the set 
        by_nft_contract_id.remove(&token_id);
        
        //if the set is now empty after removing the token ID, we remove that nft contract ID from the map
        if by_nft_contract_id.is_empty() {
            self.by_nft_contract_id.remove(&nft_contract_id);
        //if the set is not empty after removing, we insert the set back into the map for the nft contract ID
        } else {
            self.by_nft_contract_id
                .insert(&nft_contract_id, &by_nft_contract_id);
        }

        //return the sale object
        sale
    }

    pub(crate) fn internal_add_market_data(
        &mut self,
        owner_id: AccountId,
        approval_id: u64,
        nft_contract_id: AccountId,
        token_id: TokenId,
        price: U128,
        start_time: Option<U64>,
        end_time: Option<U64>,
        is_auction: bool,
    ) {

        let contract_and_token_id = format!("{}{}{}", nft_contract_id, DELIMETER, token_id);

        let bids: Option<Bids> = match is_auction {
            u => {
                if u {
                    Some(Vec::new())
                } else {
                    None
                }
            }
        };

        // Time checks
        let current_time: u64 = env::block_timestamp();

        if start_time.is_some() {
            assert!(start_time.unwrap().0 >= current_time);

            if end_time.is_some() {
                assert!(start_time.unwrap().0 < end_time.unwrap().0);
            }
        }

        if end_time.is_some() {
            assert!(end_time.unwrap().0 >= current_time);
        }

        // Making sure that start time and endtime is provided if its an auction
        if is_auction{
            assert!(start_time.is_some(), "Start time is not provided.");
            assert!(end_time.is_some(), "End time is not provided.")
        }

        // Trying to put in the old price and old bids, if anyone tries to approve again.
        let mut auction_exists: bool = false;
        let mut old_price: u128=0;
        let mut old_bids: Option<Bids>=None;
        let mut old_start_time: Option<u64>=None;
        let mut old_end_time: Option<u64>=None;

        if self.sales.get(&contract_and_token_id).is_some() && is_auction{
            auction_exists=true;
            let sale=self.sales.get(&contract_and_token_id).unwrap();
            old_price=sale.price;
            old_bids=sale.bids;
            old_start_time=sale.start_time;
            old_end_time=sale.end_time;
        }

        self.sales.insert(
            &contract_and_token_id,
            &Sale {
                owner_id: owner_id.clone().into(),
                approval_id,
                nft_contract_id: nft_contract_id.clone().into(),
                token_id: token_id.clone(),
                price: match auction_exists{
                    true=>old_price,
                    false=>price.into(),
                },
                bids: match auction_exists{
                    true=>old_bids,
                    false=>bids,
                },
                start_time: match auction_exists{
                    true=>old_start_time,
                    false=> match start_time {
                                Some(x) => Some(x.0),
                                None => None,
                            }
                },
                end_time: match auction_exists{
                    true=>old_end_time,
                    false=> match end_time {
                                Some(x) => Some(x.0),
                                None => None,
                            }
                },
                is_auction: is_auction,
            },
        );
        //get the sales by owner ID for the given owner. If there are none, we create a new empty set
        let mut by_owner_id = self.by_owner_id.get(&owner_id).unwrap_or_else(|| {
            UnorderedSet::new(
                StorageKey::ByOwnerIdInner {
                    //we get a new unique prefix for the collection by hashing the owner
                    account_id_hash: hash_account_id(&owner_id),
                }
                .try_to_vec()
                .unwrap(),
            )
        });
        
        //insert the unique sale ID into the set
        by_owner_id.insert(&contract_and_token_id);
        //insert that set back into the collection for the owner
        self.by_owner_id.insert(&owner_id, &by_owner_id);

        //get the token IDs for the given nft contract ID. If there are none, we create a new empty set
        let mut by_nft_contract_id = self
            .by_nft_contract_id
            .get(&nft_contract_id)
            .unwrap_or_else(|| {
                UnorderedSet::new(
                    StorageKey::ByNFTContractIdInner {
                        //we get a new unique prefix for the collection by hashing the owner
                        account_id_hash: hash_account_id(&nft_contract_id),
                    }
                    .try_to_vec()
                    .unwrap(),
                )
            });
        
        //insert the token ID into the set
        by_nft_contract_id.insert(&token_id);
        //insert the set back into the collection for the given nft contract ID
        self.by_nft_contract_id
            .insert(&nft_contract_id, &by_nft_contract_id);
    }
}
