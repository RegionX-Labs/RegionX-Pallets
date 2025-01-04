//! This file contains all the configuration related traits.

use crate::RelayBlockNumber;
use cumulus_relay_chain_interface::RelayChainInterface;
use sc_client_api::UsageProvider;
use sc_service::Arc;
use sc_transaction_pool_api::MaintainedTransactionPool;
use sp_api::ProvideRuntimeApi;
use sp_application_crypto::RuntimeAppPublic;
use sp_consensus_aura::{sr25519::AuthorityId, AuraApi};
use sp_runtime::traits::{Block as BlockT, Member};

pub trait OnDemandConfig {
	/// Custom order placement criteria.
	type OrderPlacementCriteria: OrderCriteria;

	/// Author identifier.
	type AuthorPub: Member + RuntimeAppPublic + std::fmt::Display;

	/// Block type.
	type Block: BlockT;

	/// Returns the block author for the current slot.
	fn author<R, P>(relay_chain: &R, para: &P, relay_head: Vec<u8>) -> Option<Self::AuthorPub>
	where
		R: RelayChainInterface + Clone,
		P: ProvideRuntimeApi<Self::Block>,
		P::Api: AuraApi<Self::Block, AuthorityId>;
}

pub trait OrderCriteria {
	type Block: BlockT;
	type P: ProvideRuntimeApi<Self::Block> + UsageProvider<Self::Block>;
	type ExPool: MaintainedTransactionPool<Block = Self::Block, Hash = <Self::Block as BlockT>::Hash>
		+ 'static;

	/// Returns true or false depending on whether an order should be placed.
	fn should_place_order(
		parachain: &Self::P,
		transaction_pool: Arc<Self::ExPool>,
		height: RelayBlockNumber,
	) -> bool;
}
