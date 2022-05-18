use std::ops::{Add, Div, Mul, Sub};

use bevy::prelude::*;

/// A Proportional Integral Derivative Controller for an object's position.
///
/// The controller is configured with a set of gains. The controller will attempt to maintain a
/// target value by adjusting the output of the controlled system.
///
/// see: https://en.wikipedia.org/wiki/PID_controller
#[derive(Component, Debug)]
pub struct PidControlledPosition {
	pub proportional_gain: f32,
	pub integral_gain: f32,
	pub derivative_gain: f32,

	target: Vec3,

	integration: Vec3,
	error_prev: Vec3,
	value_prev: Vec3,
	derivative_initialized: bool,
}

impl PidControlledPosition {
	pub fn new(proportional_gain: f32, integral_gain: f32, derivative_gain: f32) -> Self {
		Self {
			proportional_gain,
			integral_gain,
			derivative_gain,
			target: Vec3::new(0.0, 0.0, 0.0),
			integration: Vec3::new(0.0, 0.0, 0.0),
			error_prev: Vec3::new(0.0, 0.0, 0.0),
			value_prev: Vec3::new(0.0, 0.0, 0.0),
			derivative_initialized: false,
		}
	}

	pub fn compute(&mut self, delta_time: f32, current_value: Vec3) -> Vec3 {
		let error = self.target - current_value;
		let error_rate_of_change = (error - self.error_prev) / delta_time;
		let derive_measure = if self.derivative_initialized {
			error_rate_of_change
		} else {
			Vec3::new(0., 0., 0.)
		};
		self.integration += error * delta_time;
		let p = self.proportional_gain * error;
		let i = self.integral_gain * self.integration;
		let d = self.derivative_gain * derive_measure;
		let output = p + i + d;
		self.error_prev = error;
		self.value_prev = current_value;
		output
	}

	pub fn set_target(&mut self, target: Vec3) {
		self.target = target;
	}
}

pub fn system(
	time: Res<Time>,
	mut controllers: Query<(&mut PidControlledPosition, &mut Transform)>,
) {
	for (mut controller, mut transform) in controllers.iter_mut() {
		let output = controller.compute(time.delta_seconds(), transform.translation);
		transform.translation += output;
	}
}

/// A generalized Proportional Integral Derivative Controller for any type.
///
/// The controller is configured with a set of gains. The controller will attempt to maintain a
/// target value by adjusting the output of the controlled system.
///
/// see: https://en.wikipedia.org/wiki/PID_controller
#[derive(Component, Debug)]
pub struct PidControlled<T> {
	pub proportional_gain: f32,
	pub integral_gain: f32,
	pub derivative_gain: f32,

	target: T,

	integration: T,
	error_prev: T,
	value_prev: T,
	derivative_initialized: bool,
}

unsafe impl<T> Send for PidControlled<T> {}
unsafe impl<T> Sync for PidControlled<T> {}

impl<T> PidControlled<T> where
	T: Default + Copy + Sync + Send + Add<Output = T> + Sub<Output = T> + Mul<Output = T> + Mul<f32, Output = T> + Div<Output = T> + Div<f32, Output = T> {
	pub fn new(proportional_gain: f32, integral_gain: f32, derivative_gain: f32) -> Self {
		Self {
			proportional_gain,
			integral_gain,
			derivative_gain,
			target: Default::default(),
			integration: Default::default(),
			error_prev: Default::default(),
			value_prev: Default::default(),
			derivative_initialized: false,
		}
	}

	pub fn compute(&mut self, delta_time: f32, current_value: T) -> T {
		let error = self.target - current_value;
		let error_rate_of_change = (error - self.error_prev) / delta_time;
		let derive_measure = if self.derivative_initialized {
			error_rate_of_change
		} else {
			Default::default()
		};
		self.integration = self.integration + error * delta_time; // not using += so we can require less trait bounds
		let p = error * self.proportional_gain;
		let i = self.integration * self.integral_gain;
		let d = derive_measure * self.derivative_gain;
		let output = p + i + d;
		self.error_prev = error;
		self.value_prev = current_value;
		output
	}

	pub fn set_target(&mut self, target: T) {
		self.target = target;
	}
}

pub fn system_generic<T: 'static>(
	time: Res<Time>,
	mut controllers: Query<(&mut PidControlled<T>, &mut Transform)>,
) where T: Default + Copy + Sync + Send + Add<Output = T> + Sub<Output = T> + Mul<Output = T> + Mul<f32, Output = T> + Div<Output = T> + Div<f32, Output = T> {
	for (mut controller, mut transform) in controllers.iter_mut() {
		let output = controller.compute(time.delta_seconds(), transform.translation);
		transform.translation += output;
	}
}
