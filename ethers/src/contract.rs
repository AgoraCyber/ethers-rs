use ethers_types_rs::{Address, H256};

use crate::{client::Client, TxOptions};

pub struct ContractContext {
    pub address: Address,
    pub client: Client,
}

impl ContractContext {
    pub async fn eth_call(&mut self, call_data: Vec<u8>) -> anyhow::Result<Vec<u8>> {
        self.client.eth_call(self.address, call_data).await
    }

    /// Send raw transaction to contract address.
    ///
    /// # Parameters
    ///
    /// * `value` - Amount of native token transferring to contract.
    /// * `call_data` - Invoke contract method call data. visit solidity [`abi`](https://docs.soliditylang.org/en/v0.8.17/abi-spec.html) for more information
    ///
    pub async fn send_raw_transaction(
        &mut self,
        method: &str,
        call_data: Vec<u8>,
        ops: TxOptions,
    ) -> anyhow::Result<H256> {
        self.client
            .send_raw_transaction(method, Some(self.address.clone()), call_data, ops)
            .await
    }
}

#[cfg(test)]
mod tests {
    use ethers_types_rs::{Ether, Gwei};
    use serde_json::json;

    use crate::TxOptions;

    #[test]
    fn test_tx_ops() {
        let value: Ether = "1.01".try_into().expect("");

        let gas_price: Gwei = "1.01".try_into().expect("");

        let ops: TxOptions = json!({
            "gas_price":gas_price,
            "value": value,
        })
        .try_into()
        .expect("To ops");

        assert_eq!(ops.gas_price, Some(gas_price.into()));

        assert_eq!(ops.value, Some(value.into()));
    }
}
