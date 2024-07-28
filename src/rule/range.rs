use std::cmp::Ordering;

use crate::{Error, Validate};

#[derive(Debug, thiserror::Error)]
pub enum RangeError {
	#[error("Value is too small")]
	TooSmall,
	#[error("Value is too large")]
	TooLarge,
}

pub struct RangeRule<T, B> {
	min: Option<B>,
	exclusive_min: bool,
	max: Option<B>,
	exclusive_max: bool,
	inner: T,
}

impl<T, B> RangeRule<T, B> {
	pub fn new(inner: T) -> Self {
		Self {
			min: None,
			max: None,
			exclusive_min: false,
			exclusive_max: false,
			inner,
		}
	}

	pub fn min(mut self, min: B) -> Self {
		self.min = Some(min);
		self
	}

	pub fn max(mut self, max: B) -> Self {
		self.max = Some(max);
		self
	}

	pub fn exclusive_min(mut self, min: B) -> Self {
		self.min = Some(min);
		self.exclusive_min = true;
		self
	}

	pub fn exclusive_max(mut self, max: B) -> Self {
		self.max = Some(max);
		self.exclusive_max = true;
		self
	}
}

impl<T, B> Validate for RangeRule<T, B>
where
	T: PartialOrd<B>,
{
	type Context = ();

	fn validate(&self, _ctx: &Self::Context) -> Result<(), Error> {
		if let Some(ref min) = self.min {
			match self.inner.partial_cmp(min) {
				Some(Ordering::Greater) => {}
				Some(Ordering::Equal) if !self.exclusive_min => {}
				_ => return Err(RangeError::TooSmall.into()),
			}
		}

		if let Some(ref max) = self.max {
			match self.inner.partial_cmp(max) {
				Some(Ordering::Less) => {}
				Some(Ordering::Equal) if !self.exclusive_max => {}
				_ => return Err(RangeError::TooLarge.into()),
			}
		}

		Ok(())
	}
}
