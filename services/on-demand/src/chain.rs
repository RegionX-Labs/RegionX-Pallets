//! This file contains all the chain related interaction functions.

use crate::chain::polkadot::runtime_types::polkadot_parachain_primitives::primitives::Id;
use cumulus_primitives_core::ParaId;
use sp_application_crypto::AppCrypto;
use sp_core::ByteArray;
use sp_keystore::KeystorePtr;
use sp_runtime::{
	traits::{IdentifyAccount, Verify},
	MultiSignature as SpMultiSignature,
};
use std::error::Error;
use subxt::{tx::Signer, utils::MultiSignature, Config, OnlineClient, PolkadotConfig};

#[subxt::subxt(runtime_metadata_path = "../../artifacts/metadata.scale")]
pub mod polkadot {}

/// Submits order to an rpc node.
pub async fn submit_order(
	url: &str,
	para_id: ParaId,
	max_amount: u128,
	keystore: KeystorePtr,
) -> Result<(), Box<dyn Error>> {
	let client = OnlineClient::<PolkadotConfig>::from_url(url).await.unwrap(); // TODO

	let place_order = polkadot::tx()
		.on_demand()
		.place_order_allow_death(max_amount, Id(para_id.into()));

	// let submit_result = client.tx().sign_and_submit_default(&place_order,
	// &signer_keystore).await; log::info!("submit_result:{:?}", submit_result);
	// submit_result.map_err(|_e| "TODO".into())?;

	Ok(())
}
