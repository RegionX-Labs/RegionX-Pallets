//! Benchmarks for pallet-on-demand

#![cfg(feature = "runtime-benchmarks")]

use super::*;

use frame_benchmarking::v2::*;
fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

#[benchmarks]
mod benchmarks {
	use super::*;
	use frame_support::traits::EnsureOrigin;

	#[benchmark]
	fn set_slot_width() -> Result<(), BenchmarkError> {
		let origin =
			T::AdminOrigin::try_successful_origin().map_err(|_| BenchmarkError::Weightless)?;

		#[extrinsic_call]
		_(origin as T::RuntimeOrigin, 1u32.into());

		assert_last_event::<T>(Event::SlotWidthSet { width: 1u32.into() }.into());
		Ok(())
	}

    #[benchmark]
	fn set_threshold_parameter() -> Result<(), BenchmarkError> {
		let origin =
			T::AdminOrigin::try_successful_origin().map_err(|_| BenchmarkError::Weightless)?;

        let param = T::BenchmarkHelper::mock_threshold_parameter();

		#[extrinsic_call]
		_(origin as T::RuntimeOrigin, param.clone());

		assert_last_event::<T>(Event::ThresholdParameterSet { parameter: param }.into());
		Ok(())
	}

	impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
