use core::cmp::Ordering;

use crate::toolbox::rule::*;

#[doc(hidden)]
pub type Rule<Min, Max> = TimeRule<Min, Max>;

#[derive(Debug, thiserror::Error, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize), serde(untagged))]
pub enum Error {
	#[error("time is too old")]
	TooOld,
	#[error("time is too new")]
	TooNew,
}

impl Error {
	#[must_use]
	pub(crate) fn code(&self) -> &'static str {
		match self {
			Self::TooOld => "too_old",
			Self::TooNew => "too_new",
		}
	}

	pub(crate) fn message(&self) -> &'static str {
		match self {
			Self::TooOld => "time is too old",
			Self::TooNew => "time is too new",
		}
	}
}

/// Rule for checking if a time is within a range.
///
/// # Example
///
/// ```
/// use wary::{Wary, Validate};
///
/// static EPOCH_CHRONO: chrono::DateTime<chrono::Utc> = chrono::DateTime::from_timestamp_millis(1_746_333_380_000).unwrap();
/// static EPOCH_JIFF: jiff::civil::DateTime = jiff::civil::datetime(2025, 5, 4, 4, 36, 20, 0);
///
/// #[derive(Wary)]
/// struct Time {
///  #[validate(time(after = EPOCH_CHRONO))]
///  time: chrono::DateTime<chrono::Utc>,
///  #[validate(time(after = EPOCH_JIFF))]
///  time_jiff: jiff::civil::DateTime,
/// }
/// ```
#[must_use]
pub struct TimeRule<Min, Max> {
	min: Option<Min>,
	max: Option<Max>,
	exclusive_min: bool,
	exclusive_max: bool,
}

impl TimeRule<Unset, Unset> {
	#[inline]
	pub const fn new() -> Self {
		TimeRule {
			min: None,
			max: None,
			exclusive_min: false,
			exclusive_max: false,
		}
	}
}

impl<Max> TimeRule<Unset, Max> {
	/// After a specific time.
	#[inline]
	pub fn after<Min>(self, after: Min) -> TimeRule<Min, Max> {
		self.min(after)
	}

	/// Set the minimum value (inclusive).
	#[inline]
	pub fn min<Min>(self, min: Min) -> TimeRule<Min, Max> {
		TimeRule {
			min: Some(min),
			max: self.max,
			exclusive_min: false,
			exclusive_max: self.exclusive_max,
		}
	}

	/// Set the minimum value (exclusive).
	#[inline]
	pub fn exclusive_min<Min>(self, min: Min) -> TimeRule<Min, Max> {
		TimeRule {
			min: Some(min),
			max: self.max,
			exclusive_min: true,
			exclusive_max: self.exclusive_max,
		}
	}
}

impl<Min> TimeRule<Min, Unset> {
	/// Before a specific time.
	#[inline]
	pub fn before<Max>(self, before: Max) -> TimeRule<Min, Max> {
		self.max(before)
	}

	/// Set the maximum value (inclusive).
	#[inline]
	pub fn max<Max>(self, max: Max) -> TimeRule<Min, Max> {
		TimeRule {
			min: self.min,
			max: Some(max),
			exclusive_min: self.exclusive_min,
			exclusive_max: false,
		}
	}

	/// Set the maximum value (exclusive).
	#[inline]
	pub fn exclusive_max<Max>(self, max: Max) -> TimeRule<Min, Max> {
		TimeRule {
			min: self.min,
			max: Some(max),
			exclusive_min: self.exclusive_min,
			exclusive_max: true,
		}
	}
}

macro_rules! impl_rule {
	($type:ty) => {
		impl crate::Rule<$type> for TimeRule<$type, $type> {
			type Context = ();

			#[inline]
			fn validate(&self, _ctx: &Self::Context, item: &$type) -> Result<()> {
				if let Some(min) = &self.min {
					match item.cmp(min) {
						Ordering::Greater => {}
						Ordering::Equal if !self.exclusive_min => {}
						_ => return Err(Error::TooNew.into()),
					}
				}

				if let Some(max) = &self.max {
					match item.cmp(max) {
						Ordering::Less => {}
						Ordering::Equal if !self.exclusive_max => {}
						_ => return Err(Error::TooOld.into()),
					}
				}

				Ok(())
			}
		}

		impl crate::Rule<$type> for TimeRule<$type, Unset> {
			type Context = ();

			#[inline]
			fn validate(&self, _ctx: &Self::Context, item: &$type) -> Result<()> {
				if let Some(min) = &self.min {
					match item.cmp(min) {
						Ordering::Greater => {}
						Ordering::Equal if !self.exclusive_min => {}
						_ => return Err(Error::TooNew.into()),
					}
				}

				Ok(())
			}
		}

		impl crate::Rule<$type> for TimeRule<Unset, $type> {
			type Context = ();

			#[inline]
			fn validate(&self, _ctx: &Self::Context, item: &$type) -> Result<()> {
				if let Some(max) = &self.max {
					match item.cmp(max) {
						Ordering::Less => {}
						Ordering::Equal if !self.exclusive_max => {}
						_ => return Err(Error::TooOld.into()),
					}
				}

				Ok(())
			}
		}
	};
}

#[cfg(feature = "jiff")]
mod jiff_ {
	use super::*;

	impl_rule!(jiff::Zoned);
	impl_rule!(jiff::civil::DateTime);
	impl_rule!(jiff::civil::Date);
	impl_rule!(jiff::civil::Time);
}

#[cfg(feature = "chrono")]
mod chrono_ {
	use chrono::DateTime;

	use super::*;

	impl<Tz: chrono::TimeZone> crate::Rule<DateTime<Tz>> for TimeRule<DateTime<Tz>, DateTime<Tz>> {
		type Context = ();

		#[inline]
		fn validate(&self, _ctx: &Self::Context, item: &DateTime<Tz>) -> Result<()> {
			if let Some(min) = &self.min {
				match item.cmp(min) {
					Ordering::Greater => {}
					Ordering::Equal if !self.exclusive_min => {}
					_ => return Err(Error::TooNew.into()),
				}
			}

			if let Some(max) = &self.max {
				match item.cmp(max) {
					Ordering::Less => {}
					Ordering::Equal if !self.exclusive_max => {}
					_ => return Err(Error::TooOld.into()),
				}
			}

			Ok(())
		}
	}

	impl<Tz: chrono::TimeZone> crate::Rule<DateTime<Tz>> for TimeRule<DateTime<Tz>, Unset> {
		type Context = ();

		#[inline]
		fn validate(&self, _ctx: &Self::Context, item: &DateTime<Tz>) -> Result<()> {
			if let Some(min) = &self.min {
				match item.cmp(min) {
					Ordering::Greater => {}
					Ordering::Equal if !self.exclusive_min => {}
					_ => return Err(Error::TooNew.into()),
				}
			}

			Ok(())
		}
	}

	impl<Tz: chrono::TimeZone> crate::Rule<DateTime<Tz>> for TimeRule<Unset, DateTime<Tz>> {
		type Context = ();

		#[inline]
		fn validate(&self, _ctx: &Self::Context, item: &DateTime<Tz>) -> Result<()> {
			if let Some(max) = &self.max {
				match item.cmp(max) {
					Ordering::Less => {}
					Ordering::Equal if !self.exclusive_max => {}
					_ => return Err(Error::TooOld.into()),
				}
			}

			Ok(())
		}
	}

	impl_rule!(chrono::NaiveDateTime);
	impl_rule!(chrono::NaiveDate);
	impl_rule!(chrono::NaiveTime);
}
