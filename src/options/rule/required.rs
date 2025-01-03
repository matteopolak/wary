use crate::toolbox::rule::*;

#[doc(hidden)]
pub type Rule<Mode> = RequiredRule<Mode>;

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum Error {
	#[error("value should be empty")]
	ShouldBeEmpty,
	#[error("value should not be empty")]
	CannotBeEmpty,
}

pub struct Not;

#[must_use]
pub struct RequiredRule<Mode> {
	mode: PhantomData<Mode>,
}

impl RequiredRule<Unset> {
	#[inline]
	pub const fn new() -> Self {
		Self { mode: PhantomData }
	}

	#[inline]
	pub const fn not(self) -> RequiredRule<Not> {
		RequiredRule { mode: PhantomData }
	}
}

impl<I: ?Sized> crate::Rule<I> for RequiredRule<Unset>
where
	I: AsSlice,
{
	type Context = ();

	fn validate(&self, _ctx: &Self::Context, item: &I) -> Result<()> {
		let slice = item.as_slice();

		if slice.is_empty() {
			Err(Error::CannotBeEmpty.into())
		} else {
			Ok(())
		}
	}
}

impl<I: ?Sized> crate::Rule<I> for RequiredRule<Not>
where
	I: AsSlice,
{
	type Context = ();

	fn validate(&self, _ctx: &Self::Context, item: &I) -> Result<()> {
		let slice = item.as_slice();

		if slice.is_empty() {
			Ok(())
		} else {
			Err(Error::ShouldBeEmpty.into())
		}
	}
}

#[cfg(test)]
mod test {
	use super::{Not, RequiredRule};
	use crate::toolbox::test::*;

	const rule: RequiredRule<Unset> = RequiredRule::new();
	const not: RequiredRule<Not> = RequiredRule::new().not();

	#[test]
	fn test_required_rule_option() {
		assert!(rule.validate(&(), &Some(1)).is_ok());
		assert!(rule.validate(&(), &None::<i32>).is_err());

		assert!(not.validate(&(), &Some(1)).is_err());
		assert!(not.validate(&(), &None::<i32>).is_ok());
	}

	#[test]
	fn test_required_rule_slice() {
		assert!(rule.validate(&(), &[1]).is_ok());
		assert!(rule.validate(&(), &vec![1, 2, 3]).is_ok());
		assert!(rule.validate(&(), "hello").is_ok());

		assert!(rule.validate(&(), &[] as &[i32; 0]).is_err());
		assert!(rule.validate(&(), &Vec::<i32>::new()).is_err());
		assert!(rule.validate(&(), "").is_err());

		assert!(not.validate(&(), &[1]).is_err());
		assert!(not.validate(&(), &vec![1, 2, 3]).is_err());
		assert!(not.validate(&(), "hello").is_err());

		assert!(not.validate(&(), &[] as &[i32; 0]).is_ok());
		assert!(not.validate(&(), &Vec::<i32>::new()).is_ok());
		assert!(not.validate(&(), "").is_ok());
	}
}
