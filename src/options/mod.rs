#![allow(clippy::should_implement_trait)]

use core::fmt;

pub mod rule {
	pub use super::{lowercase, uppercase};

	pub mod ascii;
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
	pub use equals as eq;
	pub mod prefix;
	#[cfg(feature = "regex")]
	pub mod regex;
	pub mod required;
	#[cfg(feature = "semver")]
	pub mod semver;
	pub mod suffix;

	pub mod custom;
	pub mod dive;
	pub mod func;
	pub mod inner;
}

pub mod modifier {
	pub use super::{lowercase, uppercase};

	pub mod custom;
	pub mod func;
	pub mod inner;
}

// both rule and modifier
pub mod lowercase;
pub mod uppercase;

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
