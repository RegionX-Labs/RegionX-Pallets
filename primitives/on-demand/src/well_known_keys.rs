use cumulus_primitives_core::ParaId;
use sp_io::hashing::twox_64;
use sp_runtime::Vec;
use codec::Encode;

/// OnDemandAssignmentProvider OnDemandQueue
pub const ON_DEMAND_QUEUE: &[u8] =
	&hex_literal::hex!["8f32430b49607f8d60bfd3a003ddf4b53f35b69d817556cf6b886e5b4f01fbdc"];

/// OnDemandAssignmentProvider SpotTraffic
pub const SPOT_TRAFFIC: &[u8] =
	&hex_literal::hex!["8f32430b49607f8d60bfd3a003ddf4b5c9308a8e0e640735727536bd9069b11e"];

/// Configuration ActiveConfig
pub const ACTIVE_CONFIG: &[u8] =
	&hex_literal::hex!["06de3d8a54d27e44a9d5ce189618f22db4b49d95320d9021994c850f25b8e385"];

pub const PARAS_PARA_LIFECYCLES: &[u8] =
	&hex_literal::hex!["cd710b30bd2eab0352ddcc26417aa194281e0bfde17b36573208a06cb5cfba6b"];

/// Returns the storage key for accessing the parachain lifecycle.
pub fn para_lifecycle(para_id: ParaId) -> Vec<u8> {
	para_id.using_encoded(|para_id: &[u8]| {
		PARAS_PARA_LIFECYCLES
			.iter()
			.chain(twox_64(para_id).iter())
			.chain(para_id.iter())
			.cloned()
			.collect()
	})
}
