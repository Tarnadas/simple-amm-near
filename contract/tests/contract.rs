use near_sdk::json_types::U128;
use orderly_contract::ContractInfo;
use tokio::fs;
use workspaces::{
    network::Sandbox,
    prelude::*,
    result::{CallExecutionDetails, ViewResultDetails},
    Account, AccountId, Contract, Worker,
};

#[tokio::test]
async fn test_init() -> anyhow::Result<()> {
    let (worker, _, contract, token_a, token_b) = initialize_contracts().await?;

    contract_init(&worker, &contract, token_a.id(), token_b.id()).await?;

    Ok(())
}

#[tokio::test]
async fn test_get_contract_info() -> anyhow::Result<()> {
    let (worker, _, contract, token_a, token_b) = initialize_contracts().await?;

    contract_init(&worker, &contract, token_a.id(), token_b.id()).await?;

    assert_token_supplies(
        &worker,
        &contract,
        token_a.id(),
        0.into(),
        token_b.id(),
        0.into(),
    )
    .await?;

    Ok(())
}

#[tokio::test]
async fn test_get_contract_info_no_init() -> anyhow::Result<()> {
    let (worker, _, contract, _, _) = initialize_contracts().await?;

    let res = contract.call(&worker, "get_contract_info").view().await?;
    assert_eq!(res.json::<Option<ContractInfo>>()?, None);

    Ok(())
}

#[tokio::test]
async fn test_deposit_owner() -> anyhow::Result<()> {
    let (worker, owner, contract, token_a, token_b) = initialize_contracts().await?;

    contract_init(&worker, &contract, token_a.id(), token_b.id()).await?;
    storage_deposit(&worker, &token_a, contract.id()).await?;
    mint_tokens(&worker, &token_a, owner.id(), 1_000_000).await?;

    transfer_tokens(&worker, &owner, contract.id(), token_a.id(), 1_000.into()).await?;

    let res = ft_balance_of(&worker, &token_a, owner.id()).await?;
    assert_eq!(res.json::<U128>()?, U128::from(999_000));
    assert_token_supplies(
        &worker,
        &contract,
        token_a.id(),
        1_000.into(),
        token_b.id(),
        0.into(),
    )
    .await?;

    Ok(())
}

#[tokio::test]
async fn test_deposit_owner_2() -> anyhow::Result<()> {
    let (worker, owner, contract, token_a, token_b) = initialize_contracts().await?;

    contract_init(&worker, &contract, token_a.id(), token_b.id()).await?;
    storage_deposit(&worker, &token_a, contract.id()).await?;
    mint_tokens(&worker, &token_a, owner.id(), 1_000_000).await?;
    storage_deposit(&worker, &token_b, contract.id()).await?;
    mint_tokens(&worker, &token_b, owner.id(), 1_000_000).await?;

    transfer_tokens(&worker, &owner, contract.id(), token_a.id(), 1_000.into()).await?;
    transfer_tokens(&worker, &owner, contract.id(), token_b.id(), 69_000.into()).await?;
    transfer_tokens(&worker, &owner, contract.id(), token_b.id(), 42.into()).await?;

    let res = ft_balance_of(&worker, &token_a, owner.id()).await?;
    assert_eq!(res.json::<U128>()?, U128::from(999_000));
    let res = ft_balance_of(&worker, &token_b, owner.id()).await?;
    assert_eq!(res.json::<U128>()?, U128::from(1_000_000 - 69_000 - 42));
    assert_token_supplies(
        &worker,
        &contract,
        token_a.id(),
        1_000.into(),
        token_b.id(),
        69_042.into(),
    )
    .await?;

    Ok(())
}

#[tokio::test]
async fn test_deposit_owner_no_init_should_refund() -> anyhow::Result<()> {
    let (worker, owner, contract, token_a, _) = initialize_contracts().await?;

    storage_deposit(&worker, &token_a, contract.id()).await?;
    mint_tokens(&worker, &token_a, owner.id(), 1_000_000).await?;

    transfer_tokens(&worker, &owner, contract.id(), token_a.id(), 1_000.into()).await?;

    let res = ft_balance_of(&worker, &token_a, owner.id()).await?;
    assert_eq!(res.json::<U128>()?, U128::from(1_000_000));

    Ok(())
}

#[tokio::test]
async fn test_swap() -> anyhow::Result<()> {
    let (worker, owner, contract, token_a, token_b) = initialize_contracts().await?;
    let user = worker.dev_create_account().await?;

    contract_init(&worker, &contract, token_a.id(), token_b.id()).await?;
    storage_deposit(&worker, &token_a, contract.id()).await?;
    mint_tokens(&worker, &token_a, owner.id(), 1_000_000).await?;
    mint_tokens(&worker, &token_a, user.id(), 1_000_000).await?;
    storage_deposit(&worker, &token_b, contract.id()).await?;
    mint_tokens(&worker, &token_b, owner.id(), 1_000_000).await?;
    mint_tokens(&worker, &token_b, user.id(), 1_000_000).await?;
    transfer_tokens(&worker, &owner, contract.id(), token_a.id(), 1_000.into()).await?;
    transfer_tokens(&worker, &owner, contract.id(), token_b.id(), 1_000.into()).await?;

    transfer_tokens(&worker, &user, contract.id(), token_a.id(), 100.into()).await?;

    let res = ft_balance_of(&worker, &token_a, user.id()).await?;
    assert_eq!(res.json::<U128>()?, U128::from(999_900));
    let res = ft_balance_of(&worker, &token_b, user.id()).await?;
    assert_eq!(res.json::<U128>()?, U128::from(1_000_091));
    assert_token_supplies(
        &worker,
        &contract,
        token_a.id(),
        1_100.into(),
        token_b.id(),
        909.into(),
    )
    .await?;

    Ok(())
}

#[tokio::test]
async fn test_swap_2() -> anyhow::Result<()> {
    let (worker, owner, contract, token_a, token_b) = initialize_contracts().await?;
    let user = worker.dev_create_account().await?;

    contract_init(&worker, &contract, token_a.id(), token_b.id()).await?;
    storage_deposit(&worker, &token_a, contract.id()).await?;
    mint_tokens(&worker, &token_a, owner.id(), 1_000_000).await?;
    mint_tokens(&worker, &token_a, user.id(), 1_000_000).await?;
    storage_deposit(&worker, &token_b, contract.id()).await?;
    mint_tokens(&worker, &token_b, owner.id(), 1_000_000).await?;
    mint_tokens(&worker, &token_b, user.id(), 1_000_000).await?;
    transfer_tokens(&worker, &owner, contract.id(), token_a.id(), 1_000.into()).await?;
    transfer_tokens(&worker, &owner, contract.id(), token_b.id(), 1_000.into()).await?;

    transfer_tokens(&worker, &user, contract.id(), token_a.id(), 50.into()).await?;
    transfer_tokens(&worker, &user, contract.id(), token_b.id(), 150.into()).await?;
    transfer_tokens(&worker, &user, contract.id(), token_a.id(), 200.into()).await?;

    let res = ft_balance_of(&worker, &token_a, user.id()).await?;
    assert_eq!(res.json::<U128>()?, U128::from(999_893));
    let res = ft_balance_of(&worker, &token_b, user.id()).await?;
    assert_eq!(res.json::<U128>()?, U128::from(1_000_098));
    assert_token_supplies(
        &worker,
        &contract,
        token_a.id(),
        1_107.into(),
        token_b.id(),
        902.into(),
    )
    .await?;

    Ok(())
}

#[tokio::test]
async fn test_swap_all() -> anyhow::Result<()> {
    let (worker, owner, contract, token_a, token_b) = initialize_contracts().await?;
    let user = worker.dev_create_account().await?;

    contract_init(&worker, &contract, token_a.id(), token_b.id()).await?;
    storage_deposit(&worker, &token_a, contract.id()).await?;
    mint_tokens(&worker, &token_a, owner.id(), 1_000_000).await?;
    mint_tokens(&worker, &token_a, user.id(), 1_000_000).await?;
    storage_deposit(&worker, &token_b, contract.id()).await?;
    mint_tokens(&worker, &token_b, owner.id(), 1_000_000).await?;
    mint_tokens(&worker, &token_b, user.id(), 1_000_000).await?;
    transfer_tokens(&worker, &owner, contract.id(), token_a.id(), 1_000.into()).await?;
    transfer_tokens(&worker, &owner, contract.id(), token_b.id(), 1_000.into()).await?;

    transfer_tokens(
        &worker,
        &user,
        contract.id(),
        token_a.id(),
        1_000_000.into(),
    )
    .await?;

    let res = ft_balance_of(&worker, &token_a, user.id()).await?;
    assert_eq!(res.json::<U128>()?, U128::from(0));
    let res = ft_balance_of(&worker, &token_b, user.id()).await?;
    assert_eq!(res.json::<U128>()?, U128::from(1_001_000));
    assert_token_supplies(
        &worker,
        &contract,
        token_a.id(),
        1_001_000.into(),
        token_b.id(),
        0.into(),
    )
    .await?;

    Ok(())
}

#[tokio::test]
async fn test_swap_not_enough_liquidity() -> anyhow::Result<()> {
    let (worker, owner, contract, token_a, token_b) = initialize_contracts().await?;
    let user = worker.dev_create_account().await?;

    contract_init(&worker, &contract, token_a.id(), token_b.id()).await?;
    storage_deposit(&worker, &token_a, contract.id()).await?;
    mint_tokens(&worker, &token_a, owner.id(), 1_000_000).await?;
    mint_tokens(&worker, &token_a, user.id(), 1_000_000).await?;
    storage_deposit(&worker, &token_b, contract.id()).await?;
    mint_tokens(&worker, &token_b, owner.id(), 1_000_000).await?;
    mint_tokens(&worker, &token_b, user.id(), 1_000_000).await?;
    transfer_tokens(&worker, &owner, contract.id(), token_a.id(), 1_000.into()).await?;

    transfer_tokens(&worker, &user, contract.id(), token_a.id(), 10.into()).await?;

    let res = ft_balance_of(&worker, &token_a, user.id()).await?;
    assert_eq!(res.json::<U128>()?, U128::from(1_000_000));
    let res = ft_balance_of(&worker, &token_b, user.id()).await?;
    assert_eq!(res.json::<U128>()?, U128::from(1_000_000));
    assert_token_supplies(
        &worker,
        &contract,
        token_a.id(),
        1_000.into(),
        token_b.id(),
        0.into(),
    )
    .await?;

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
    token_a_contract
        .call(&worker, "new")
        .args_json(("TokenA", "TKNA"))?
        .transact()
        .await?;

    let token_b_contract = worker
        .dev_deploy(&fs::read("../res/test_token.wasm").await?)
        .await?;
    token_b_contract
        .call(&worker, "new")
        .args_json(("TokenB", "TKNB"))?
        .transact()
        .await?;

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

async fn transfer_tokens(
    worker: &Worker<Sandbox>,
    sender: &Account,
    receiver: &AccountId,
    token: &AccountId,
    amount: U128,
) -> anyhow::Result<CallExecutionDetails> {
    let res = sender
        .call(worker, token, "ft_transfer_call")
        .args_json((receiver, amount, Option::<String>::None, "".to_string()))?
        .max_gas()
        .deposit(1)
        .transact()
        .await?;
    assert!(res.is_success());
    Ok(res)
}

async fn ft_balance_of(
    worker: &Worker<Sandbox>,
    token: &Contract,
    account_id: &AccountId,
) -> anyhow::Result<ViewResultDetails> {
    let res = token
        .call(worker, "ft_balance_of")
        .args_json((account_id,))?
        .view()
        .await?;
    Ok(res)
}

async fn assert_token_supplies(
    worker: &Worker<Sandbox>,
    contract: &Contract,
    token_a: &AccountId,
    token_a_supply: U128,
    token_b: &AccountId,
    token_b_supply: U128,
) -> anyhow::Result<()> {
    let res = contract.call(worker, "get_contract_info").view().await?;
    assert_eq!(
        res.json::<ContractInfo>()?,
        ContractInfo {
            token_a_id: token_a.to_string().parse().unwrap(),
            token_a_name: "TokenA".to_string(),
            token_a_symbol: "TKNA".to_string(),
            token_a_supply,
            token_a_decimals: 12,
            token_b_id: token_b.to_string().parse().unwrap(),
            token_b_name: "TokenB".to_string(),
            token_b_symbol: "TKNB".to_string(),
            token_b_supply,
            token_b_decimals: 12
        }
    );
    Ok(())
}
