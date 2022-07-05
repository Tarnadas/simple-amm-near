use near_contract_standards::fungible_token::metadata::FungibleTokenMetadata;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::Promise;
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::LazyOption,
    env, ext_contract,
    json_types::U128,
    log, near_bindgen, AccountId, PanicOnDefault,
};

#[ext_contract]
pub trait ExtFungibleToken {
    fn ft_metadata(&self) -> FungibleTokenMetadata;
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct OrderlyContract {
    owner: AccountId,
    token_a: LazyOption<(AccountId, FungibleTokenMetadata)>,
    token_b: LazyOption<(AccountId, FungibleTokenMetadata)>,
    liquidity_pool: LazyOption<LiquidityPool>,
}

#[near_bindgen]
impl OrderlyContract {
    #[init]
    pub fn new(owner: AccountId) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        log!("Contract initialized with {} as owner", owner);
        Self {
            owner,
            token_a: LazyOption::new(StorageKey::TokenA.try_to_vec().unwrap(), None),
            token_b: LazyOption::new(StorageKey::TokenB.try_to_vec().unwrap(), None),
            liquidity_pool: LazyOption::new(StorageKey::LiquidityPool.try_to_vec().unwrap(), None),
        }
    }

    #[private]
    pub fn init(&mut self, token_a: AccountId, token_b: AccountId) -> Promise {
        assert!(self.token_a.get().is_none(), "Already initialized");
        ext_fungible_token::ext(token_a.clone())
            .ft_metadata()
            .and(ext_fungible_token::ext(token_b.clone()).ft_metadata())
            .then(Self::ext(env::current_account_id()).handle_init(token_a, token_b))
    }

    #[private]
    pub fn handle_init(
        &mut self,
        token_a: AccountId,
        token_b: AccountId,
        #[callback_unwrap] token_a_metadata: FungibleTokenMetadata,
        #[callback_unwrap] token_b_metadata: FungibleTokenMetadata,
    ) {
        let ticker = format!("{}-{}-LP", token_a, token_b);
        let decimals = token_a_metadata.decimals + token_b_metadata.decimals;
        self.token_a.set(&(token_a, token_a_metadata));
        self.token_b.set(&(token_b, token_b_metadata));
        self.liquidity_pool.set(&LiquidityPool {
            ticker,
            decimals,
            token_a: U128::from(0),
            token_b: U128::from(0),
        });
        log!("{:?}", self.liquidity_pool.get().unwrap());
    }

    pub fn get_contract_info(&self) -> Option<LiquidityPool> {
        log!("{:?}", self.liquidity_pool.get().unwrap());
        self.liquidity_pool.get()
    }
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Eq, PartialEq, Debug)]
pub struct LiquidityPool {
    pub ticker: String,
    pub decimals: u8,
    pub token_a: U128,
    pub token_b: U128,
}

#[derive(BorshSerialize)]
enum StorageKey {
    TokenA,
    TokenB,
    LiquidityPool,
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;

    use near_sdk::{
        test_utils::{accounts, VMContextBuilder},
        testing_env,
    };

    fn get_context(predecessor_account_id: AccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(accounts(0))
            .signer_account_id(predecessor_account_id.clone())
            .predecessor_account_id(predecessor_account_id);
        builder
    }

    #[test]
    fn test_new() {
        let context = get_context(accounts(1));
        testing_env!(context.build());
        OrderlyContract::new(accounts(1));
    }

    // TODO not sure how to stub cross contracts, so rest will be tested in integration tests
}
