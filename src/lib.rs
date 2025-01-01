#![allow(clippy::new_without_default)]
//#![deny(clippy::print_stdout)]

pub mod error;
pub mod options;
pub mod util;
pub use error::{Error, Report};
#[cfg(feature = "derive")]
pub use wary_derive::*;

pub struct Transcript;

pub mod toolbox {
	pub mod rule {
		pub use core::marker::PhantomData;

		pub use crate::{options::Unset, AsRef, AsSlice, Error, Report};
	}
}

pub trait Modifier<I: ?Sized> {
	type Context;

	fn modify(&self, ctx: &Self::Context, item: &mut I);
}

pub trait Modify {
	type Context;

	fn modify(&mut self, ctx: &Self::Context);
}

pub trait Rule<I: ?Sized> {
	type Context;

	fn validate(&self, ctx: &Self::Context, item: &I) -> Result<(), Error>;
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

	#[inline]
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

	#[inline]
	fn validate(&self, ctx: &Self::Context) -> Result<(), Error> {
		(*self).validate(ctx)
	}
}

pub trait AsRef<T: ?Sized> {
	fn as_ref(&self) -> &T;
}

impl<To: ?Sized, From: core::convert::AsRef<To> + ?Sized> AsRef<To> for From {
	#[inline]
	fn as_ref(&self) -> &To {
		self.as_ref()
	}
}

pub trait AsSlice {
	type Item;

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

impl AsSlice for &str {
	type Item = u8;

	#[inline]
	fn as_slice(&self) -> &[Self::Item] {
		self.as_bytes()
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

pub trait AsSliceMut: AsSlice {
	fn as_slice_mut(&mut self) -> &mut [Self::Item];
}

impl<T> AsSliceMut for &mut T
where
	T: AsSliceMut,
{
	#[inline]
	fn as_slice_mut(&mut self) -> &mut [Self::Item] {
		(**self).as_slice_mut()
	}
}

impl<T> AsSliceMut for Vec<T> {
	#[inline]
	fn as_slice_mut(&mut self) -> &mut [Self::Item] {
		self
	}
}

impl<T> AsSliceMut for [T] {
	#[inline]
	fn as_slice_mut(&mut self) -> &mut [Self::Item] {
		self
	}
}

impl<const N: usize, T> AsSliceMut for [T; N] {
	#[inline]
	fn as_slice_mut(&mut self) -> &mut [Self::Item] {
		self
	}
}
