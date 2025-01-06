use wary::Wary;

#[allow(non_camel_case_types)]
mod rule {
	pub type password = super::PasswordRule;
}

/// Rule for validating passwords.
///
/// In order to be valid, the password must be EITHER 12+ characters long OR
/// 8+ characters long and contain at least one uppercase letter, one lowercase
/// letter, one digit, and one special character.
struct PasswordRule;

impl PasswordRule {
	const fn new() -> Self {
		Self
	}
}

impl<I> wary::Rule<I> for PasswordRule
where
	I: AsRef<str>,
{
	type Context = ();

	fn validate(&self, _ctx: &Self::Context, item: &I) -> Result<(), wary::Error> {
		let string = item.as_ref();

		if string.len() >= 12
			|| string.len() >= 8
				&& string.chars().any(char::is_uppercase)
				&& string.chars().any(char::is_lowercase)
				&& string.chars().any(char::is_numeric)
				&& string.chars().any(|ch| !ch.is_alphanumeric())
		{
			Ok(())
		} else {
			Err(wary::Error::with_message(
				"password_too_weak",
				"Password must be 12+ characters long or 8+ characters long and contain at least one of \
				 each: uppercase, lowercase, number, special.",
			))
		}
	}
}

#[derive(Debug, Wary)]
struct User {
	#[validate(length(3..=16))]
	#[transform(lowercase)]
	username: String,
	#[validate(custom(password))]
	password: String,
	#[validate(email)]
	email: String,
	#[validate(inner(dive), length(1..))]
	#[transform(inner(dive))]
	posts: Vec<Post>,
}

#[derive(Debug, Wary)]
struct Post {
	#[validate(length(1..=128))]
	#[transform(trim)]
	title: String,
	#[validate(length(1..=1024))]
	#[transform(trim)]
	content: String,
	#[validate(or(equals(other = -5), range(0..)))]
	likes: i64,
}

fn main() {
	let mut user = User {
		username: "GABEN".into(),
		password: "Some long-ish password".into(),
		email: "gaben@valvesoftware.com".into(),
		posts: vec![Post {
			title: "Half-Life 3".into(),
			content: "  It's coming soon.        \n\n".into(),
			likes: -5,
		}],
	};

	if let Err(report) = user.wary(&()) {
		serde_json::to_writer_pretty(std::io::stdout(), &report).unwrap();
	} else {
		println!("Validation passed! {user:#?}");
	}
}
