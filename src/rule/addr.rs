use core::net::{Ipv4Addr, Ipv6Addr, IpAddr};
use std::marker::PhantomData;

use crate::{Error, Validate};

#[doc(hidden)]
pub type Rule<T, Mode> = AddrRule<T, Mode>;

pub struct Ip;
pub struct IpV4;
pub struct IpV6;

pub struct AddrRule<T, Mode> {
	inner: T,
	mode: PhantomData<Mode>
}

impl<T> AddrRule<T, Ip> {
	pub fn new(inner: T) -> AddrRule<T, Ip> {
		AddrRule {
			inner,
			mode: PhantomData
		}
	}
}

impl<T, M> AddrRule<T, M> {
	pub fn ipv4(self) -> AddrRule<T, IpV4> {
		AddrRule {
			inner: self.inner,
			mode: PhantomData
		}
	}

	pub fn ipv6(self) -> AddrRule<T, IpV6> {
		AddrRule {
			inner: self.inner,
			mode: PhantomData
		}
	}

	pub fn ip(self) -> AddrRule<T, Ip> {
		AddrRule {
			inner: self.inner,
			mode: PhantomData
		}
	}
}

impl<T> Validate for AddrRule<T, IpV4>
	where T: AsRef<str>
{
	type Context = ();

	fn validate(&self, _ctx: &Self::Context) -> Result<(), Error> {
		let addr = self.inner.as_ref();

		if addr.parse::<Ipv4Addr>().is_ok() {
			Ok(())
		} else {
			panic!()
		}
	}
}

impl<T> Validate for AddrRule<T, IpV6>
	where T: AsRef<str>
{
	type Context = ();

	fn validate(&self, _ctx: &Self::Context) -> Result<(), Error> {
		let addr = self.inner.as_ref();

		if addr.parse::<Ipv6Addr>().is_ok() {
			Ok(())
		} else {
			panic!()
		}
	}
}

impl<T> Validate for AddrRule<T, Ip>
	where T: AsRef<str>
{
	type Context = ();

	fn validate(&self, _ctx: &Self::Context) -> Result<(), Error> {
		let addr = self.inner.as_ref();

		if addr.parse::<IpAddr>().is_ok() {
			Ok(())
		} else {
			panic!()
		}
	}
}
