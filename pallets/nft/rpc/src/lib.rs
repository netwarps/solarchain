extern crate sp_api;
extern crate sp_blockchain;
extern crate jsonrpc_core;
extern crate sp_runtime;

use std::sync::Arc;

use jsonrpc_core::{Error, ErrorCode, Result};
use jsonrpc_derive::rpc;
use codec::Codec;
use serde::{Deserialize, Serialize};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{
    generic::BlockId,
    traits::Block as BlockT,
};

pub use pallet_nft_rpc_runtime_api::NftApi as NftRuntimeApi;

const RUNTIME_ERROR: i64 = 1;

/// A struct that encodes RPC parameters required for a call to a smart-contract.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct TokensOfRequest<AccountId> {
    account: AccountId,
    offset: u64,
    limit: u64,
}

#[rpc]
pub trait NftApi<BlockHash, AccountId> {
    #[rpc(name = "nft_tokensOf")]
    fn tokens_of(
        &self,
        req: TokensOfRequest<AccountId>,
        at: Option<BlockHash>,
    ) -> Result<Vec<u128>>;
}

/// An implementation of nft specific RPC methods.
pub struct Nft<C, B> {
    client: Arc<C>,
    _marker: std::marker::PhantomData<B>,
}

impl<C, B> Nft<C, B> {
    /// Create new `Contracts` with the given reference to the client.
    pub fn new(client: Arc<C>) -> Self {
        Nft { client, _marker: Default::default() }
    }
}

impl<C, Block, AccountId> NftApi<<Block as BlockT>::Hash, AccountId> for Nft<C, Block>
where
    Block: BlockT,
    C: Send + Sync + 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
    C::Api: NftRuntimeApi<Block, AccountId>,
    AccountId: Codec,
{
    fn tokens_of(
        &self,
        req: TokensOfRequest<AccountId>,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<Vec<u128>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(||
            // If the block hash is not supplied assume the best block.
            self.client.info().best_hash));

        let TokensOfRequest { account, limit, offset } = req;

        let ret = api
            .tokens_of(&at, account, limit, offset).map_err(runtime_error_into_rpc_err)?;

        Ok(ret)
    }
}

/// Converts a runtime trap into an RPC error.
fn runtime_error_into_rpc_err(err: impl std::fmt::Debug) -> Error {
    Error {
        code: ErrorCode::ServerError(RUNTIME_ERROR),
        message: "Runtime error".into(),
        data: Some(format!("{:?}", err).into()),
    }
}
