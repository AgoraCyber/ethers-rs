use std::fmt::{Debug, Display};

use ethers_utils_rs::{eth::BlockHash, hex};
use jsonrpc_rs::RPCResult;
use types::{Block, BlockNumberOrTag};

pub mod error;
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
    pub async fn eth_getblockbyhash<B>(&mut self, block_hash: B, hydrated: bool) -> RPCResult<Block>
    where
        B: TryInto<BlockHash>,
        B::Error: Debug + Display,
    {
        let block_hash = block_hash.try_into().map_err(jsonrpc_rs::map_error)?;

        self.rpc_client
            .call("eth_getBlockByHash", (block_hash, hydrated))
            .await
    }

    /// Returns information about a block by number
    pub async fn eth_getblockbynumber<BT>(
        &mut self,
        block_number_or_tag: BT,
        hydrated: bool,
    ) -> RPCResult<Block>
    where
        BT: TryInto<BlockNumberOrTag>,
        BT::Error: Debug + Display,
    {
        let block_number_or_tag = block_number_or_tag
            .try_into()
            .map_err(jsonrpc_rs::map_error)?;

        self.rpc_client
            .call("eth_getBlockByHash", (block_number_or_tag, hydrated))
            .await
    }

    /// Returns the number of transactions in a block from a block matching the given block hash
    pub async fn eth_getblocktransactioncountbyhash<H>(&mut self, hash: H) -> RPCResult<u64>
    where
        H: TryInto<BlockHash>,
        H::Error: Debug + Display,
    {
        let hash = hash.try_into().map_err(jsonrpc_rs::map_error)?;

        let count: String = self
            .rpc_client
            .call("eth_getBlockTransactionCountByNumber", vec![hash])
            .await?;

        Ok(hex::hex_to_integer(&count).map_err(jsonrpc_rs::map_error)?)
    }
}

#[cfg(test)]
mod tests {

    use jsonrpc_rs::RPCResult;

    use crate::providers::http;

    #[async_std::test]
    async fn test_block_number() -> RPCResult<()> {
        _ = pretty_env_logger::try_init();

        let mut provider = http::connect_to("http://localhost:1545");

        let block_number = provider.eth_blocknumber().await?;

        log::debug!("block number {}", block_number);

        Ok(())
    }

    #[async_std::test]
    async fn test_chain_id() -> RPCResult<()> {
        _ = pretty_env_logger::try_init();

        let mut provider = http::connect_to("http://localhost:1545");

        let block_number = provider.eth_chainid().await?;

        log::debug!("chain_id {}", block_number);

        Ok(())
    }

    #[async_std::test]
    async fn test_eth_getblockbynumber() -> RPCResult<()> {
        _ = pretty_env_logger::try_init();

        let mut provider = http::connect_to("http://localhost:1545");

        let block = provider.eth_getblockbynumber("latest", true).await?;

        log::debug!(
            "block
             {}",
            serde_json::to_string(&block).expect("Serialize block to json")
        );

        let block = provider.eth_getblockbynumber("0x1", true).await?;

        log::debug!(
            "block
             {}",
            serde_json::to_string(&block).expect("Serialize block to json")
        );

        Ok(())
    }

    #[async_std::test]
    async fn test_eth_getblocktransactioncountbyhash() -> RPCResult<()> {
        _ = pretty_env_logger::try_init();

        let mut provider = http::connect_to("http://localhost:1545");

        let block = provider.eth_getblockbynumber("latest", true).await?;

        let tx_count = provider
            .eth_getblocktransactioncountbyhash(block.hash)
            .await?;

        assert_eq!(block.transactions.len() as u64, tx_count);

        Ok(())
    }
}
