use near_contract_standards::fungible_token::metadata::FungibleTokenMetadata;
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::LazyOption,
    env, ext_contract,
    json_types::U128,
    log, near_bindgen, AccountId, PanicOnDefault, PromiseOrValue,
};

#[ext_contract]
pub trait ExtFungibleToken {
    fn ft_metadata(&self) -> FungibleTokenMetadata;
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    owner: AccountId,
    token_a: (AccountId, FungibleTokenMetadata),
    token_b: (AccountId, FungibleTokenMetadata),
    liquidity_pool: LazyOption<LiquidityPool>,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(owner: AccountId, token_a: AccountId, token_b: AccountId) -> PromiseOrValue<Self> {
        assert!(!env::state_exists(), "Already initialized");
        ext_fungible_token::ext(token_a.clone())
            .ft_metadata()
            .and(ext_fungible_token::ext(token_b.clone()).ft_metadata())
            .then(Self::ext(env::current_account_id()).handle_new_callback(owner, token_a, token_b))
            .into()
    }

    pub fn get_contract_info(&self) -> Option<LiquidityPool> {
        self.liquidity_pool.get()
    }

    #[private]
    pub fn handle_new_callback(
        owner: AccountId,
        token_a: AccountId,
        token_b: AccountId,
        #[callback_unwrap] token_a_metadata: FungibleTokenMetadata,
        #[callback_unwrap] token_b_metadata: FungibleTokenMetadata,
    ) -> Self {
        log!("Contract initialized with {} as owner", owner);
        Self {
            owner,
            token_a: (token_a, token_a_metadata),
            token_b: (token_b, token_b_metadata),
            liquidity_pool: LazyOption::new(StorageKey::LiquidityPool.try_to_vec().unwrap(), None),
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct LiquidityPool {
    ticker: String,
    ratio: U128,
    decimals: u8,
    token_a: U128,
    token_b: U128,
}

#[derive(BorshSerialize)]
enum StorageKey {
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
        let mut context = get_context(accounts(1));
        testing_env!(context.build());
        let contract = Contract::new(accounts(1));
        testing_env!(context.is_view(true).build());
        assert_eq!(contract.owner, accounts(1));
    }
}
