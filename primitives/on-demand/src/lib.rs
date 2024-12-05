#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Codec, Encode, Decode};
use sp_runtime::traits::MaybeDisplay;
use cumulus_primitives_core::ParaId;

pub mod well_known_keys;

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
