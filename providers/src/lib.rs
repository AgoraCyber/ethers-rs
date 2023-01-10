use jsonrpc_rs::RPCResult;

pub mod providers;

/// Ether network accessor provider trait
///
#[derive(Clone)]
#[allow(dead_code)]
pub struct Provider {
    rpc_client: jsonrpc_rs::Client,
}

impl Provider {
    pub(crate) fn new(rpc_client: jsonrpc_rs::Client) -> Self {
        Self { rpc_client }
    }

    pub async fn eth_block_number(&mut self) -> RPCResult<u64> {
        let block_number: u64 = self.rpc_client.call("eth_blockNumber", ()).await?;

        Ok(block_number)
    }
}

#[cfg(test)]
mod tests {
    use jsonrpc_rs::RPCResult;

    use crate::providers::http;

    #[async_std::test]
    async fn test_block_number() -> RPCResult<()> {
        _ = pretty_env_logger::try_init();

        let mut provider = http::connect_to("https://data-seed-prebsc-1-s1.binance.org:8545");

        let block_number = provider.eth_block_number().await?;

        log::debug!("block number {}", block_number);

        Ok(())
    }
}
