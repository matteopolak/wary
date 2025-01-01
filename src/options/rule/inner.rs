use crate::toolbox::rule::*;

#[doc(hidden)]
pub type Rule<F> = InnerRule<F>;

pub struct InnerRule<F> {
	validate: F,
}

impl<F> InnerRule<F> {
	pub fn new(validate: F) -> Self {
		Self { validate }
	}
}

impl<I: ?Sized, O, F> crate::Rule<I> for InnerRule<F>
where
	I: AsSlice<Item = O>,
	F: Fn(&O) -> Result<(), Error>,
{
	type Context = ();

	fn validate(&self, _ctx: &Self::Context, item: &I) -> Result<(), Error> {
		for item in item.as_slice() {
			(self.validate)(item)?;
		}

		Ok(())
	}
}
