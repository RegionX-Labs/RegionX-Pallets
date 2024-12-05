//! This file contains all the chain related interaction functions.

use crate::chain::polkadot::runtime_types::polkadot_parachain_primitives::primitives::Id;
use cumulus_primitives_core::ParaId;
use sp_application_crypto::AppCrypto;
use sp_core::{H256, ByteArray};
use sp_keystore::KeystorePtr;
use sp_runtime::{
	traits::{IdentifyAccount, Verify},
	MultiSignature as SpMultiSignature,
};
use std::error::Error;
use subxt::{tx::Signer, utils::MultiSignature, Config, OnlineClient, PolkadotConfig};
use cumulus_relay_chain_interface::RelayChainInterface;
use on_demand_primitives::well_known_keys::para_lifecycle;
use polkadot_runtime_parachains::ParaLifecycle;
use codec::Decode;

#[subxt::subxt(runtime_metadata_path = "../../artifacts/metadata.scale")]
pub mod polkadot {}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Signature(pub [u8; 64]);

impl From<Signature> for MultiSignature {
	fn from(value: Signature) -> Self {
		MultiSignature::Sr25519(value.0)
	}
}

pub struct SignerKeystore<T: Config> {
	/// Account ID
	account_id: T::AccountId,
	/// Keystore of node
	keystore: KeystorePtr,
}

impl<T> SignerKeystore<T>
where
	T: Config,
	T::AccountId: From<[u8; 32]>,
{
	pub fn new(keystore: KeystorePtr) -> Self {
		let pub_key =
			keystore.sr25519_public_keys(sp_consensus_aura::sr25519::AuthorityPair::ID)[0];

		let binding = <SpMultiSignature as Verify>::Signer::from(pub_key).into_account().clone();

		let account_id = binding.as_slice();
		let mut r = [0u8; 32];
		r.copy_from_slice(account_id);
		let acc = T::AccountId::try_from(r).ok().unwrap();
		Self { account_id: acc.clone(), keystore }
	}
}
impl<T> Signer<T> for SignerKeystore<T>
where
	T: Config,
	T::AccountId: From<[u8; 32]>,
	T::Signature: From<Signature>,
{
	fn account_id(&self) -> T::AccountId {
		self.account_id.clone()
	}

	fn address(&self) -> T::Address {
		self.account_id.clone().into()
	}

	/// Use aura's key to sign
	fn sign(&self, signer_payload: &[u8]) -> T::Signature {
		let pub_key =
			self.keystore.sr25519_public_keys(sp_consensus_aura::sr25519::AuthorityPair::ID)[0];

		let signature = self
			.keystore
			.sr25519_sign(sp_consensus_aura::sr25519::AuthorityPair::ID, &pub_key, signer_payload)
			.unwrap()
			.unwrap();

		Signature(signature.0).into()
	}
}

/// Submits order to an rpc node.
pub async fn submit_order(
	url: &str,
	para_id: ParaId,
	max_amount: u128,
	keystore: KeystorePtr,
) -> Result<(), Box<dyn Error>> {
	let client = OnlineClient::<PolkadotConfig>::from_url(url).await.unwrap(); // TODO

	let place_order = polkadot::tx()
		.on_demand_assignment_provider()
		.place_order_allow_death(max_amount, Id(para_id.into()));

	let signer_keystore = SignerKeystore::<PolkadotConfig>::new(keystore.clone());

	let submit_result = client.tx().sign_and_submit_default(&place_order, &signer_keystore).await;
	// log::info!("submit_result:{:?}", submit_result);
	// submit_result.unwrap(); // TODO

	Ok(())
}

/// Is this a parathread?
pub async fn is_parathread(
	relay_chain: &(impl RelayChainInterface + Clone),
	p_hash: H256,
	para_id: ParaId,
) -> Result<bool, Box<dyn Error>> {
	let para_lifecycle_storage = relay_chain
		.get_storage_by_key(p_hash, para_lifecycle(para_id).as_slice())
		.await?;
	let para_lifecycle = para_lifecycle_storage
		.map(|raw| <ParaLifecycle>::decode(&mut &raw[..]))
		.transpose()?;

	let is_parathread = para_lifecycle == Some(ParaLifecycle::Parathread);
	Ok(is_parathread)
}
