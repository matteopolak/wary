#![allow(clippy::should_implement_trait)]

use core::fmt;

#[cfg(feature = "alloc")]
use crate::alloc::{format, string::String};

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
	#[cfg(feature = "credit_card")]
	pub mod credit_card;
	pub mod prefix;
	#[cfg(feature = "regex")]
	pub mod regex;
	pub mod required;
	#[cfg(feature = "semver")]
	pub mod semver;
	pub mod suffix;
	#[cfg(feature = "uuid")]
	pub mod uuid;

	pub mod and;
	pub mod custom;
	pub mod dive;
	pub mod func;
	pub mod inner;
	pub mod or;
}

pub mod transformer {
	pub use super::{lowercase, uppercase};

	#[cfg(feature = "alloc")]
	pub mod trim;

	pub mod custom;
	pub mod dive;
	pub mod func;
	pub mod inner;
}

// both rule and modifier
pub mod lowercase;
pub mod uppercase;

pub struct Unset;

pub(crate) struct DebugDisplay<T>(pub T);

#[cfg(feature = "alloc")]
pub(crate) type ItemSlice = String;
#[cfg(not(feature = "alloc"))]
pub(crate) type ItemSlice = ();

impl<T> DebugDisplay<T>
where
	T: fmt::Debug,
{
	#[allow(clippy::inherent_to_string)]
	#[cfg(feature = "alloc")]
	fn to_string(&self) -> String {
		format!("{:?}", self.0)
	}

	#[cfg(not(feature = "alloc"))]
	fn to_string(&self) {}
}
