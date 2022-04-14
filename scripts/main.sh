# Add to market with nft approve and try buying with another function

#-------------Uncomment these lines to check if a normal sale goes through-------------

near call $c storage_deposit --deposit 0.5 --accountId alice.evin.testnet
near call royalties.evin.testnet nft_approve '{"token_id":"'$token_id'", "account_id":"'$c'", "msg":"{\"price\":\"2000000000000000000000000\",\"is_auction\":false}"}' --accountId alice.evin.testnet --deposit 1
# near call $c remove_sale '{"nft_contract_id":"royalties.evin.testnet", "token_id": "'$token_id'"}' --accountId alice.evin.testnet --depositYocto 1

# near call $c offer '{"nft_contract_id": "royalties.evin.testnet", "token_id":"VeryNewToken10"}' --accountId bob.evin.testnet --deposit 2 --gas 300000000000000

#-------------Uncomment these lines to check if an auction sale goes through-------------

# near call $c storage_deposit --deposit 0.5 --accountId alice.evin.testnet
# near call $c storage_deposit --deposit 0.5 --accountId bob.evin.testnet

# near call royalties.evin.testnet nft_mint '{"token_id": "VeryNewToken1", "metadata": {"title": "Testing auctions part 2", "description": "testing out auction bidding and ending I just wrote", "media": "https://images.unsplash.com/photo-1432457990754-c8b5f21448de?ixlib=rb-1.2.1&ixid=MnwxMjA3fDB8MHx0b3BpYy1mZWVkfDIxfGhTUDZKeDh3NFo0fHxlbnwwfHx8fA%3D%3D&auto=format&fit=crop&w=500&q=60"}, "receiver_id": "alice.evin.testnet"}' --accountId $c --amount 0.1
# near call royalties.evin.testnet nft_approve '{"token_id":"VeryNewToken1", "account_id":"'$c'", "msg":"{\"price\":\"2\",\"is_auction\":true,\"start_time\":\"'$start_time'\",\"end_time\":\"'$end_time'\"}"}' --accountId alice.evin.testnet --deposit 1

# near call royalties.evin.testnet nft_mint '{"token_id": "VeryNewToken2", "metadata": {"title": "Testing auctions part 2", "description": "testing out auction bidding and ending I just wrote", "media": "https://images.unsplash.com/photo-1648514741567-b2d28e0700b8?ixlib=rb-1.2.1&ixid=MnwxMjA3fDB8MHx0b3BpYy1mZWVkfDI0fGhTUDZKeDh3NFo0fHxlbnwwfHx8fA%3D%3D&auto=format&fit=crop&w=500&q=60"}, "receiver_id": "bob.evin.testnet"}' --accountId $c --amount 0.1
# near call royalties.evin.testnet nft_approve '{"token_id":"VeryNewToken2", "account_id":"'$c'", "msg":"{\"price\":\"2\",\"is_auction\":true,\"start_time\":\"'$start_time'\",\"end_time\":\"'$end_time'\"}"}' --accountId bob.evin.testnet --deposit 1


