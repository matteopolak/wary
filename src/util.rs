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

impl<'s> DerefStr<'s> for Option<String> {
	type Target = Option<&'s str>;

	fn deref_str(&'s self) -> Self::Target {
		self.as_deref()
	}
}
