use ethers_core_rs::utils::{eth::BlockHash, hex};
use jsonrpc_rs::RPCResult;
use types::Block;

pub mod providers;
pub mod types;

/// Ether network api provider
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

    /// Returns the number of most recent block.
    pub async fn eth_blocknumber(&mut self) -> RPCResult<u64> {
        let block_number: String = self
            .rpc_client
            .call("eth_blockNumber", Vec::<String>::new())
            .await?;

        Ok(hex::hex_to_integer(&block_number).map_err(jsonrpc_rs::map_error)?)
    }

    /// Returns the chain ID of the current network
    pub async fn eth_chainid(&mut self) -> RPCResult<u64> {
        let block_number: String = self
            .rpc_client
            .call("eth_chainId", Vec::<String>::new())
            .await?;

        Ok(hex::hex_to_integer(&block_number).map_err(jsonrpc_rs::map_error)?)
    }

    /// Returns information about a block by hash.
    pub async fn get_blockbyhash(
        &mut self,
        block_hash: BlockHash,
        hydrated: bool,
    ) -> RPCResult<Block> {
        self.rpc_client
            .call("eth_getBlockByHash", (block_hash, hydrated))
            .await
    }
}

#[cfg(test)]
mod tests {

    use jsonrpc_rs::RPCResult;

    use crate::providers::http;

    #[async_std::test]
    async fn test_block_number() -> RPCResult<()> {
        _ = pretty_env_logger::try_init();

        let mut provider = http::connect_to("http://localhost:7545");

        let block_number = provider.eth_blocknumber().await?;

        log::debug!("block number {}", block_number);

        Ok(())
    }

    #[async_std::test]
    async fn test_chain_id() -> RPCResult<()> {
        _ = pretty_env_logger::try_init();

        let mut provider = http::connect_to("http://localhost:7545");

        let block_number = provider.eth_chainid().await?;

        log::debug!("chain_id {}", block_number);

        Ok(())
    }
}
