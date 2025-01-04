#![allow(clippy::should_implement_trait)]

use core::fmt;

pub mod rule {
	pub use super::{custom, func, lowercase, uppercase};

	pub mod ascii;
	/// Validates that a string-like value is an email address.
	///
	/// It is recommended to instead parse directly into an
	/// [`EmailAddress`][email] if you need to parse it afterwards anyway. Other
	/// validators that accept string-like values such as [`ascii`][ascii],
	/// [`length`][length], [`contains`][contains], etc. can still be used with
	/// an [`EmailAddress`][email]!
	///
	/// [ascii]: crate::rule::ascii
	/// [contains]: crate::rule::contains
	/// [length]: crate::rule::length
	///
	/// [email]: email_address::EmailAddress
	#[cfg(feature = "email")]
	pub mod email;
	pub mod length;
	pub use length as len;
	pub mod alphanumeric;
	pub mod range;
	#[cfg(feature = "url")]
	pub mod url;
	pub use alphanumeric as alnum;
	pub mod addr;
	pub mod contains;
	pub mod equals;
	pub mod inner;
	pub use equals as eq;
	pub mod prefix;
	#[cfg(feature = "regex")]
	pub mod regex;
	pub mod required;
	#[cfg(feature = "semver")]
	pub mod semver;
	pub mod suffix;

	pub mod dive {}
}

pub mod modifier {
	pub use super::{custom, func, lowercase, uppercase};

	pub mod inner;
}

// both rule and modifier
pub mod lowercase;
pub mod uppercase;

// for auto-complete
pub mod func {}
pub mod custom {}

pub struct Unset;

pub(crate) struct DebugDisplay<T>(pub T);

impl<T> fmt::Display for DebugDisplay<T>
where
	T: fmt::Debug,
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		self.0.fmt(f)
	}
}
