#![doc = include_str!("../README.md")]
#![cfg_attr(not(any(test, feature = "std")), no_std)]
#![warn(
	clippy::pedantic,
	clippy::print_stdout,
	clippy::print_stderr,
	clippy::panic
)]
#![allow(
	clippy::new_without_default,
	clippy::wildcard_imports,
	clippy::enum_glob_use
)]
#![cfg_attr(test, allow(non_upper_case_globals))]

pub mod error;
pub mod options;

#[doc(hidden)]
#[cfg(all(not(feature = "std"), feature = "alloc"))]
pub extern crate alloc;
#[cfg(feature = "alloc")]
use alloc::{string::String, vec::Vec};
use core::{future::Future, prelude::rust_2021::*};
#[doc(hidden)]
#[cfg(feature = "std")]
pub use std as alloc;

use error::Path;
pub use error::{Error, Report};
pub use options::rule::{length::Length, range::Compare};
#[cfg(feature = "derive")]
pub use wary_derive::*;

#[doc(hidden)]
pub mod internal {
	#[cfg(all(feature = "regex", feature = "std"))]
	#[macro_export]
	macro_rules! init_regex {
		(static $id:ident = $s:expr) => {
			#[allow(non_upper_case_globals)]
			static $id: $crate::alloc::sync::LazyLock<$crate::options::rule::regex::Regex> =
				$crate::alloc::sync::LazyLock::new(|| {
					$crate::options::rule::regex::Regex::new($s).unwrap()
				});
		};
	}

	#[cfg(all(feature = "regex", not(feature = "std")))]
	#[macro_export]
	macro_rules! init_regex {
		(static $id:ident = $s:expr) => {
			#[allow(non_upper_case_globals)]
			static $id: once_cell::sync::Lazy<$crate::options::rule::regex::Regex> =
				once_cell::sync::Lazy::new(|| $crate::options::rule::regex::Regex::new($s).unwrap());
		};
	}

	#[cfg(feature = "regex")]
	pub use init_regex;
}

pub mod toolbox {
	//! A collection of common imports for various use-cases.

	#[allow(unused_imports)]
	pub mod rule {
		//! A collection of common imports for writing rules and modifiers.

		pub use core::{marker::PhantomData, prelude::rust_2021::*};

		#[cfg(feature = "alloc")]
		pub(crate) use crate::alloc::{
			borrow::Cow,
			boxed::Box,
			format,
			string::{String, ToString},
			vec,
			vec::Vec,
		};
		pub use crate::{options::Unset, AsMut, AsRef, AsSlice, Error, Report};
		#[allow(missing_docs)]
		pub type Result<T> = core::result::Result<T, Error>;
	}

	#[allow(unused_imports)]
	pub mod test {
		pub use crate::{
			toolbox::rule::*, AsyncRule, AsyncTransform, AsyncTransformer, AsyncValidate, Rule,
			Transform, Transformer, Validate, Wary,
		};
	}
}

/// Trait for validating and transforming data.
///
/// This is a simple wrapper around types that are [`Validate`] and
/// [`Transform`], first validating the type then transforming if validation
/// returned no errors.
pub trait Wary<C>: Validate<Context = C> + Transform<Context = C> {
	/// Validates with [`Validate::validate`], then (if successful) modifies with
	/// [`Transform::transform`].
	///
	/// # Errors
	///
	/// Forwards any errors from [`Validate::validate`].
	fn wary(&mut self, ctx: &C) -> Result<(), Report> {
		self.validate(ctx)?;
		self.transform(ctx);
		Ok(())
	}
}

impl<T, C> Wary<C> for T where T: Validate<Context = C> + Transform<Context = C> {}

pub trait AsyncWary<C>: AsyncValidate<Context = C> + AsyncTransform<Context = C> {
	/// Validates with [`AsyncValidate::validate_async`], then (if successful)
	/// modifies with [`AsyncTransform::transform_async`].
	///
	/// # Errors
	///
	/// Forwards any errors from [`AsyncValidate::validate_async`].
	fn wary_async(&mut self, ctx: &C) -> impl Future<Output = Result<(), Report>> + Send;
}

impl<T, C> AsyncWary<C> for T
where
	T: AsyncValidate<Context = C> + AsyncTransform<Context = C> + Send + Sync,
	C: Sync,
{
	async fn wary_async(&mut self, ctx: &C) -> Result<(), Report> {
		let mut report = Report::default();

		self
			.validate_into_async(ctx, &Path::default(), &mut report)
			.await;
		if report.is_empty() {
			self.transform_async(ctx).await;
			Ok(())
		} else {
			Err(report)
		}
	}
}

/// Trait for transforming other data.
pub trait Transformer<I: ?Sized> {
	/// Additional context required to transform the input.
	type Context;

	/// Transform the input.
	fn transform(&self, ctx: &Self::Context, item: &mut I);
}

pub trait AsyncTransformer<I: ?Sized> {
	/// Additional context required to transform the input.
	type Context: Send;

	/// Transform the input.
	fn transform_async(&self, ctx: &Self::Context, item: &mut I) -> impl Future<Output = ()> + Send;
}

/// Trait for transforming itself.
pub trait Transform {
	/// Additional context required to transform itself.
	type Context;

	/// Transform itself.
	fn transform(&mut self, ctx: &Self::Context);
}

pub trait AsyncTransform {
	/// Additional context required to transform itself.
	type Context: Send;

	/// Transform itself.
	fn transform_async(&mut self, ctx: &Self::Context) -> impl Future<Output = ()> + Send;
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

pub trait AsyncRule<I: ?Sized> {
	/// Additional context required to validate the input.
	type Context: Send;

	/// Validates the item.
	///
	/// # Errors
	///
	/// Returns an error if the item does not pass validation.
	fn validate_async(
		&self,
		ctx: &Self::Context,
		item: &I,
	) -> impl Future<Output = Result<(), Error>> + Send;
}

/// Trait for validating itself.
pub trait Validate {
	/// Additional context required to validate itself.
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

pub trait AsyncValidate {
	/// Additional context required to validate itself.
	type Context: Send;

	/// Validates itself and appends all errors to the attached [`Report`].
	fn validate_into_async(
		&self,
		ctx: &Self::Context,
		parent: &Path,
		report: &mut Report,
	) -> impl Future<Output = ()> + Send;

	/// Validates itself.
	fn validate_async(&self, ctx: &Self::Context) -> impl Future<Output = Result<(), Report>> + Send
	where
		Self: Sync,
		Self::Context: Sync,
	{
		let mut report = Report::default();

		async move {
			self
				.validate_into_async(ctx, &Path::default(), &mut report)
				.await;
			if report.is_empty() {
				Ok(())
			} else {
				Err(report)
			}
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

impl<T: ?Sized> Validate for &T
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

/// Trait for cheap mutable-to-mutable reference conversion.
///
/// This trait contains a blanket implementation for all
/// [`AsMut`](std::convert::AsMut) types using the standard library's trait of
/// the same name. Additional implementations were created for better ergonomics
/// with strings and other data.
pub trait AsMut<T: ?Sized> {
	/// Converts this type into a mutable reference of the input type.
	fn as_mut(&mut self) -> &mut T;
}

impl<To: ?Sized, From: core::convert::AsMut<To> + ?Sized> AsMut<To> for From {
	#[inline]
	fn as_mut(&mut self) -> &mut To {
		self.as_mut()
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

impl<T: ?Sized> AsSlice for &T
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
	T: AsSlice + ?Sized,
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
		self.as_slice()
	}
}

#[cfg(feature = "alloc")]
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

impl AsSlice for str {
	type Item = u8;

	#[inline]
	fn as_slice(&self) -> &[Self::Item] {
		self.as_bytes()
	}
}

#[cfg(feature = "alloc")]
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
		self.as_mut_slice()
	}
}

#[cfg(feature = "alloc")]
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
