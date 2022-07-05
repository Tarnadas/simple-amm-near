use near_sdk::{json_types::U128, AccountId};
use orderly_contract::LiquidityPool;
use tokio::fs;
use workspaces::{network::Sandbox, prelude::*, Contract, Worker};

#[tokio::test]
async fn test_init() -> anyhow::Result<()> {
    let (worker, contract, token_a_contract, token_b_contract) = initialize_contracts().await?;

    let res = contract
        .call(&worker, "init")
        .args_json((token_a_contract.id(), token_b_contract.id()))?
        .max_gas()
        .transact()
        .await?;
    assert!(res.is_success());

    let res = contract.call(&worker, "get_contract_info").view().await?;
    assert_eq!(
        res.json::<LiquidityPool>()?,
        LiquidityPool {
            ticker: format!("{}-{}-LP", token_a_contract.id(), token_b_contract.id()),
            decimals: 24,
            token_a: U128::from(0),
            token_b: U128::from(0)
        }
    );

    Ok(())
}

async fn initialize_contracts() -> anyhow::Result<(Worker<Sandbox>, Contract, Contract, Contract)> {
    let worker = workspaces::sandbox().await?;
    let contract = worker
        .dev_deploy(&fs::read("../res/orderly_contract.wasm").await?)
        .await?;
    contract
        .call(&worker, "new")
        .args_json(("owner.near".parse::<AccountId>().unwrap(),))?
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

    Ok((worker, contract, token_a_contract, token_b_contract))
}
