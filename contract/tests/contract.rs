use near_sdk::json_types::U128;
use orderly_contract::LiquidityPool;
use tokio::fs;
use workspaces::{network::Sandbox, prelude::*, Account, AccountId, Contract, Worker};

#[tokio::test]
async fn test_init() -> anyhow::Result<()> {
    let (worker, _, contract, token_a, token_b) = initialize_contracts().await?;

    contract_init(&worker, &contract, token_a.id(), token_b.id()).await?;

    Ok(())
}

#[tokio::test]
async fn test_contract_info() -> anyhow::Result<()> {
    let (worker, _, contract, token_a, token_b) = initialize_contracts().await?;

    contract_init(&worker, &contract, token_a.id(), token_b.id()).await?;

    let res = contract.call(&worker, "get_contract_info").view().await?;
    assert_eq!(
        res.json::<LiquidityPool>()?,
        LiquidityPool {
            ticker: format!("{}-{}-LP", token_a.id(), token_b.id()),
            decimals: 24,
            token_a_supply: U128::from(0),
            token_b_supply: U128::from(0)
        }
    );

    Ok(())
}

#[tokio::test]
async fn test_deposit_owner() -> anyhow::Result<()> {
    let (worker, owner, contract, token_a, token_b) = initialize_contracts().await?;

    contract_init(&worker, &contract, token_a.id(), token_b.id()).await?;
    storage_deposit(&worker, &token_a, contract.id()).await?;
    mint_tokens(&worker, &token_a, owner.id(), 1_000_000).await?;

    let res = owner
        .call(&worker, token_a.id(), "ft_transfer_call")
        .args_json((
            contract.id(),
            U128::from(1),
            Option::<String>::None,
            "".to_string(),
        ))?
        .max_gas()
        .deposit(1)
        .transact()
        .await?;
    assert!(res.is_success());

    Ok(())
}

async fn initialize_contracts(
) -> anyhow::Result<(Worker<Sandbox>, Account, Contract, Contract, Contract)> {
    let worker = workspaces::sandbox().await?;

    let owner = worker.dev_create_account().await?;

    let contract = worker
        .dev_deploy(&fs::read("../res/orderly_contract.wasm").await?)
        .await?;
    contract
        .call(&worker, "new")
        .args_json((owner.id(),))?
        .max_gas()
        .transact()
        .await?;

    let token_a_contract = worker
        .dev_deploy(&fs::read("../res/test_token.wasm").await?)
        .await?;
    token_a_contract.call(&worker, "new").transact().await?;

    let token_b_contract = worker
        .dev_deploy(&fs::read("../res/test_token.wasm").await?)
        .await?;
    token_b_contract.call(&worker, "new").transact().await?;

    Ok((worker, owner, contract, token_a_contract, token_b_contract))
}

async fn contract_init(
    worker: &Worker<Sandbox>,
    contract: &Contract,
    token_a: &AccountId,
    token_b: &AccountId,
) -> anyhow::Result<()> {
    let res = contract
        .call(worker, "init")
        .args_json((token_a, token_b))?
        .max_gas()
        .transact()
        .await?;
    assert!(res.is_success());
    Ok(())
}

async fn storage_deposit(
    worker: &Worker<Sandbox>,
    token: &Contract,
    receiver: &AccountId,
) -> anyhow::Result<()> {
    let res = token
        .call(worker, "storage_deposit")
        .args_json((receiver, Option::<bool>::None))?
        .deposit(1_000_000_000_000_000_000_000_000)
        .transact()
        .await?;
    assert!(res.is_success());
    Ok(())
}

async fn mint_tokens(
    worker: &Worker<Sandbox>,
    token: &Contract,
    receiver: &AccountId,
    amount: u128,
) -> anyhow::Result<()> {
    let res = token
        .call(worker, "mint")
        .args_json((receiver, U128::from(amount)))?
        .transact()
        .await?;
    assert!(res.is_success());
    Ok(())
}
