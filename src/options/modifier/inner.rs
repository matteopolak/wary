use crate::AsSliceMut;

#[doc(hidden)]
pub type Modifier<F> = InnerRule<F>;

pub struct InnerRule<F> {
	modify: F,
}

impl<F> InnerRule<F> {
	pub fn new(modify: F) -> Self {
		Self { modify }
	}
}

impl<I: ?Sized, O, F> crate::Modifier<I> for InnerRule<F>
where
	I: AsSliceMut<Item = O>,
	F: Fn(&mut O),
{
	type Context = ();

	fn modify(&self, _ctx: &Self::Context, item: &mut I) {
		for item in item.as_slice_mut() {
			(self.modify)(item);
		}
	}
}
