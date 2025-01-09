//! This file contains all the configuration related traits.

use crate::{ParaId, PersistedValidationData, RelayBlockNumber};
use codec::{Codec, Decode, Encode};
use cumulus_primitives_core::{relay_chain::BlakeTwo256, ConsensusEngineId};
use cumulus_relay_chain_interface::{PHash, RelayChainInterface};
use on_demand_primitives::{well_known_keys::EVENTS, OnDemandRuntimeApi, ThresholdParameterT};
use sc_client_api::UsageProvider;
use sc_service::Arc;
use sc_transaction_pool_api::MaintainedTransactionPool;
use scale_info::TypeInfo;
use sp_api::ProvideRuntimeApi;
use sp_application_crypto::RuntimeAppPublic;
use sp_consensus_aura::{AuraApi, Slot, AURA_ENGINE_ID};
use sp_core::crypto::Pair as PairT;
use sp_inherents::InherentIdentifier;
use sp_runtime::{
	generic::BlockId,
	traits::{
		AtLeast32BitUnsigned, Block as BlockT, Debug, Header as HeaderT, MaybeDisplay, Member,
		PhantomData,
	},
};
use sp_trie::{Trie, LayoutV1, TrieDBBuilder};
use std::{error::Error, fmt::Display};

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

	fn order_placer(
		para: &Self::P,
		relay_hash: PHash,
		para_header: <Self::Block as BlockT>::Header,
	) -> Result<Self::AuthorPub, Box<dyn Error>>;
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

pub struct OnDemandSlot<R, P, Block, Pair, ExPool, Balance, C, T>(
	PhantomData<(R, P, Block, Pair, ExPool, Balance, C, T)>,
);
#[async_trait::async_trait]
impl<P, R, Block, Pair, ExPool, Balance, Criteria, Threshold> OnDemandConfig
	for OnDemandSlot<R, P, Block, Pair, ExPool, Balance, Criteria, Threshold>
where
	R: RelayChainInterface + Clone + Sync + Send,
	P: ProvideRuntimeApi<Block> + UsageProvider<Block> + Sync + Send,
	P::Api: AuraApi<Block, Pair::Public>
		+ OnDemandRuntimeApi<Block, Balance, RelayBlockNumber, Threshold>,
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
	Block: BlockT,
	<<Block as BlockT>::Header as HeaderT>::Number: Into<u128>,
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

	fn order_placer(
		para: &P,
		_relay_hash: PHash,
		para_header: <Self::Block as BlockT>::Header,
	) -> Result<Self::AuthorPub, Box<dyn Error>> {
		let para_hash = para_header.hash();
		let para_height: u128 = para_header.number().clone().into();
		let authorities = para.runtime_api().authorities(para_hash).map_err(Box::new)?;
		let slot_width = para.runtime_api().slot_width(para_hash).map_err(Box::new)?;

		let indx = (para_height >> slot_width) % authorities.len() as u128;
		let author = authorities.get(indx as usize).ok_or("Failed to get selected collator")?;

		Ok(author.clone())
	}
}

pub struct OnDemandAura<R, P, Block, Pair, ExPool, Balance, C, T>(
	PhantomData<(R, P, Block, Pair, ExPool, Balance, C, T)>,
);
#[async_trait::async_trait]
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
	Block: BlockT,
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

	fn order_placer(
		para: &P,
		_relay_hash: PHash,
		para_header: <Self::Block as BlockT>::Header,
	) -> Result<Self::AuthorPub, Box<dyn Error>> {
		let para_hash = para_header.hash();
		let authorities = para.runtime_api().authorities(para_hash).map_err(Box::new)?;

		let author_index = find_author(
			para_header.digest().logs().iter().filter_map(|d| d.as_pre_runtime()),
			authorities.len(),
		)
		.ok_or("Could not find aura author index")?;

		let author = authorities.get(author_index as usize).ok_or("Invalid aura index")?;
		Ok(author.clone())
	}
}

async fn collect_relay_storage_proof(
	relay_chain: &impl RelayChainInterface,
	relay_parent: PHash,
) -> Option<sp_state_machine::StorageProof> {
	let mut relevant_keys = Vec::new();
	// Get storage proof for events at a specific block.
	relevant_keys.push(EVENTS.to_vec());

	let relay_storage_proof = relay_chain.prove_read(relay_parent, &relevant_keys).await;
	match relay_storage_proof {
		Ok(proof) => Some(proof),
		Err(err) => {
			log::info!("RelayChainError:{:?}", err);
			None
		},
	}
}

fn find_author<'a, I>(digests: I, authorities_len: usize) -> Option<u32>
where
	I: 'a + IntoIterator<Item = (ConsensusEngineId, &'a [u8])>,
{
	for (id, mut data) in digests.into_iter() {
		if id == AURA_ENGINE_ID {
			let slot = Slot::decode(&mut data).ok()?;
			let author_index = *slot % authorities_len as u64;
			return Some(author_index as u32)
		}
	}

	None
}

// Identifier of the order inherent
pub const INHERENT_IDENTIFIER: InherentIdentifier = *b"orderiht";

#[derive(Encode, Decode, sp_core::RuntimeDebug, Clone, PartialEq, TypeInfo)]
pub struct OrderInherentData<AuthorityId> {
	pub relay_storage_proof: sp_trie::StorageProof,
	pub validation_data: Option<PersistedValidationData>,
	pub para_id: ParaId,
	pub sequence_number: u64,
	pub author_pub: Option<AuthorityId>,
}

impl<AuthorityId: Clone> OrderInherentData<AuthorityId> {
	pub async fn create_at(
		relay_chain_interface: &impl RelayChainInterface,
		para_id: ParaId,
	) -> Option<OrderInherentData<AuthorityId>> {
		let best_hash = relay_chain_interface.best_block_hash().await.unwrap(); // TODO
		let header = relay_chain_interface.header(BlockId::Hash(best_hash)).await.unwrap().unwrap(); // TODO

		let relay_storage_proof =
			collect_relay_storage_proof(relay_chain_interface, header.hash()).await?;

		let db = relay_storage_proof.to_memory_db::<BlakeTwo256>();
		// TODO: LayoutV1 ?
		let trie = TrieDBBuilder::<LayoutV1<BlakeTwo256>>::new(&db, &header.state_root).build();
		let events = trie.get(EVENTS); // TODO: try to decode.

		/*
		Some(OrderInherentData {
			relay_storage_proof: relay_storage_proof.clone(),
			validation_data: validation_data.clone(),
			para_id,
			sequence_number,
			author_pub: author_pub.clone(),
		})
		*/
		None
	}
}

#[async_trait::async_trait]
impl<AuthorityId: Send + Sync + Codec> sp_inherents::InherentDataProvider
	for OrderInherentData<AuthorityId>
{
	async fn provide_inherent_data(
		&self,
		inherent_data: &mut sp_inherents::InherentData,
	) -> Result<(), sp_inherents::Error> {
		inherent_data.put_data(INHERENT_IDENTIFIER, &self)
	}

	async fn try_handle_error(
		&self,
		_: &sp_inherents::InherentIdentifier,
		_: &[u8],
	) -> Option<Result<(), sp_inherents::Error>> {
		None
	}
}
