#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Codec, Decode, Encode, MaxEncodedLen};
use cumulus_primitives_core::ParaId;
use frame_support::Parameter;
use sp_runtime::traits::{MaybeDisplay, MaybeSerializeDeserialize, Member};

pub mod well_known_keys;

#[derive(Encode, Decode, Debug, PartialEq, Clone)]
pub struct EnqueuedOrder {
	/// Parachain ID
	pub para_id: ParaId,
}

pub trait ThresholdParameterT:
	Parameter + Member + Default + MaybeSerializeDeserialize + MaxEncodedLen
{
}

impl<T> ThresholdParameterT for T where
	T: Parameter + Member + Default + MaybeSerializeDeserialize + MaxEncodedLen
{
}

sp_api::decl_runtime_apis! {
	#[api_version(2)]
	pub trait OnDemandRuntimeApi<Balance, BlockNumber, ThresholdParameter> where
		Balance: Codec + MaybeDisplay,
		BlockNumber: Codec + From<u32>,
		ThresholdParameter: ThresholdParameterT,
	{
		/// Order placement slot width.
		fn slot_width()-> BlockNumber;

		/// Runtime configured order placement threshold parameter.
		fn threshold_parameter() -> ThresholdParameter;
	}
}
