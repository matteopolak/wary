use crate::toolbox::rule::*;

#[doc(hidden)]
pub type Rule = UppercaseRule;

pub struct UppercaseRule;

impl<I: ?Sized> crate::Rule<I> for UppercaseRule
where
	I: AsRef<str>,
{
	type Context = ();

	#[inline]
	fn validate(&self, _ctx: &Self::Context, item: &I) -> Result<(), Error> {
		let string = item.as_ref();

		if string.chars().all(|c| c.is_uppercase()) {
			Ok(())
		} else {
			panic!()
		}
	}
}

impl crate::Modifier<String> for UppercaseRule {
	type Context = ();

	#[inline]
	fn modify(&self, _ctx: &Self::Context, item: &mut String) {
		*item = item.to_uppercase();
	}
}
