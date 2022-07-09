# Simple AMM

This is a simple Automated Market Maker (AMM) Smart Contract that supports swapping two tokens.
The owner of the Smart Contract can add liquidity via sending the appropriate token.
All other users can then swap via 

## Building

- Install Rust via [Rustup](https://rustup.rs/)
- Add WebAssembly tookchain: `rustup target add wasm32-unknown-unknown`
- Compile this Smart Contract: `./build.sh`

## Deployment

Make sure to have [Near CLI](https://github.com/near/near-cli) installed: `npm install -g near-cli`.
We will create a subaccount where we deploy the contract.

```bash
# setup
NEAR_ENV=testnet # change this to mainnet for prod
MASTER_ACCOUNT=
CONTRACT_ID=amm.$MASTER_ACCOUNT
# the contract owner will be the only one who can add liquidity
OWNER_ID=

# Login
near login

# create subaccount with 10 Near initially
near create-account $CONTRACT_ID --masterAccount $MASTER_ACCOUNT --initialBalance 10

# deploy contract
near deploy --wasmFile res/orderly_contract.wasm --accountId $CONTRACT_ID
```

## Test tokens

There are plenty of test tokens on [Ref Finance](https://testnet.ref.finance/), that you can use.
If you want to you can also deploy your own test tokens, which are also used in integration tests.
We can also freely mint these tokens, which makes testing easy.

```bash
TOKEN_ID1=token-a.$MASTER_ACCOUNT
TOKEN_ID2=token-b.$MASTER_ACCOUNT

near create-account $TOKEN_ID1 --masterAccount $MASTER_ACCOUNT --initialBalance 2
near deploy --wasmFile res/test_token.wasm --accountId $TOKEN_ID1
near call $TOKEN_ID1 new '{ "name": "TokenA", "symbol": "TKNA" }' --accountId $TOKEN_ID1
near call $TOKEN_ID1 mint '{ "account_id": "'$OWNER_ID'", "amount": "1000000" }' --accountId $TOKEN_ID1

near create-account $TOKEN_ID2 --masterAccount $MASTER_ACCOUNT --initialBalance 2
near deploy --wasmFile res/test_token.wasm --accountId $TOKEN_ID2
near call $TOKEN_ID2 new '{ "name": "TokenB", "symbol": "TKNB" }' --accountId $TOKEN_ID2
near call $TOKEN_ID2 mint '{ "account_id": "'$OWNER_ID'", "amount": "1000000" }' --accountId $TOKEN_ID2
```

## Initialization

```bash
# we now initialize amm contract
near call $CONTRACT_ID new '{ "owner": "'$OWNER_ID'" }' --accountId $CONTRACT_ID

# we also need to setup the two tokens, that will be used in the liquidity pool for swapping
near call $CONTRACT_ID init '{ "token_a": "'$TOKEN_ID1'", "token_b": "'$TOKEN_ID2'" }' --accountId $CONTRACT_ID

# and register contract for these tokens
near call $TOKEN_ID1 storage_deposit '{ "account_id": "'$CONTRACT_ID'" }' --accountId $CONTRACT_ID --deposit 1
near call $TOKEN_ID2 storage_deposit '{ "account_id": "'$CONTRACT_ID'" }' --accountId $CONTRACT_ID --deposit 1

# owner now needs to add liquidity for both tokens
near call $TOKEN_ID1 ft_transfer_call '{ "receiver_id": "'$CONTRACT_ID'", "amount": "1000000", "msg": "" }' --accountId $OWNER_ID --depositYocto 1 --gas 300000000000000
near call $TOKEN_ID2 ft_transfer_call '{ "receiver_id": "'$CONTRACT_ID'", "amount": "1000000", "msg": "" }' --accountId $OWNER_ID --depositYocto 1 --gas 300000000000000
```

## Testing

The contract has various integration tests for testing the cross contract interactions.
These can be executed via `cargo test`.

Since we now set up everything, we can also do manual testing of swap:

```bash
# let's do a quick check, if the contract set up the two tokens
near view $CONTRACT_ID get_contract_info
# it should return metadata about the contract and the tokens with accountId, name, supply, symbol, decimals

# setup swap user
TEST_USER=user.$MASTER_ACCOUNT
near create-account $TEST_USER --masterAccount $MASTER_ACCOUNT --initialBalance 2
near call $TOKEN_ID1 mint '{ "account_id": "'$TEST_USER'", "amount": "1000000" }' --accountId $TOKEN_ID1
near call $TOKEN_ID1 mint '{ "account_id": "'$TEST_USER'", "amount": "1000000" }' --accountId $TOKEN_ID2

# swap token-a for token-b
near call $TOKEN_ID1 ft_transfer_call '{ "receiver_id": "'$CONTRACT_ID'", "amount": "1000", "msg": "" }' --accountId $TEST_USER --depositYocto 1 --gas 300000000000000

# check token balance
near view $TOKEN_ID1 ft_balance_of '{ "account_id": "'$TEST_USER'" }'
near view $TOKEN_ID2 ft_balance_of '{ "account_id": "'$TEST_USER'" }'
```
