mod redis_cache;

use redis_cache::hash_inputs;
use cached_macro::custom_cached;
use web3::types::{H160, U256};
use std::str::FromStr;

#[tokio::main]
async fn main() {
    let result = expensive_calculation(11).await;
    println!("Result: {}", result.unwrap());

    // Call again to test cache hit
    let cached_result = expensive_calculation(11).await;
    println!("Cached result: {}", cached_result.unwrap());

    let balance = get_ethereum_balance_rpc("0x95222290dd7278aa3ddd389cc1e1d165cc4bafe5").await;
    let balance2 = get_ethereum_balance_rpc("0x4Db2436B63D0Af2DFDaDb9465E8eAC38AC1A8eC1").await;
    println!("Ethereum balance: {}", balance.unwrap());
    println!("Ethereum balance: {}", balance2.unwrap());
}

#[custom_cached(20)]
pub async fn expensive_calculation(input: u64) -> Result<i64, Box<dyn std::error::Error>> {
    // Simulate an expensive calculation
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    Ok::<i64, Box<dyn std::error::Error>>(input as i64 * 2)
}

#[custom_cached(10)]
pub async fn get_ethereum_balance_rpc(address: &str) -> Result<f64, Box<dyn std::error::Error>> {
    let rpc_url = "https://ethereum-rpc.publicnode.com"; // Replace with your Infura project ID or use another Ethereum node
    let transport = web3::transports::Http::new(rpc_url)?;
    let web3 = web3::Web3::new(transport);

    let address = H160::from_str(address)?;
    let balance = web3.eth().balance(address, None).await?;

    let balance_eth = wei_to_eth(balance);
    Ok(balance_eth)
}

fn wei_to_eth(wei: U256) -> f64 {
    let wei_f: f64 = wei.as_u128() as f64;
    wei_f / 1e18
}
