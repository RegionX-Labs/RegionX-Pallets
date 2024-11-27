use sc_service::TaskManager;
use cumulus_primitives_core::{
	relay_chain::BlockNumber as RelayBlockNumber, ParaId, PersistedValidationData,
};
use cumulus_relay_chain_interface::{RelayChainInterface, RelayChainResult};
use futures::{Stream, StreamExt};
use polkadot_primitives::OccupiedCoreAssumption;
use sp_core::H256;

/// Start all the on-demand order creation related tasks.
pub async fn start_on_demand<T, R, ExPool, Block, Balance>(
	relay_chain: R,
	task_manager: &TaskManager,
) -> sc_service::error::Result<()>
where
	R: RelayChainInterface + Clone + 'static,
{
	let on_demand_task = run_on_demand_task(relay_chain);

	// TODO: spawn_blocking?
	task_manager.spawn_essential_handle().spawn_blocking(
		"on-demand order placement task",
		"on-demand",
		on_demand_task,
	);

	Ok(())
}

async fn run_on_demand_task<R>(relay_chain: R)
where
	R: RelayChainInterface + Clone,
{
	follow_relay_chain(relay_chain);
}

async fn follow_relay_chain<R>(relay_chain: R)
where
	R: RelayChainInterface + Clone,
{
	// TODO: follow heads
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
