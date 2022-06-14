This set of files implement the NEAR smart contracts required for buying, selling and auctioning of an NFT. 
This repository has to be cloned in to a workspace and the below command has to be run
RUSTFLAGS='-C link-arg=-s' cargo build --target wasm32-unknown-unknown --release
to generate the ./target/wasm32-unknown-unknown/release/nft_simple.wasm file. 
This wasm file has to be used in the command below to deploy the smart contracts. 
near deploy --accountId xyz.near --wasmFile ./target/wasm32-unknown-unknown/release/nft_simple.wasm
Subsequently the contract has to be initialized with the command below.
near call abc.near new '{"owner_id": "def.near", "charges_id": "ghi.near", "commissions_id": "jkl.near", "charges":800, "commissions": 100}' --accountId mno.near
