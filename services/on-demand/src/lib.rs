use codec::{Codec, Decode};
use cumulus_primitives_core::{
	relay_chain::BlockNumber as RelayBlockNumber, ParaId, PersistedValidationData,
};
use cumulus_relay_chain_interface::{RelayChainInterface, RelayChainResult};
use futures::{pin_mut, select, Stream, StreamExt};
use polkadot_primitives::OccupiedCoreAssumption;
use sc_client_api::UsageProvider;
use sc_service::TaskManager;
use sc_transaction_pool_api::{InPoolTransaction, MaintainedTransactionPool};
use sp_api::ProvideRuntimeApi;
use sp_consensus_aura::{sr25519::AuthorityId, AuraApi};
use sp_core::H256;
use sp_keystore::KeystorePtr;
use sp_runtime::traits::{AtLeast32BitUnsigned, Block as BlockT, Header, MaybeDisplay};
use std::{error::Error, fmt::Debug, net::SocketAddr, sync::Arc};

/// Start all the on-demand order creation related tasks.
pub async fn start_on_demand<P, R, ExPool, Block, Balance>(
	parachain: Arc<P>,
	para_id: ParaId,
	relay_chain: R,
	transaction_pool: Arc<ExPool>,
	task_manager: &TaskManager,
	keystore: KeystorePtr,
	relay_rpc: Option<SocketAddr>,
) -> sc_service::error::Result<()>
where
	Block: BlockT,
	Balance: Codec
		+ MaybeDisplay
		+ 'static
		+ Debug
		+ Send
		+ Into<u128>
		+ AtLeast32BitUnsigned
		+ Copy
		+ From<u128>,
	R: RelayChainInterface + Clone + 'static,
	P: Send + Sync + 'static + ProvideRuntimeApi<Block> + UsageProvider<Block>,
	P::Api: AuraApi<Block, AuthorityId>,
	ExPool: MaintainedTransactionPool<Block = Block, Hash = <Block as BlockT>::Hash> + 'static,
{
	let on_demand_task = run_on_demand_task::<P, R, Block, ExPool, Balance>(
		para_id,
		parachain,
		relay_chain,
		keystore,
		transaction_pool,
	);

	// TODO: spawn_blocking?
	task_manager.spawn_essential_handle().spawn_blocking(
		"on-demand order placement task",
		"on-demand",
		on_demand_task,
	);

	Ok(())
}

async fn run_on_demand_task<P, R, Block, ExPool, Balance>(
	para_id: ParaId,
	parachain: Arc<P>,
	relay_chain: R,
	keystore: KeystorePtr,
	transaction_pool: Arc<ExPool>,
) where
	Block: BlockT,
	Balance: Codec
		+ MaybeDisplay
		+ 'static
		+ Debug
		+ Into<u128>
		+ AtLeast32BitUnsigned
		+ Copy
		+ From<u128>,
	R: RelayChainInterface + Clone,
	P: Send + Sync + 'static + ProvideRuntimeApi<Block> + UsageProvider<Block>,
	P::Api: AuraApi<Block, AuthorityId>,
{
	follow_relay_chain::<P, R, Block, ExPool, Balance>(
		para_id,
		parachain,
		relay_chain,
		keystore,
		transaction_pool,
	);
}

async fn follow_relay_chain<P, R, Block, ExPool, Balance>(
	para_id: ParaId,
	parachain: Arc<P>,
	relay_chain: R,
	keystore: KeystorePtr,
	transaction_pool: Arc<ExPool>,
) where
	Block: BlockT,
	Balance: Codec
		+ MaybeDisplay
		+ 'static
		+ Debug
		+ Into<u128>
		+ AtLeast32BitUnsigned
		+ Copy
		+ From<u128>,
	R: RelayChainInterface + Clone,
	P: Send + Sync + 'static + ProvideRuntimeApi<Block> + UsageProvider<Block>,
	P::Api: AuraApi<Block, AuthorityId>,
{
	let new_best_heads = match new_best_heads(relay_chain.clone(), para_id).await {
		Ok(best_heads_stream) => best_heads_stream.fuse(),
		Err(_err) => {
			return;
		},
	};

	pin_mut!(new_best_heads);
	loop {
		select! {
			h = new_best_heads.next() => {
				match h {
					Some((height, head, hash)) => {
						let _ = handle_relaychain_stream::<P, Block, ExPool, Balance>(
							head,
							height,
							&*parachain,
							keystore.clone(),
							transaction_pool.clone(),
							relay_chain.clone(),
							hash,
							para_id,
						).await;
					},
					None => {
						return;
					}
				}
			},
		}
	}
}

/// Order placement logic
async fn handle_relaychain_stream<P, Block, ExPool, Balance>(
	validation_data: PersistedValidationData,
	height: RelayBlockNumber,
	parachain: &P,
	keystore: KeystorePtr,
	transaction_pool: Arc<ExPool>,
	relay_chain: impl RelayChainInterface + Clone,
	p_hash: H256,
	para_id: ParaId,
) -> Result<(), Box<dyn Error>>
where
	Block: BlockT,
	Balance: Codec
		+ MaybeDisplay
		+ 'static
		+ Debug
		+ Into<u128>
		+ AtLeast32BitUnsigned
		+ Copy
		+ From<u128>,
	P: ProvideRuntimeApi<Block> + UsageProvider<Block>,
	P::Api: AuraApi<Block, AuthorityId>, /* 
	                                      * + OrderRuntimeApi<Block, Balance> <- TODO
	                                      * + TransactionPaymentApi<Block, Balance> <- TODO
	                                      * + OnRelayChainApi<Block>, <- TODO */
{
	let is_parathread = true; // TODO: check from the relay chain.

	if !is_parathread {
		// TODO: is there anything that should be done?
		//
		// Probably clear on-chain state regarding on-demand orders.
		return Ok(())
	}

	let is_on_demand_supported = true; // TODO: check from the relay chain.

	if !is_on_demand_supported {
		// TODO: probably add some logs
		return Ok(())
	}

	let head_encoded = validation_data.clone().parent_head.0;
	let head = <<Block as BlockT>::Header>::decode(&mut &head_encoded[..])?;

	let head_hash = head.hash();
	let authorities = parachain.runtime_api().authorities(head_hash).map_err(Box::new)?;

	Ok(())
}

async fn new_best_heads(
	relay_chain: impl RelayChainInterface + Clone,
	para_id: ParaId,
) -> RelayChainResult<impl Stream<Item = (u32, PersistedValidationData, H256)>> {
	let new_best_notification_stream =
		relay_chain.new_best_notification_stream().await?.filter_map(move |n| {
			let relay_chain = relay_chain.clone();
			async move {
				let relay_head: PersistedValidationData = relay_chain
					.persisted_validation_data(n.hash(), para_id, OccupiedCoreAssumption::TimedOut)
					.await
					.map(|s| s.map(|s| s))
					.ok()
					.flatten()?;
				Some((n.number, relay_head, n.hash()))
			}
		});

	Ok(new_best_notification_stream)
}
