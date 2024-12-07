//! On-demand order placement service.
//!
//! NOTE: Inspiration was taken from the Magnet(https://github.com/Magport/Magnet) on-demand integration.

use crate::chain::{get_spot_price, is_parathread, on_demand_cores_available};
use codec::{Codec, Decode};
use cumulus_primitives_core::{
	relay_chain::BlockNumber as RelayBlockNumber, ParaId, PersistedValidationData,
};
use cumulus_relay_chain_interface::{RelayChainInterface, RelayChainResult};
use futures::{pin_mut, select, FutureExt, Stream, StreamExt};
use on_demand_primitives::{well_known_keys::ON_DEMAND_QUEUE, EnqueuedOrder, OnDemandRuntimeApi};
use polkadot_primitives::OccupiedCoreAssumption;
use sc_client_api::UsageProvider;
use sc_service::TaskManager;
use sc_transaction_pool_api::MaintainedTransactionPool;
use sp_api::ProvideRuntimeApi;
use sp_consensus_aura::{sr25519::AuthorityId, AuraApi};
use sp_core::{ByteArray, H256};
use sp_keystore::KeystorePtr;
use sp_runtime::traits::{AtLeast32BitUnsigned, Block as BlockT, Header, MaybeDisplay};
use std::{error::Error, fmt::Debug, net::SocketAddr, sync::Arc};

mod chain;

const LOG_TARGET: &str = "on-demand-service";

/// Start all the on-demand order creation related tasks.
pub fn start_on_demand<P, R, ExPool, Block, Balance>(
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
	P::Api: AuraApi<Block, AuthorityId> + OnDemandRuntimeApi<Block, Balance, RelayBlockNumber>,
	ExPool: MaintainedTransactionPool<Block = Block, Hash = <Block as BlockT>::Hash> + 'static,
{
	let mut url = String::from("ws://"); // <- TODO wss
	url.push_str(
		&relay_rpc
			.expect("RPC address required for submitting on-demand orders")
			.to_string(),
	);

	let on_demand_task = run_on_demand_task::<P, R, Block, ExPool, Balance>(
		para_id,
		parachain,
		relay_chain,
		keystore,
		transaction_pool,
		url,
	);

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
	relay_url: String,
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
	P::Api: AuraApi<Block, AuthorityId> + OnDemandRuntimeApi<Block, Balance, RelayBlockNumber>,
{
	log::info!(
		target: LOG_TARGET,
		"Starting on-demand task"
	);

	let relay_chain_notification = follow_relay_chain::<P, R, Block, ExPool, Balance>(
		para_id,
		parachain,
		relay_chain,
		keystore,
		transaction_pool,
		relay_url,
	);

	// let event_notification = event_notification(para_id, url, order_record);
	select! {
		_ = relay_chain_notification.fuse() => {},
		// _ = event_notification.fuse() => {},
	}
}

async fn follow_relay_chain<P, R, Block, ExPool, Balance>(
	para_id: ParaId,
	parachain: Arc<P>,
	relay_chain: R,
	keystore: KeystorePtr,
	transaction_pool: Arc<ExPool>,
	relay_url: String,
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
	P::Api: AuraApi<Block, AuthorityId> + OnDemandRuntimeApi<Block, Balance, RelayBlockNumber>,
{
	let new_best_heads = match new_best_heads(relay_chain.clone(), para_id).await {
		Ok(best_heads_stream) => best_heads_stream.fuse(),
		Err(_err) => {
			log::error!(
				target: LOG_TARGET,
				"Error: {:?}",
				_err
			);
			return;
		},
	};

	pin_mut!(new_best_heads);
	loop {
		select! {
			h = new_best_heads.next() => {
				match h {
					Some((height, head, hash)) => {
						log::info!(
							target: LOG_TARGET,
							"New best head: {}",
							hash
						);

						let _ = handle_relaychain_stream::<P, Block, ExPool, Balance>(
							head,
							height,
							&*parachain,
							keystore.clone(),
							transaction_pool.clone(),
							relay_chain.clone(),
							hash,
							para_id,
							relay_url.clone(),
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
	relay_url: String,
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
	P::Api: AuraApi<Block, AuthorityId> + OnDemandRuntimeApi<Block, Balance, RelayBlockNumber>,
{
	// let is_parathread = is_parathread(&relay_chain, p_hash, para_id).await?;
	let is_parathread = true; // TODO: remove, this is only for testing

	if !is_parathread {
		log::info!(
			target: LOG_TARGET,
			"Not a parathread, switching to bulk coretime",
		);
		// TODO: switch to bulk (is there actually anything to be done?)

		return Ok(())
	}

	let available = on_demand_cores_available(&relay_chain, p_hash, para_id)
		.await
		.ok_or("Failed to check if there are on-demand cores available")?;

	if available {
		log::info!(
			target: LOG_TARGET,
			"No cores allocated to on-demand"
		);

		return Ok(())
	}

	let head_encoded = validation_data.clone().parent_head.0;
	let head = <<Block as BlockT>::Header>::decode(&mut &head_encoded[..])?;

	let head_hash = head.hash();
	let authorities = parachain.runtime_api().authorities(head_hash).map_err(Box::new)?;
	let slot_width = parachain.runtime_api().slot_width(head_hash)?;

	// Taken from: https://github.com/paritytech/polkadot-sdk/issues/1487
	let indx = (height >> slot_width) % authorities.len() as u32;
	let expected_author = authorities.get(indx as usize).ok_or::<Box<dyn Error>>("TODO".into())?;

	if !keystore.has_keys(&[(expected_author.to_raw_vec(), sp_application_crypto::key_types::AURA)])
	{
		// Expected author is not in the keystore therefore we are not responsible for order
		// creation.
		log::info!(
			target: LOG_TARGET,
			"Waiting for {} to create an order",
			expected_author
		);
		return Ok(())
	}

	let on_demand_queue_storage = relay_chain.get_storage_by_key(p_hash, ON_DEMAND_QUEUE).await?;
	let on_demand_queue = on_demand_queue_storage
		.map(|raw| <Vec<EnqueuedOrder>>::decode(&mut &raw[..]))
		.transpose()?;

	let order_exists = if let Some(queue) = on_demand_queue {
		queue.into_iter().position(|e| e.para_id == para_id).is_some()
	} else {
		false
	};

	if order_exists {
		return Ok(())
	}

	// Before placing an order ensure that the criteria for placing an order has been reached.
	let order_criteria_reached = true; // TODO: this should be customizable

	if !order_criteria_reached {
		return Ok(())
	}

	let spot_price =
		get_spot_price::<Balance>(relay_chain, p_hash).await.unwrap_or(1_000u32.into()); // TODO

	log::info!(
		target: LOG_TARGET,
		"Placing an order",
	);

	chain::submit_order(&relay_url, para_id, spot_price.into(), keystore).await?;

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
