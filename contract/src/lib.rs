use near_contract_standards::fungible_token::{
    metadata::FungibleTokenMetadata, receiver::FungibleTokenReceiver,
};
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::LazyOption,
    env, ext_contract,
    json_types::U128,
    log, near_bindgen,
    serde::{Deserialize, Serialize},
    AccountId, PanicOnDefault, Promise, PromiseOrValue,
};

#[ext_contract]
pub trait ExtFungibleToken {
    fn ft_metadata(&self) -> FungibleTokenMetadata;
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>);
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct OrderlyContract {
    owner: AccountId,
    token_a: LazyOption<TokenPair>,
    token_b: LazyOption<TokenPair>,
}

#[derive(BorshDeserialize, BorshSerialize)]
struct TokenPair {
    pub account_id: AccountId,
    pub metadata: FungibleTokenMetadata,
    pub supply: U128,
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
        self.token_a.set(&TokenPair {
            account_id: token_a,
            metadata: token_a_metadata,
            supply: U128::from(0),
        });
        self.token_b.set(&TokenPair {
            account_id: token_b,
            metadata: token_b_metadata,
            supply: U128::from(0),
        });
    }

    pub fn get_contract_info(&self) -> Option<ContractInfo> {
        if let (Some(token_a), Some(token_b)) = (self.token_a.get(), self.token_b.get()) {
            Some(ContractInfo {
                token_a_id: token_a.account_id,
                token_a_name: token_a.metadata.name,
                token_a_symbol: token_a.metadata.symbol,
                token_a_supply: token_a.supply,
                token_a_decimals: token_a.metadata.decimals,
                token_b_id: token_b.account_id,
                token_b_name: token_b.metadata.name,
                token_b_symbol: token_b.metadata.symbol,
                token_b_supply: token_b.supply,
                token_b_decimals: token_b.metadata.decimals,
            })
        } else {
            None
        }
    }
}

#[near_bindgen]
impl FungibleTokenReceiver for OrderlyContract {
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        #[allow(unused_variables)] msg: String,
    ) -> PromiseOrValue<U128> {
        let (mut pair_a, mut pair_b) = (
            self.token_a.get().expect("Contract uninitialized"),
            self.token_b.get().expect("Contract uninitialized"),
        );

        let prod = pair_a.supply.0 * pair_b.supply.0;
        let (in_pair, in_token, out_pair, out_token) =
            if env::predecessor_account_id() == pair_a.account_id {
                (
                    &mut pair_a,
                    &mut self.token_a,
                    &mut pair_b,
                    &mut self.token_b,
                )
            } else if env::predecessor_account_id() == pair_b.account_id {
                (
                    &mut pair_b,
                    &mut self.token_b,
                    &mut pair_a,
                    &mut self.token_a,
                )
            } else {
                log!("Deposited token address does not belong to liquidity pool");
                return PromiseOrValue::Value(amount);
            };
        in_pair.supply.0 += amount.0;
        in_token.set(in_pair);
        if sender_id != self.owner {
            let out_pair_supply = out_pair.supply.0;
            // this will truncate the remainder, thus resulting in a loss of lp token.
            // in a real world solution, this would need to be addressed.
            out_pair.supply.0 = prod / in_pair.supply.0;
            let out_pair_diff = out_pair_supply - out_pair.supply.0;
            log!(
                "User {} swapping {} of token {} for {} of token {}",
                &sender_id,
                amount.0,
                in_pair.account_id,
                out_pair_diff,
                out_pair.account_id
            );
            out_token.set(out_pair);

            ext_fungible_token::ext(out_pair.account_id.clone())
                .with_attached_deposit(1)
                .with_static_gas(10_000_000_000_000.into())
                .ft_transfer(sender_id, out_pair_diff.into(), Some("swap".to_string())); // .then(Self::ext(env::current_account_id()).handle_swap(token_a, token_b)),
            PromiseOrValue::Value(0.into())
        } else {
            PromiseOrValue::Value(0.into())
        }
    }
}

#[derive(Deserialize, Serialize, Eq, PartialEq, Debug)]
pub struct ContractInfo {
    pub token_a_id: AccountId,
    pub token_a_name: String,
    pub token_a_symbol: String,
    pub token_a_supply: U128,
    pub token_a_decimals: u8,
    pub token_b_id: AccountId,
    pub token_b_name: String,
    pub token_b_symbol: String,
    pub token_b_supply: U128,
    pub token_b_decimals: u8,
}

#[derive(BorshSerialize)]
enum StorageKey {
    TokenA,
    TokenB,
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
}
