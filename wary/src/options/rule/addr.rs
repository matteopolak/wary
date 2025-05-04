//! Rule for address validation.
//!
//! See [`AddrRule`] for more information.

use core::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use crate::toolbox::rule::*;

#[doc(hidden)]
pub type Rule<Mode> = AddrRule<Mode>;

#[derive(Debug, thiserror::Error, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize), serde(untagged))]
pub enum Error {
	#[error("invalid_ip")]
	InvalidIp,
	#[error("invalid_ipv4")]
	InvalidIpv4,
	#[error("invalid_ipv6")]
	InvalidIpv6,
}

impl Error {
	#[must_use]
	pub(crate) fn code(&self) -> &'static str {
		match self {
			Self::InvalidIp => "invalid_ip",
			Self::InvalidIpv4 => "invalid_ipv4",
			Self::InvalidIpv6 => "invalid_ipv6",
		}
	}

	pub(crate) fn message(&self) -> &'static str {
		match self {
			Self::InvalidIp => "invalid IP address",
			Self::InvalidIpv4 => "invalid IPv4 address",
			Self::InvalidIpv6 => "invalid IPv6 address",
		}
	}
}

pub struct Ip;
pub struct IpV4;
pub struct IpV6;

/// Rule for address validation.
///
/// # Example
///
/// ```
/// use wary::{Wary, Validate};
///
/// #[derive(Wary)]
/// struct Packet {
///   #[validate(addr(ipv4))]
///   src: String,
///   #[validate(addr(ipv6))]
///   dst: String,
/// }
///
/// let packet = Packet {
///   src: "192.168.1.1".into(),
///   dst: "2001:0db8:85a3:0000:0000:8a2e:0370:7334".into(),
/// };
///
/// assert!(packet.validate(&()).is_ok());
///
/// let packet = Packet {
///   src: "1.2.3.4.5".into(),
///   dst: "localhost".into(),
/// };
///
/// assert!(packet.validate(&()).is_err());
/// ```
#[must_use]
pub struct AddrRule<Mode> {
	mode: PhantomData<Mode>,
}

impl AddrRule<Ip> {
	#[inline]
	pub const fn new() -> AddrRule<Ip> {
		AddrRule { mode: PhantomData }
	}
}

impl<M> AddrRule<M> {
	#[inline]
	pub const fn ipv4(self) -> AddrRule<IpV4> {
		AddrRule { mode: PhantomData }
	}

	#[inline]
	pub const fn ipv6(self) -> AddrRule<IpV6> {
		AddrRule { mode: PhantomData }
	}

	#[inline]
	pub const fn ip(self) -> AddrRule<Ip> {
		AddrRule { mode: PhantomData }
	}
}

impl<I> crate::Rule<I> for AddrRule<IpV4>
where
	I: AsRef<str>,
{
	type Context = ();

	fn validate(&self, _ctx: &Self::Context, item: &I) -> Result<()> {
		let addr = item.as_ref();

		if addr.parse::<Ipv4Addr>().is_ok() {
			Ok(())
		} else {
			Err(Error::InvalidIpv4.into())
		}
	}
}

impl<I> crate::Rule<I> for AddrRule<IpV6>
where
	I: AsRef<str>,
{
	type Context = ();

	fn validate(&self, _ctx: &Self::Context, item: &I) -> Result<()> {
		let addr = item.as_ref();

		if addr.parse::<Ipv6Addr>().is_ok() {
			Ok(())
		} else {
			Err(Error::InvalidIpv6.into())
		}
	}
}

impl<I> crate::Rule<I> for AddrRule<Ip>
where
	I: AsRef<str>,
{
	type Context = ();

	fn validate(&self, _ctx: &Self::Context, item: &I) -> Result<()> {
		let addr = item.as_ref();

		if addr.parse::<IpAddr>().is_ok() {
			Ok(())
		} else {
			Err(Error::InvalidIp.into())
		}
	}
}

#[cfg(test)]
mod test {
	use crate::toolbox::test::*;

	#[test]
	fn test_addr_ipv4_rule() {
		#[derive(Wary)]
		#[wary(crate = "crate")]
		struct Packet {
			#[validate(addr(ipv4))]
			src: String,
			#[validate(addr(ipv4))]
			dst: String,
		}

		let packet = Packet {
			src: "192.168.1.1".into(),
			dst: "1.1.1.1".into(),
		};

		assert!(packet.validate(&()).is_ok());

		let packet = Packet {
			src: "192.168.1.1.1".into(),
			dst: "1.1.1.1".into(),
		};

		assert!(packet.validate(&()).is_err());
	}

	#[test]
	fn test_addr_ipv6_rule() {
		#[derive(Wary)]
		#[wary(crate = "crate")]
		struct Packet {
			#[validate(addr(ipv6))]
			src: String,
			#[validate(addr(ipv6))]
			dst: String,
		}

		let packet = Packet {
			src: "2001:0db8:85a3:0000:0000:8a2e:0370:7334".into(),
			dst: "2001:0db8:85a3:0000:0000:8a2e:0370:7334".into(),
		};

		assert!(packet.validate(&()).is_ok());

		let packet = Packet {
			src: "2001:0db8:85a3:0000:0000:8a2e:0370:7334".into(),
			dst: "2001:0db8:85a3:0000:0000:8a2e:0370:7334:7334".into(),
		};

		assert!(packet.validate(&()).is_err());
	}

	#[test]
	fn test_addr_ip_rule() {
		#[derive(Wary)]
		#[wary(crate = "crate")]
		struct Packet {
			#[validate(addr)]
			src: String,
			#[validate(addr)]
			dst: String,
		}

		let packet = Packet {
			src: "192.168.1.1".into(),
			dst: "2001:0db8:85a3:0000:0000:8a2e:0370:7334".into(),
		};

		assert!(packet.validate(&()).is_ok());
	}
}
