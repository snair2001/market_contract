use crate::*;
use near_sdk::promise_result_as_success;
use near_sdk::log;

const MIN_BID_INCREMENT : u128 = 10_000_000_000_000_000_000_000; // 0.01 N

//struct that holds important information about each sale on the market
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Sale {
    pub owner_id: AccountId,
    pub approval_id: u64,
    pub nft_contract_id: String,
    pub token_id: String,
    pub price: u128,
    pub bids: Option<Bids>,
    pub is_auction: bool,
    pub start_time: Option<u64>, //Unix timestamp for when auction starts
    pub end_time: Option<u64>, //Unix timestamp for when auction finishes
}

#[near_bindgen]
impl Contract {
    
    //removes a sale from the market. 
    #[payable]
    pub fn remove_sale(&mut self, nft_contract_id: AccountId, token_id: String) {
        //assert that the user has attached exactly 1 yoctoNEAR (for security reasons)
        assert_one_yocto();
        //get the sale object as the return value from removing the sale internally
        let sale = self.internal_remove_sale(nft_contract_id.into(), token_id);
        //get the predecessor of the call and make sure they're either sale owner or smart contract owner
        let caller_id = env::predecessor_account_id();
        //if this fails, the remove sale will revert
        if caller_id!=sale.owner_id && caller_id!=self.owner_id{
            env::panic_str("Must be either sale owner or owner of smart contract!");
        }

        let current_time: u64 = env::block_timestamp();

        /* For auction removal: 
          1.smart contract owner can remove the auction any time, no constraints. (will only be exercised in case of tokens where marketplace is not approved anymore)
          2.token owner can remove it any time if it has no bids else no removal allowed after end_time if there are bids.
        */
        if sale.is_auction && sale.bids.is_some(){
            
            let bids= sale.bids.unwrap();
            
            if bids.len()>0{
                let end_time=sale.end_time;
                
                if  caller_id==sale.owner_id {
                    assert!(current_time < end_time.unwrap(), "Cannot remove auction now since the end_time has been crossed. Consider ending the auction instead.");
                }

                let current_bid = &bids[bids.len() - 1];
                // refund
                Promise::new(current_bid.bidder_id.clone()).transfer(current_bid.price.0);
            }
        } 
    }

    //updates the price for a sale on the market
    #[payable]
    pub fn update_price(
        &mut self,
        nft_contract_id: AccountId,
        token_id: String,
        amount: U128,
    ) {
        //assert that the user has attached exactly 1 yoctoNEAR (for security reasons)
        assert_one_yocto();
        
        //create the unique sale ID from the nft contract and token
        let contract_id: AccountId = nft_contract_id.into();
        let contract_and_token_id = format!("{}{}{}", contract_id, DELIMETER, token_id);
        
        //get the sale object from the unique sale ID. If there is no token, panic. 
        let mut sale = self.sales.get(&contract_and_token_id).expect("No sale");

        if sale.is_auction{
            env::panic_str("Sorry, cannot update an auction");
        }

        //assert that the caller of the function is the sale owner
        assert_eq!(
            env::predecessor_account_id(),
            sale.owner_id,
            "Must be sale owner"
        );
        
        //set the price equal to the passed in amount
        sale.price = amount.into();
        //insert the sale back into the map for the unique sale ID
        self.sales.insert(&contract_and_token_id, &sale);
    }

    #[payable]
    pub fn add_bid(&mut self, nft_contract_id: AccountId, token_id: String){
        let deposit = env::attached_deposit();
        assert!(deposit > 0, "Attached deposit must be greater than 0");

        let contract_id: AccountId = nft_contract_id.into();
        let contract_and_token_id = format!("{}{}{}", contract_id, DELIMETER, token_id);
        let mut sale = self.sales.get(&contract_and_token_id).expect("Doesn't exist");

        if sale.is_auction {
            let current_time: u64 = env::block_timestamp();
            let start_time=sale.start_time;
            let end_time=sale.end_time;

            assert!( start_time.unwrap() < current_time, "Cannot bid before auction starts");
            assert!( current_time < end_time.unwrap() ,"Cannot bid since auction is over" );
        }
        else{
            env::panic_str("Sale should be an auction");
        }

        let bidder_id = env::predecessor_account_id();
        assert_ne!(sale.owner_id, bidder_id, "Cannot bid on your own sale.");

        let new_bid = Bid {
            bidder_id: bidder_id.clone(),
            price: U128(deposit),
        };

        let mut bids = sale.bids.unwrap_or(Vec::new());

        if !bids.is_empty() {
            let current_bid = &bids[bids.len() - 1];

            assert!(
                deposit >= (current_bid.price.0 + MIN_BID_INCREMENT),
                "Can't pay less than or equal to current bid price + increment (0.01 N) : {:?}",
                current_bid.price
            );

            assert!(
                deposit > sale.price,
                "Can't pay less than or equal to starting price: {:?}",
                U128(sale.price)
            );

            // refund
            Promise::new(current_bid.bidder_id.clone()).transfer(current_bid.price.0);

            // always keep 1 bid for now
            bids.remove(bids.len() - 1);
        } else {
            assert!(
                deposit >= (sale.price + MIN_BID_INCREMENT),
                "Can't pay less than or equal to starting price + increment (0.01 N): {}",
                sale.price
            );
        }

        bids.push(new_bid);
        sale.bids = Some(bids);
        self.sales.insert(&contract_and_token_id, &sale);
    }

    #[payable]
    pub fn end_auction(&mut self, nft_contract_id: AccountId, token_id: String){
        let contract_id: AccountId = nft_contract_id.into();
        let contract_and_token_id = format!("{}{}{}", contract_id, DELIMETER, token_id);
        let sale = self.sales.get(&contract_and_token_id).expect("Doesn't exist");

        if sale.is_auction {
            let current_time: u64 = env::block_timestamp();
            let end_time=sale.end_time;

            assert!( Some(current_time) > end_time, "Cannot end before end_time mentioned for the auction");
        }
        else{
            env::panic_str("Sale should be an auction");
        }

        let bids = sale.bids.unwrap_or(Vec::new());

        if !bids.is_empty() {
            let current_bid = &bids[bids.len() - 1];
            let buyer_id= current_bid.bidder_id.clone();
            self.process_purchase(
                contract_id,
                token_id,
                current_bid.price,
                buyer_id,
            );
        }
        else{
            self.internal_remove_sale(contract_id, token_id);
        }
    }

    #[payable]
    pub fn offer(&mut self, nft_contract_id: AccountId, token_id: String) {
        //get the attached deposit and make sure it's greater than 0
        let deposit = env::attached_deposit();
        assert!(deposit > 0, "Attached deposit must be greater than 0");

        //convert the nft_contract_id from a AccountId to an AccountId
        let contract_id: AccountId = nft_contract_id.into();
        //get the unique sale ID (contract + DELIMITER + token ID)
        let contract_and_token_id = format!("{}{}{}", contract_id, DELIMETER, token_id);
        
        //get the sale object from the unique sale ID. If the sale doesn't exist, panic.
        let sale = self.sales.get(&contract_and_token_id).expect("No sale");
        
        //get the buyer ID which is the person who called the function and make sure they're not the owner of the sale
        let buyer_id = env::predecessor_account_id();
        assert_ne!(sale.owner_id, buyer_id, "Cannot bid on your own sale.");
        
        assert!(sale.is_auction==false, "Please use add_bid function to bid on this auction item!");

        let price = sale.price;

        //make sure the deposit is greater than the price
        assert!(deposit >= price, "Attached deposit must be greater than or equal to the current price: {:?}", price);

        //process the purchase (which will remove the sale, transfer and get the payout from the nft contract, and then distribute royalties) 
        self.process_purchase(
            contract_id,
            token_id,
            U128(deposit),
            buyer_id,
        );
    }

    //private function used when a sale is purchased. 
    //this will remove the sale, transfer and get the payout from the nft contract, and then distribute royalties
    #[private]
    pub fn process_purchase(
        &mut self,
        nft_contract_id: AccountId,
        token_id: String,
        price: U128,
        buyer_id: AccountId,
    ) -> Promise {
        //get the sale object by removing the sale
        let sale = self.internal_remove_sale(nft_contract_id.clone(), token_id.clone());

        //initiate a cross contract call to the nft contract. This will transfer the token to the buyer and return
        //a payout object used for the market to distribute funds to the appropriate accounts.
        ext_contract::nft_transfer_payout(
            buyer_id.clone(), //purchaser (person to transfer the NFT to)
            token_id, //token ID to transfer
            sale.approval_id, //market contract's approval ID in order to transfer the token on behalf of the owner
            "payout from market".to_string(), //memo (to include some context)
            /*
                the price that the token was purchased for. This will be used in conjunction with the royalty percentages
                for the token in order to determine how much money should go to which account. 
            */
            price,
			10, //the maximum amount of accounts the market can payout at once (this is limited by GAS)
            nft_contract_id, //contract to initiate the cross contract call to
            1, //yoctoNEAR to attach to the call
            GAS_FOR_NFT_TRANSFER, //GAS to attach to the call
        )
        //after the transfer payout has been initiated, we resolve the promise by calling our own resolve_purchase function. 
        //resolve purchase will take the payout object returned from the nft_transfer_payout and actually pay the accounts
        .then(ext_self::resolve_purchase(
            buyer_id, //the buyer and price are passed in incase something goes wrong and we need to refund the buyer
            price,
            sale,
            env::current_account_id(), //we are invoking this function on the current contract
            NO_DEPOSIT, //don't attach any deposit
            GAS_FOR_ROYALTIES, //GAS attached to the call to payout royalties
        ))
    }

    /*
        private method used to resolve the promise when calling nft_transfer_payout. This will take the payout object and 
        check to see if it's authentic and there's no problems. If everything is fine, it will pay the accounts. If there's a problem,
        it will refund the buyer for the price. 
    */
    #[private]
    pub fn resolve_purchase(
        &mut self,
        buyer_id: AccountId,
        price: U128,
        sale: Sale,
    ) -> U128 {
        // checking for payout information returned from the nft_transfer_payout method
        let payout_option = promise_result_as_success().and_then(|value| {
            //if we set the payout_option to None, that means something went wrong and we should refund the buyer
            near_sdk::serde_json::from_slice::<Payout>(&value)
                //converts the result to an optional value
                .ok()
                //returns None if the none. Otherwise executes the following logic
                .and_then(|payout_object| {
                    //we'll check if length of the payout object is > 10 or it's empty. In either case, we return None
                    if payout_object.payout.len() > 10 || payout_object.payout.is_empty() {
                        env::log_str("Cannot have more than 10 royalties");
                        None
                    
                    //if the payout object is the correct length, we move forward
                    } else {
                        //we'll keep track of how much the nft contract wants us to payout. Starting at the full price payed by the buyer
                        let mut remainder = price.0;
                        
                        //loop through the payout and subtract the values from the remainder. 
                        for &value in payout_object.payout.values() {
                            //checked sub checks for overflow or any errors and returns None if there are problems
                            remainder = remainder.checked_sub(value.0)?;
                        }
                        //Check to see if the NFT contract sent back a faulty payout that requires us to pay more or too little. 
                        //The remainder will be 0 if the payout summed to the total price. The remainder will be 1 if the royalties
                        //we something like 3333 + 3333 + 3333. 
                        if remainder == 0 || remainder == 1 {
                            //set the payout_option to be the payout because nothing went wrong
                            Some(payout_object.payout)
                        } else {
                            //if the remainder was anything but 1 or 0, we return None
                            None
                        }
                    }
                })
        });

        // if the payout option was some payout, we set this payout variable equal to that some payout
        let payout = if let Some(payout_option) = payout_option {
            payout_option
        //if the payout option was None, we refund the buyer for the price they payed and return
        } else {
            Promise::new(buyer_id).transfer(u128::from(price));
            // leave function and return the price that was refunded
            return price;
        };

        let fee_percentage = 500u128;
        let fee = price.0 * fee_percentage / 10_000u128;

        log!("Fees that should be going is: {}", fee);
        // NEAR payouts
        for (receiver_id, amount) in payout {
            if receiver_id == sale.owner_id {
                Promise::new(receiver_id).transfer(amount.0 - fee);
                if fee != 0 {
                    Promise::new(self.treasury_id.clone()).transfer(fee);
                }
            } 
            else {
                Promise::new(receiver_id).transfer(amount.0);
            }
        }

        //return the price payout out
        price
    }
}

//this is the cross contract call that we call on our own contract. 
/*
    private method used to resolve the promise when calling nft_transfer_payout. This will take the payout object and 
    check to see if it's authentic and there's no problems. If everything is fine, it will pay the accounts. If there's a problem,
    it will refund the buyer for the price. 
*/
#[ext_contract(ext_self)]
trait ExtSelf {
    fn resolve_purchase(
        &mut self,
        buyer_id: AccountId,
        price: U128,
        sale : Sale,
    ) -> Promise;
}
