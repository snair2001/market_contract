#------------To dev-deploy--------------

near dev-deploy ./target/wasm32-unknown-unknown/release/nft_simple.wasm


#------------To deploy to an account---- 

# near create-account test.evin.testnet --masterAccount evin.testnet  --initialBalance 50
# near deploy --wasmFile ./target/wasm32-unknown-unknown/release/nft_simple.wasm --accountId test.evin.testnet