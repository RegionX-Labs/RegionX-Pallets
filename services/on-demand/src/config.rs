//! This file contains all the configuration related traits.

use crate::RelayBlockNumber;
use codec::Codec;
use cumulus_client_consensus_common as consensus_common;
use cumulus_relay_chain_interface::RelayChainInterface;
use on_demand_primitives::ThresholdParameterT;
use sc_client_api::UsageProvider;
use sc_consensus_aura::standalone::slot_author;
use sc_service::Arc;
use sc_transaction_pool_api::MaintainedTransactionPool;
use sp_api::ProvideRuntimeApi;
use sp_application_crypto::RuntimeAppPublic;
use sp_consensus_aura::AuraApi;
use sp_core::{crypto::Pair as PairT, H256};
use sp_runtime::{
	generic::BlockId,
	traits::{AtLeast32BitUnsigned, Block as BlockT, Debug, MaybeDisplay, Member, PhantomData},
};
use std::{error::Error, fmt::Display, future::Future, pin::Pin, time::Duration};

pub trait OnDemandConfig {
	/// Custom order placement criteria.
	type OrderPlacementCriteria: OrderCriteria;

	/// Author identifier.
	type AuthorPub: Member + RuntimeAppPublic + Display + Send;

	/// Block type.
	type Block: BlockT;

	/// Relay chain.
	type R: RelayChainInterface + Clone;

	/// Parachain.
	type P: ProvideRuntimeApi<Self::Block> + UsageProvider<Self::Block> + Send + Sync;

	/// Extrinsic pool.
	type ExPool: MaintainedTransactionPool<Block = Self::Block, Hash = <Self::Block as BlockT>::Hash>
		+ 'static;

	/// Balance type.
	type Balance: Codec
		+ MaybeDisplay
		+ 'static
		+ Debug
		+ Send
		+ Into<u128>
		+ AtLeast32BitUnsigned
		+ Copy
		+ From<u128>;

	/// On-demand pallet threshold parameter.
	type ThresholdParameter: ThresholdParameterT;

	type OrderPlacerFuture: Future<Output = Result<Self::AuthorPub, Box<dyn Error>>> + Send;

	fn order_placer(
		relay_chain: &'static Self::R,
		para: &Self::P,
		relay_hash: H256,
		para_hash: <Self::Block as BlockT>::Hash,
		relay_chain_slot_duration: Duration,
	) -> Self::OrderPlacerFuture;
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

pub struct OnDemandAura<R, P, Block, Pair, ExPool, Balance, C, T>(
	PhantomData<(R, P, Block, Pair, ExPool, Balance, C, T)>,
);
impl<P, R, Block, Pair, ExPool, Balance, Criteria, Threshold> OnDemandConfig
	for OnDemandAura<R, P, Block, Pair, ExPool, Balance, Criteria, Threshold>
where
	R: RelayChainInterface + Clone + Sync + Send,
	P: ProvideRuntimeApi<Block> + UsageProvider<Block> + Sync + Send,
	P::Api: AuraApi<Block, Pair::Public>,
	Criteria: OrderCriteria,
	Pair: PairT + 'static,
	ExPool: MaintainedTransactionPool<Block = Block, Hash = <Block as BlockT>::Hash> + 'static,
	Balance: Codec
		+ MaybeDisplay
		+ 'static
		+ Debug
		+ Send
		+ Into<u128>
		+ AtLeast32BitUnsigned
		+ Copy
		+ From<u128>,
	Pair::Public: RuntimeAppPublic + Display + Member + Codec,
	Block: BlockT<Hash = H256>,
	Threshold: ThresholdParameterT,
{
	type P = P;
	type R = R;

	type OrderPlacementCriteria = Criteria;
	type AuthorPub = Pair::Public;
	type Block = Block;

	type ExPool = ExPool;
	type Balance = Balance;
	type ThresholdParameter = Threshold;

	type OrderPlacerFuture =
		Pin<Box<dyn Future<Output = Result<Self::AuthorPub, Box<dyn Error>>> + Send>>;

	fn order_placer(
		relay_chain: &'static R,
		para: &P,
		relay_hash: H256,
		para_hash: <Block as BlockT>::Hash,
		relay_chain_slot_duration: Duration,
	) -> Self::OrderPlacerFuture {
		let authorities_result = para.runtime_api().authorities(para_hash).map_err(Box::new);
		let relay_header_future = relay_chain.header(BlockId::Hash(relay_hash));

		Box::pin(async move {
			let authorities = authorities_result?;
			let relay_header = relay_header_future.await?.ok_or("Header not found")?;

			let (slot, _) = consensus_common::relay_slot_and_timestamp(
				&relay_header,
				relay_chain_slot_duration,
			)
			.ok_or("Failed to get current relay slot")?;

			let expected_author: &Pair::Public =
				slot_author::<Pair>(slot, &authorities).ok_or("Failed to get current author")?;

			Ok(expected_author.clone())
		})
	}
}
