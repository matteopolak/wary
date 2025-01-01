pub mod rule;
pub mod util;

use core::fmt;
use std::borrow::Cow;

#[cfg(feature = "derive")]
pub use wary_derive::*;

pub struct Transcript;

#[derive(thiserror::Error)]
#[non_exhaustive]
pub enum Error {
	#[error("value is not alphanumeric")]
	Alphanumeric,
	#[error("value is not ascii")]
	Ascii,
	#[error("value does not contain {0}")]
	Contains(Box<dyn std::fmt::Display>),
	#[cfg(feature = "email")]
	#[error(transparent)]
	Email(#[from] email_address::Error),
	#[cfg(feature = "url")]
	#[error(transparent)]
	Url(#[from] url::ParseError),
	#[error(transparent)]
	Length(#[from] rule::length::LengthError),
	#[error(transparent)]
	Range(#[from] rule::range::RangeError),
	#[cfg(feature = "semver")]
	#[error(transparent)]
	Semver(#[from] semver::Error),
	#[error("{0}")]
	Custom(Cow<'static, str>),
}

impl fmt::Debug for Error {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Alphanumeric => f.debug_struct("Alphanumeric").finish(),
			Self::Ascii => f.debug_struct("Ascii").finish(),
			Self::Contains(value) => f.debug_tuple("Contains").field(&value.to_string()).finish(),
			#[cfg(feature = "email")]
			Self::Email(err) => f.debug_tuple("Email").field(err).finish(),
			#[cfg(feature = "url")]
			Self::Url(err) => f.debug_tuple("Url").field(err).finish(),
			Self::Length(err) => f.debug_tuple("Length").field(err).finish(),
			Self::Range(err) => f.debug_tuple("Range").field(err).finish(),
			#[cfg(feature = "semver")]
			Self::Semver(err) => f.debug_tuple("Semver").field(err).finish(),
			Self::Custom(err) => f.debug_tuple("Custom").field(err).finish(),
		}
	}
}

pub trait Validate {
	type Context;

	fn validate(&self, ctx: &Self::Context) -> Result<(), Error>;
}

impl<T> Validate for Option<T>
where
	T: Validate,
{
	type Context = T::Context;

	fn validate(&self, ctx: &Self::Context) -> Result<(), Error> {
		if let Some(inner) = self {
			inner.validate(ctx)
		} else {
			Ok(())
		}
	}
}

impl<T> Validate for &T
where
	T: Validate,
{
	type Context = T::Context;

	fn validate(&self, ctx: &Self::Context) -> Result<(), Error> {
		(*self).validate(ctx)
	}
}
