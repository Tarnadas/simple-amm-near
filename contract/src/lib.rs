use near_contract_standards::fungible_token::metadata::FungibleTokenMetadata;
use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_sdk::collections::LookupMap;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::LazyOption,
    env, ext_contract,
    json_types::U128,
    log, near_bindgen, AccountId, PanicOnDefault,
};
use near_sdk::{Balance, Promise, PromiseOrValue};

#[ext_contract]
pub trait ExtFungibleToken {
    fn ft_metadata(&self) -> FungibleTokenMetadata;
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct OrderlyContract {
    owner: AccountId,
    token_a: LazyOption<TokenPair>,
    token_a_accounts: LookupMap<AccountId, Balance>,
    token_b: LazyOption<TokenPair>,
    token_b_accounts: LookupMap<AccountId, Balance>,
    liquidity_pool: LazyOption<LiquidityPool>,
}

#[derive(BorshDeserialize, BorshSerialize)]
struct TokenPair {
    pub account_id: AccountId,
    pub metadata: FungibleTokenMetadata,
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
            token_a_accounts: LookupMap::new(StorageKey::TokenAAccounts.try_to_vec().unwrap()),
            token_b: LazyOption::new(StorageKey::TokenB.try_to_vec().unwrap(), None),
            token_b_accounts: LookupMap::new(StorageKey::TokenBAccounts.try_to_vec().unwrap()),
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
        self.token_a.set(&TokenPair {
            account_id: token_a,
            metadata: token_a_metadata,
        });
        self.token_b.set(&TokenPair {
            account_id: token_b,
            metadata: token_b_metadata,
        });
        self.liquidity_pool.set(&LiquidityPool {
            ticker,
            decimals,
            token_a_supply: U128::from(0),
            token_b_supply: U128::from(0),
        });
        log!("{:?}", self.liquidity_pool.get().unwrap());
    }

    pub fn get_contract_info(&self) -> Option<LiquidityPool> {
        log!("{:?}", self.liquidity_pool.get().unwrap());
        self.liquidity_pool.get()
    }
}

impl FungibleTokenReceiver for OrderlyContract {
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        _msg: String,
    ) -> PromiseOrValue<U128> {
        log!("{} {}", &sender_id, &amount.0);
        let (pair_a, pair_b, mut lp) = (
            self.token_a.get().expect("Contract uninitialized"),
            self.token_b.get().expect("Contract uninitialized"),
            self.liquidity_pool.get().expect("Contract uninitialized"),
        );

        let (accounts, lp_token) = if env::predecessor_account_id() == pair_a.account_id {
            (&mut self.token_a_accounts, &mut lp.token_a_supply)
        } else if env::predecessor_account_id() == pair_b.account_id {
            (&mut self.token_b_accounts, &mut lp.token_b_supply)
        } else {
            log!("Deposited token address does not belong to liquidity pool");
            return PromiseOrValue::Value(amount);
        };
        lp_token.0.checked_add(amount.0).unwrap();
        if sender_id != self.owner {
            let mut signer_balance = accounts.get(&sender_id).unwrap_or_default();
            signer_balance += amount.0;
            accounts.insert(&sender_id, &signer_balance);
        }
        PromiseOrValue::Value(0.into())
    }
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Eq, PartialEq, Debug)]
pub struct LiquidityPool {
    pub ticker: String,
    pub decimals: u8,
    pub token_a_supply: U128,
    pub token_b_supply: U128,
}

#[derive(BorshSerialize)]
enum StorageKey {
    TokenA,
    TokenB,
    TokenAAccounts,
    TokenBAccounts,
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
