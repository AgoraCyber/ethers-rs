use ethers_rs::hardhat;

hardhat!(Example);

#[cfg(test)]
mod tests {

    use ethers_rs::{
        hardhat::{hardhat_network, utils::*},
        *,
    };

    use super::*;

    #[async_std::test]
    async fn test_deploy() {
        _ = pretty_env_logger::try_init();

        let mut network = hardhat_network!();

        network.start().await.expect("Start hardhat network");

        let s0 = get_hardhat_network_account(0);

        let provider = get_hardhat_network_provider();

        let value = "1.1".parse::<Ether>().expect("Parse payment eth value");

        let client: Client = (provider.clone(), s0).into();

        // Deploy contract and send ether to deployed
        let lock = Example::deploy_with(client.clone(), value.clone().to_tx_options())
            .await
            .expect("Deploy lock contract");

        log::debug!("deploy lock success, {}", lock.address);

        let balance = client.balance().await.expect("Get after deploy balance");

        let mut tx = lock.withdraw().await.expect("Try withdraw");

        let receipt = tx.wait().await.expect("Wait withdraw tx mint");

        let balance_after = client.balance().await.expect("Get after deploy balance");

        let used = receipt.gas_used * receipt.effective_gas_price;

        log::debug!(
            "balance({}) balance after({}) used({}) withdraw({})",
            Gwei(balance),
            Gwei(balance_after),
            Gwei(used),
            value.unit_to::<Gwei>()
        );

        assert_eq!(balance + value.to_u256() - used, balance_after);

        let data = lock
            .get_from((100usize.into(), 200usize.into()))
            .await
            .expect("call from pure method");

        assert_eq!(data, 100usize.into());

        let data = lock
            .get_to((100usize.into(), 300usize.into()))
            .await
            .expect("call from pure method");

        assert_eq!(data, 300usize.into());

        let (from, to) = lock.get().await.expect("call from pure method");

        assert_eq!(from, 1000usize.into());

        assert_eq!(to, 140u128.into());
    }
}
