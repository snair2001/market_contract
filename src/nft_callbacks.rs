use crate::*;

/// approval callbacks from NFT Contracts

//struct for keeping track of the sale conditions for a Sale
#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct SaleArgs {
    pub price: U128, // Sale price is in yoctonear
    pub is_auction: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<U64>, //Unix timestamp for when auction starts
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<U64>, //Unix timestamp for when auction finishes
}

/*
    trait that will be used as the callback from the NFT contract. When nft_approve is
    called, it will fire a cross contract call to this marketplace and this is the function
    that is invoked. 
*/
trait NonFungibleTokenApprovalsReceiver {
    fn nft_on_approve(
        &mut self,
        token_id: TokenId,
        owner_id: AccountId,
        approval_id: u64,
        msg: String,
    );
}

//implementation of the trait
#[near_bindgen]
impl NonFungibleTokenApprovalsReceiver for Contract {
    /// where we add the sale because we know nft owner can only call nft_approve

    fn nft_on_approve(
        &mut self,
        token_id: TokenId,
        owner_id: AccountId,
        approval_id: u64,
        msg: String,
    ) {
        let nft_contract_id = env::predecessor_account_id();
        let signer_id = env::signer_account_id();

        //make sure that the signer isn't the predecessor. This is so that we're sure
        //this was called via a cross-contract call
        assert_ne!(
            nft_contract_id,
            signer_id,
            "nft_on_approve should only be called via cross-contract call"
        );
        //make sure the owner ID is the signer. 
        assert_eq!(
            owner_id,
            signer_id,
            "owner_id should be signer_id"
        );

        //we need to enforce that the user has enough storage for 1 EXTRA sale.  

        //get the storage for a sale. dot 0 converts from U128 to u128
        let storage_amount = self.storage_minimum_balance().0;
        //get the total storage paid by the owner
        let owner_paid_storage = self.storage_deposits.get(&signer_id).unwrap_or(0);
        //get the storage required which is simply the storage for the number of sales they have + 1 
        let signer_storage_required = (self.get_supply_by_owner_id(signer_id).0 + 1) as u128 * storage_amount;
        
        //make sure that the total paid is >= the required storage
        assert!(
            owner_paid_storage >= signer_storage_required,
            "Insufficient storage paid: {}, for {} sales at {} rate of per sale",
            owner_paid_storage, signer_storage_required / STORAGE_PER_SALE, STORAGE_PER_SALE
        );

        let SaleArgs {
            price,
            is_auction,
            start_time,
            end_time
        } = near_sdk::serde_json::from_str(&msg).expect("Not valid MarketArgs");
        
        self.internal_add_market_data(
            owner_id,
            approval_id,
            nft_contract_id,
            token_id,
            price,
            start_time,
            end_time,
            is_auction,
        );
        //Extra functionality that populates collections necessary for the view calls 
    }
}
