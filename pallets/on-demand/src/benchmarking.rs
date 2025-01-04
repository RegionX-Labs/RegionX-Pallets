//! Benchmarks for pallet-on-demand

use super::*;

pub trait BenchmarkHelper<ThresholdParameter> {
	// Return a mock threshold parameter that is not the default value.
	fn mock_threshold_parameter() -> ThresholdParameter;
}

use frame_benchmarking::v2::*;
fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

#[benchmarks]
mod benchmarks {
	use super::*;
	use frame_support::traits::EnsureOrigin;

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
