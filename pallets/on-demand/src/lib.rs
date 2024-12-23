//! Pallet for managing the on-demand configuration.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use frame::prelude::*;
use frame_system::WeightInfo;

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame::pallet]
pub mod pallet {
	use super::*;

	/// The module configuration trait.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The admin origin for managing the on-demand configuration.
		type AdminOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		/// Block number type.
		type BlockNumber: Parameter + Member + Default + MaybeSerializeDeserialize + MaxEncodedLen;

		/// Given that we want to keep this pallet as generic as possible, we don't assume the type
		/// of the threshold.
		///
		/// We are adding this for implementations that have some kind of threshold and want it to
		/// be stored within the runtime.
		///
		/// For example, this threshold could represent the total weight of all the ready
		/// transactions from the pool, or their total fees.
		///
		/// NOTE: If there isn't a threshold parameter, this can simply be set to `()`.
		type ThresholdParameter: Member
			+ Parameter
			+ Default
			+ MaybeSerializeDeserialize
			+ MaxEncodedLen;

		/// Weight Info
		type WeightInfo: WeightInfo;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn slot_width)]
	pub type SlotWidth<T: Config> = StorageValue<_, T::BlockNumber, ValueQuery>;

	/// The threshold parameter stored in the runtime state.
	#[pallet::storage]
	#[pallet::getter(fn threshold_parameter)]
	pub type ThresholdParameter<T: Config> = StorageValue<_, T::ThresholdParameter, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Configuration of the coretime chain was set.
		SlotWidthSet { width: T::BlockNumber },
		/// Threshold parameter set.
		ThresholdParameterSet { parameter: T::ThresholdParameter },
	}

	#[pallet::error]
	#[derive(PartialEq)]
	pub enum Error<T> {}

	#[pallet::genesis_config]
	#[derive(DefaultNoBound)]
	pub struct GenesisConfig<T: Config> {
		/// Initial threshold parameter.
		pub threshold_parameter: T::ThresholdParameter,
	}

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {
			ThresholdParameter::<T>::set(self.threshold_parameter.clone());
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Set the slot width for on-demand blocks.
		///
		/// - `origin`: Must be Root or pass `AdminOrigin`.
		/// - `width`: The slot width in relay chain blocks.
		#[pallet::call_index(0)]
		#[pallet::weight(10_000)]
		pub fn set_slot_width(origin: OriginFor<T>, width: T::BlockNumber) -> DispatchResult {
			T::AdminOrigin::ensure_origin_or_root(origin)?;

			SlotWidth::<T>::set(width.clone());
			Self::deposit_event(Event::SlotWidthSet { width });

			Ok(())
		}

		/// Set the threshold parameter.
		///
		/// - `origin`: Must be Root or pass `AdminOrigin`.
		/// - `parameter`: The threshold parameter.
		#[pallet::call_index(1)]
		#[pallet::weight(10_000)]
		pub fn set_threshold_parameter(
			origin: OriginFor<T>,
			parameter: T::ThresholdParameter,
		) -> DispatchResult {
			T::AdminOrigin::ensure_origin_or_root(origin)?;

			ThresholdParameter::<T>::set(parameter.clone());
			Self::deposit_event(Event::ThresholdParameterSet { parameter });

			Ok(())
		}
	}
}
