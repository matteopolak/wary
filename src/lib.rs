#![warn(
	clippy::pedantic,
	clippy::print_stdout,
	clippy::print_stderr,
	clippy::panic
)]
#![allow(clippy::new_without_default, clippy::wildcard_imports)]
#![cfg_attr(test, allow(non_upper_case_globals))]
#![doc = include_str!("../README.md")]

pub mod error;
pub mod options;

use error::Path;
pub use error::{Error, Report};
#[cfg(feature = "derive")]
pub use wary_derive::*;

pub mod toolbox {
	//! A collection of common imports for various use-cases.

	pub mod rule {
		//! A collection of common imports for writing rules and modifiers.

		pub use core::marker::PhantomData;

		pub use crate::{options::Unset, AsRef, AsSlice, Error, Report};
		#[allow(missing_docs)]
		pub type Result<T> = core::result::Result<T, Error>;
	}

	#[allow(unused_imports)]
	pub(crate) mod test {
		pub use crate::{toolbox::rule::*, Modifier, Modify, Rule, Validate, Wary};
	}
}

/// Trait for validating and modifying data.
///
/// This is a simple wrapper around types that are [`Validate`] and [`Modify`],
/// first validating the type then modifying if validation returned no errors.
pub trait Wary: Validate + Modify {
	/// Validates with [`Validate::validate`], then (if successful) modifies with
	/// [`Modify::modify`].
	///
	/// # Errors
	///
	/// Forwards any errors from [`Validate::validate`].
	fn wary(&mut self, ctx: &Self::Context) -> Result<(), Report> {
		self.validate(ctx)?;
		self.modify(ctx);
		Ok(())
	}
}

impl<T> Wary for T where T: Validate + Modify {}

/// Trait for modifying other data.
pub trait Modifier<I: ?Sized> {
	/// Additional context required to modify the input.
	type Context;

	/// Modify the input.
	fn modify(&self, ctx: &Self::Context, item: &mut I);
}

/// Trait for modifying itself.
pub trait Modify: Validate {
	/// Modify itself.
	fn modify(&mut self, ctx: &Self::Context);
}

/// Trait for validating other data.
pub trait Rule<I: ?Sized> {
	/// Additional context required to validate the input.
	type Context;

	/// Validates the item.
	///
	/// # Errors
	///
	/// Returns an error if the item does not pass validation.
	fn validate(&self, ctx: &Self::Context, item: &I) -> Result<(), Error>;
}

/// Trait for validating itself.
pub trait Validate {
	/// Additional context required to validate or modify the input.
	type Context;

	/// Validates itself and appends all errors to the attached [`Report`].
	fn validate_into(&self, ctx: &Self::Context, parent: &Path, report: &mut Report);

	/// Validates itself.
	///
	/// # Errors
	///
	/// Returns all errors found during validation.
	fn validate(&self, ctx: &Self::Context) -> Result<(), Report> {
		let mut report = Report::default();
		self.validate_into(ctx, &Path::default(), &mut report);

		if report.is_empty() {
			Ok(())
		} else {
			Err(report)
		}
	}
}

impl<T> Validate for Option<T>
where
	T: Validate,
{
	type Context = T::Context;

	#[inline]
	fn validate_into(&self, ctx: &Self::Context, parent: &Path, report: &mut Report) {
		if let Some(inner) = self {
			inner.validate_into(ctx, parent, report);
		}
	}
}

impl<T> Validate for &T
where
	T: Validate,
{
	type Context = T::Context;

	#[inline]
	fn validate_into(&self, ctx: &Self::Context, parent: &Path, report: &mut Report) {
		(*self).validate_into(ctx, parent, report);
	}
}

/// Trait for cheap reference-to-reference conversion.
///
/// This trait contains a blanket implementation for all
/// [`AsRef`](std::convert::AsRef) types using the standard library's trait of
/// the same name. Additional implementations were created for better ergonomics
/// with strings and other data.
pub trait AsRef<T: ?Sized> {
	/// Converts this type into a shared reference of the input type.
	fn as_ref(&self) -> &T;
}

impl<To: ?Sized, From: core::convert::AsRef<To> + ?Sized> AsRef<To> for From {
	#[inline]
	fn as_ref(&self) -> &To {
		self.as_ref()
	}
}

/// Trait for cheap reference-to-slice conversion.
///
/// This trait is used for accepting slices of data like [`Vec`],
/// [`std::slice`], [`Option`], and other slice-like types for validation and
/// modification.
pub trait AsSlice {
	/// An element of the output slice.
	type Item;

	/// Converts the type into a slice.
	fn as_slice(&self) -> &[Self::Item];
}

impl<T> AsSlice for &T
where
	T: AsSlice,
{
	type Item = T::Item;

	#[inline]
	fn as_slice(&self) -> &[Self::Item] {
		(**self).as_slice()
	}
}

impl<T> AsSlice for &mut T
where
	T: AsSlice,
{
	type Item = T::Item;

	#[inline]
	fn as_slice(&self) -> &[Self::Item] {
		(**self).as_slice()
	}
}

impl<T> AsSlice for Option<T> {
	type Item = T;

	#[inline]
	fn as_slice(&self) -> &[Self::Item] {
		Option::as_slice(self)
	}
}

impl<T> AsSlice for Vec<T> {
	type Item = T;

	#[inline]
	fn as_slice(&self) -> &[Self::Item] {
		self
	}
}

impl<T> AsSlice for [T] {
	type Item = T;

	#[inline]
	fn as_slice(&self) -> &[Self::Item] {
		self
	}
}

impl<const N: usize, T> AsSlice for [T; N] {
	type Item = T;

	#[inline]
	fn as_slice(&self) -> &[Self::Item] {
		self
	}
}

impl AsSlice for &str {
	type Item = u8;

	#[inline]
	fn as_slice(&self) -> &[Self::Item] {
		self.as_bytes()
	}
}

impl AsSlice for str {
	type Item = u8;

	#[inline]
	fn as_slice(&self) -> &[Self::Item] {
		self.as_bytes()
	}
}

impl AsSlice for String {
	type Item = u8;

	#[inline]
	fn as_slice(&self) -> &[Self::Item] {
		self.as_bytes()
	}
}

/// Trait for cheap reference-to-slice conversion with mutability.
///
/// Similar to [`AsSlice`], but mutable.
pub trait AsMutSlice: AsSlice {
	/// Converts the type into a mutable slice.
	fn as_mut_slice(&mut self) -> &mut [Self::Item];
}

impl<T> AsMutSlice for &mut T
where
	T: AsMutSlice,
{
	#[inline]
	fn as_mut_slice(&mut self) -> &mut [Self::Item] {
		(**self).as_mut_slice()
	}
}

impl<T> AsMutSlice for Option<T> {
	#[inline]
	fn as_mut_slice(&mut self) -> &mut [Self::Item] {
		Option::as_mut_slice(self)
	}
}

impl<T> AsMutSlice for Vec<T> {
	#[inline]
	fn as_mut_slice(&mut self) -> &mut [Self::Item] {
		self
	}
}

impl<T> AsMutSlice for [T] {
	#[inline]
	fn as_mut_slice(&mut self) -> &mut [Self::Item] {
		self
	}
}

impl<const N: usize, T> AsMutSlice for [T; N] {
	#[inline]
	fn as_mut_slice(&mut self) -> &mut [Self::Item] {
		self
	}
}
