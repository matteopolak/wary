use std::borrow::Cow;

pub trait DerefStr<'s> {
	type Target;

	fn deref_str(&'s self) -> Self::Target;
}

impl<'s> DerefStr<'s> for str {
	type Target = &'s str;

	fn deref_str(&'s self) -> Self::Target {
		self
	}
}

impl<'s> DerefStr<'s> for String {
	type Target = &'s str;

	fn deref_str(&'s self) -> Self::Target {
		self
	}
}

impl<'s> DerefStr<'s> for Cow<'_, str> {
	type Target = &'s str;

	fn deref_str(&'s self) -> Self::Target {
		self
	}
}

impl<'s, T> DerefStr<'s> for Option<T>
where
	T: DerefStr<'s>,
{
	type Target = Option<T::Target>;

	fn deref_str(&'s self) -> Self::Target {
		self.as_ref().map(DerefStr::deref_str)
	}
}
