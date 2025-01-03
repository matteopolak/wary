use crate::AsMutSlice;

#[doc(hidden)]
pub type Modifier<F> = InnerModifier<F>;

pub struct InnerModifier<F> {
	modify: F,
}

impl<F> InnerModifier<F> {
	pub fn new(modify: F) -> Self {
		Self { modify }
	}
}

impl<I: ?Sized, O, F> crate::Modifier<I> for InnerModifier<F>
where
	I: AsMutSlice<Item = O>,
	F: Fn(&mut O),
{
	type Context = ();

	fn modify(&self, _ctx: &Self::Context, item: &mut I) {
		for item in item.as_mut_slice() {
			(self.modify)(item);
		}
	}
}
