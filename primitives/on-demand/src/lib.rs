use codec::{Codec, Encode, Decode};
use sp_runtime::traits::MaybeDisplay;
use cumulus_primitives_core::ParaId;

/// OnDemandAssignmentProvider OnDemandQueue
pub const ON_DEMAND_QUEUE: &[u8] =
	&hex_literal::hex!["8f32430b49607f8d60bfd3a003ddf4b53f35b69d817556cf6b886e5b4f01fbdc"];

/// OnDemandAssignmentProvider SpotTraffic
pub const SPOT_TRAFFIC: &[u8] =
	&hex_literal::hex!["8f32430b49607f8d60bfd3a003ddf4b5c9308a8e0e640735727536bd9069b11e"];

/// Configuration ActiveConfig
pub const ACTIVE_CONFIG: &[u8] =
	&hex_literal::hex!["06de3d8a54d27e44a9d5ce189618f22db4b49d95320d9021994c850f25b8e385"];

#[derive(Encode, Decode, Debug, PartialEq, Clone)]
pub struct EnqueuedOrder {
	/// Parachain ID
	pub para_id: ParaId,
}

sp_api::decl_runtime_apis! {
	#[api_version(2)]
	pub trait OnDemandRuntimeApi<Balance, BlockNumber> where
		Balance: Codec + MaybeDisplay,
		BlockNumber: Codec + From<u32>,
	{
		/// Order placement slot width.
		fn slot_width()-> BlockNumber;
	}
}
