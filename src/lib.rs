pub mod rule;
pub mod util;

#[cfg(feature = "derive")]
pub use wary_derive::*;

pub struct Transcript;

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
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
	#[error("{0}")]
	Custom(String),
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

pub struct OrRule<T, U> {
	left: T,
	right: U,
}

impl<T, U> OrRule<T, U> {
	pub fn new(left: T, right: U) -> Self {
		Self { left, right }
	}
}

impl<T, U> Validate for OrRule<T, U>
where
	T: Validate,
	U: Validate,
{
	type Context = (T::Context, U::Context);

	fn validate(&self, ctx: &Self::Context) -> Result<(), Error> {
		self
			.left
			.validate(&ctx.0)
			.or_else(|_| self.right.validate(&ctx.1))
	}
}
