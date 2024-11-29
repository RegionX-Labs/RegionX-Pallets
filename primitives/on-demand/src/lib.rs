use codec::Codec;
use sp_runtime::traits::MaybeDisplay;

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
