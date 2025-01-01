use core::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use crate::toolbox::rule::*;

#[doc(hidden)]
pub type Rule_<Mode> = AddrRule<Mode>;

pub struct Ip;
pub struct IpV4;
pub struct IpV6;

pub struct AddrRule<Mode> {
	mode: PhantomData<Mode>,
}

impl AddrRule<Ip> {
	pub fn new() -> AddrRule<Ip> {
		AddrRule { mode: PhantomData }
	}
}

impl<M> AddrRule<M> {
	pub fn ipv4(self) -> AddrRule<IpV4> {
		AddrRule { mode: PhantomData }
	}

	pub fn ipv6(self) -> AddrRule<IpV6> {
		AddrRule { mode: PhantomData }
	}

	pub fn ip(self) -> AddrRule<Ip> {
		AddrRule { mode: PhantomData }
	}
}

impl<I> Rule<I> for AddrRule<IpV4>
where
	I: AsRef<str>,
{
	type Context = ();

	fn validate(&self, _ctx: &Self::Context, item: &I) -> Result<(), Error> {
		let addr = item.as_ref();

		if addr.parse::<Ipv4Addr>().is_ok() {
			Ok(())
		} else {
			panic!()
		}
	}
}

impl<I> Rule<I> for AddrRule<IpV6>
where
	I: AsRef<str>,
{
	type Context = ();

	fn validate(&self, _ctx: &Self::Context, item: &I) -> Result<(), Error> {
		let addr = item.as_ref();

		if addr.parse::<Ipv6Addr>().is_ok() {
			Ok(())
		} else {
			panic!()
		}
	}
}

impl<I> Rule<I> for AddrRule<Ip>
where
	I: AsRef<str>,
{
	type Context = ();

	fn validate(&self, _ctx: &Self::Context, item: &I) -> Result<(), Error> {
		let addr = item.as_ref();

		if addr.parse::<IpAddr>().is_ok() {
			Ok(())
		} else {
			panic!()
		}
	}
}
