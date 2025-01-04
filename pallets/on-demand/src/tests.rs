use crate::{
	mock::{new_test_ext, OnDemand, RuntimeOrigin, System, Test},
	Event, ThresholdParameter,
};
use frame_support::{assert_noop, assert_ok};
use sp_runtime::traits::BadOrigin;

#[test]
fn set_threshold_parameter_works() {
	new_test_ext().execute_with(|| {
		assert_eq!(ThresholdParameter::<Test>::get(), 0);

		// Failure: Bad origin
		assert_noop!(OnDemand::set_threshold_parameter(RuntimeOrigin::signed(1), 1_000), BadOrigin);

		// Should be working fine
		assert_ok!(OnDemand::set_threshold_parameter(RuntimeOrigin::root(), 1_000));

		// Check the storage item
		assert_eq!(ThresholdParameter::<Test>::get(), 1_000);

		// Check the emitted events
		System::assert_last_event(Event::ThresholdParameterSet { parameter: 1_000 }.into());
	})
}
