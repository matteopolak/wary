# Wary

[![<https://img.shields.io/crates/v/wary>](https://img.shields.io/crates/v/wary)](https://crates.io/crates/wary)
[![<https://img.shields.io/docsrs/wary>](https://img.shields.io/docsrs/wary)](https://docs.rs/wary/latest/wary/)
[![ci status](https://github.com/matteopolak/wary/workflows/ci/badge.svg)](https://github.com/matteopolak/wary/actions)

A validation and transformation library.

- Basic usage
  - [Basic struct example](#basic-struct-example)
  - [Basic enum example](#basic-enum-example)
- [Accessing context](#context)
- [Validation rules](#validation-rules)
  - [Implementing custom `Rule`s](#rule-custom)
  - [Implementing `Validate` manually](#manual-validate)
- [Transformation rules](#transformation-rules)
  - [Implementing custom `Modifier`s](#modifier-custom)
  - [Implementing `Modify` manually](#manual-modify)

### Basic struct example

```rust
use std::borrow::Cow;
use wary::Wary;

#[derive(Wary)]
struct Name<'n>(
  #[validate(alphanumeric, length(chars, 5..=20), equals(not, other = "john"))]
  Cow<'n, str>
);

#[derive(Wary)]
struct Person<'n> {
  #[validate(dive)]
  name: Name<'n>,
  #[validate(range(..=100))]
  age: u8,
}

let mut person = Person {
  name: Name(Cow::Borrowed("jane")),
  age: 25,
};

if let Err(report) = person.wary(&()) {
  eprintln!("invalid person: {report:?}");
}
```

### Basic enum example

```rust
use std::borrow::Cow;
use wary::Wary;

#[derive(Wary)]
struct Name<'n>(
  #[validate(alphanumeric, length(chars, 5..=20), equals(not, other = "john"))]
  #[modify(lowercase(ascii))]
  &'n mut str
);

// for length(bytes)
impl wary::AsRef<[u8]> for Name<'_> {
  fn as_ref(&self) -> &[u8] {
    self.0.as_bytes()
  }
}

#[derive(Wary)]
enum Person<'n> {
  Child {
    #[validate(dive)]
    name: Name<'n>,
    #[validate(range(..=17))]
    age: u8,
  },
  Adult {
    #[validate(dive, length(bytes, ..=32))]
    name: Name<'n>,
    #[validate(range(18..=100))]
    age: u8,
  },
}

let mut name = "Jane".to_string();
let mut person = Person::Adult {
  name: Name(&mut name),
  age: 25,
};

if let Err(report) = person.wary(&()) {
  eprintln!("invalid person: {report:?}");
} else {
  let Person::Adult { name, age } = person else {
    unreachable!();
  };

  assert_eq!(name.0, "jane");
}
```

## Accessing context <a id="context"></a>

```rust
use wary::Wary;
use wary::toolbox::rule::*;
use std::ops::Range;

// allows one context to be passed to all rules
#[derive(AsRef)]
struct Context {
  range: Range<u8>,
  #[as_ref(skip)]
  useless: bool,
}

struct RangeRule<C> {
  ctx: PhantomData<C>,
}

impl<C> RangeRule<C> {
  fn new() -> Self {
    Self {
      ctx: PhantomData,
    }
  }
}

impl<C> wary::Rule<u8> for RangeRule<C>
where
  C: AsRef<Range<u8>>,
{
  type Context = C;

  fn validate(&self, ctx: &Self::Context, item: &u8) -> Result<()> {
    if ctx.as_ref().contains(item) {
      Ok(())
    } else {
      Err(wary::Error::with_message("out_of_range", "The number is out of range"))
    }
  }
}

#[allow(non_camel_case_types)]
mod rule {
  pub type range<C> = super::RangeRule<C>;
}

#[derive(Wary)]
#[wary(context = Context)]
struct Age {
  #[validate(custom(range))]
  number: u8,
}

# fn main() {}
```

## Validation rules

Validation rules applied through the proc-macro `Wary` attribute are (for the most part) simply forwarded
directly to their respective builders inside the [`rule`](crate::options::rule) module. As a result of this
decision, all rules (except `and`, `or`, `inner`, and `dive`) will have auto-completion when writing macro attributes!

If you're providing no options to a rule, you can omit the parentheses. For example: `#[validate(alphanumeric)]`
and `#[validate(alphanumeric())]` are equivalent.

| rule | trait | feature |
| ---- | ----- | ------- |
| [`addr`](#rule-addr) | [`AsRef<str>`](wary::AsRef) | - |
| [`alphanumeric`](#rule-alphanumeric) | [`AsRef<str>`](wary::AsRef) | - |
| [`ascii`](#rule-ascii) | [`AsRef<str>`](wary::AsRef) | - |
| [`contains`](#rule-contains) | [`AsSlice`](wary::AsSlice) | - |
| [`custom`](#rule-custom) | [`Rule<T>`](wary::Rule) | - |
| [`dive`](#rule-dive) | [`Validate`](wary::Validate) | - |
| [`email`](#rule-email) | [`AsRef<str>`](wary::AsRef) | `email` |
| [`equals`](#rule-equals) | [`std::cmp::PartialEq`](std::cmp::PartialEq) | - |
| [`func`](#rule-func) | `Fn(&T) -> Result<(), wary::Error>` | - |
| [`inner`](#rule-inner) | [`AsSlice`](wary::AsSlice) | - |
| [`length`](#rule-length) | [`Length`](wary::Length) | `graphemes` (optional, for `graphemes` length) |
| [`lowercase`](#rule-lowercase) | [`AsRef<str>`](wary::AsRef) | - |
| [`prefix`](#rule-prefix) | [`AsSlice`](wary::AsSlice) | - |
| [`range`](#rule-range) | [`Range`](wary::Range) | - |
| [`regex`](#rule-regex) | [`AsRef<str>`](wary::AsRef) | `regex` |
| [`required`](#rule-required) | [`AsSlice`](wary::AsSlice) | - |
| [`semver`](#rule-semver) | [`AsRef<str>`](wary::AsRef) | `semver` |
| [`suffix`](#rule-suffix) | [`AsSlice`](wary::AsSlice) | - |
| [`uppercase`](#rule-uppercase) | [`AsRef<str>`](wary::AsRef) | - |
| [`url`](#rule-url) | [`AsRef<str>`](wary::AsRef) | `url` |

### `addr`

Validates an address (currently only an IP).

```rust
use wary::Wary;

#[derive(Wary)]
struct Packet {
  #[validate(addr(ipv4))]
  src: String,
  #[validate(addr(ipv6))]
  dest: String,
  #[validate(addr(ip))]
  more: String,
}
```

### `alphanumeric`

Validates that the input is alphanumeric.

```rust
use wary::Wary;

#[derive(Wary)]
struct Name {
  #[validate(alphanumeric)]
  left: String,
  #[validate(alphanumeric(ascii))]
  right: String,
}
```

### `ascii` <a id="rule-ascii"></a>

Validates that the input is ascii.

```rust
use wary::Wary;

#[derive(Wary)]
struct Name(
  #[validate(ascii)]
  String
);
```

### `contains` <a id="rule-contains"></a>

Validates that the input contains a substring or subslice.

```rust
use wary::Wary;

#[derive(Wary)]
struct Name(
  #[validate(contains(str = "hello"))]
  String
);
```

### `custom` <a id="rule-custom"></a>

Validates the input with a custom [`Rule`](wary::Rule).

```rust
use wary::Wary;
use wary::toolbox::rule::*;

struct SecretRule;

impl SecretRule {
  fn new() -> Self {
    Self
  }
}

impl<I> wary::Rule<I> for SecretRule
where
  I: AsRef<str>,
{
  type Context = ();

  fn validate(&self, _ctx: &Self::Context, item: &I) -> Result<()> {
    let string = item.as_ref();

    if string.contains("secret") {
      Err(Error::with_message("secret_found", "You cannot use the word 'secret'"))
    } else {
      Ok(())
    }
  }
}

#[allow(non_camel_case_types)]
mod rule {
  pub type secret = super::SecretRule;
}

#[derive(Wary)]
struct Person {
  #[validate(custom(secret))]
  name: String,
}

# fn main() {}
```

### `dive` <a id="rule-dive"></a>

Validates the inner fields of a struct or enum.

```rust
use wary::Wary;

#[derive(Wary)]
struct Item {
  #[validate(ascii)]
  name: &'static str,
}

#[derive(Wary)]
struct Name {
  #[validate(dive)]
  item: Item,
}
```

### `email` (requires feature `email`) <a id="rule-email"></a>

Validates that the input is an email.

```rust
use wary::Wary;

#[derive(Wary)]
struct Email(
  #[validate(email)]
  String
);
```

### `equals` <a id="rule-equals"></a>

Validates that the input is equal to a value. Currently does not support `self` fields.

```rust
use wary::Wary;

#[derive(Wary)]
struct Name(
  #[validate(equals(other = "John"))]
  String
);
```

### `func` <a id="rule-func"></a>

Validates the input with a function.

```rust
use wary::{Wary, Error};

fn check(_ctx: &(), name: &str) -> Result<(), Error> {
  if name.len() > 5 {
    Ok(())
  } else {
    Err(Error::with_message("name_too_short", "Your name must be longer than 5 characters"))
  }
}

#[derive(Wary)]
struct Name {
  #[validate(func = |ctx: &(), name: &str| {
    if name.len() > 5 {
      Ok(())
    } else {
      Err(Error::with_message("name_too_short", "Your name must be longer than 5 characters"))
    }
  })]
  left: String,
  #[validate(func = check)]
  right: String,
}
```

### `inner` <a id="rule-inner"></a>

Validates the inner fields of a slice-like type.

```rust
use wary::Wary;

#[derive(Wary)]
struct Name {
  #[validate(inner(ascii))]
  items: Vec<String>,
}
```

### `length` <a id="rule-length"></a>

Validates the length of the input.

```rust
use wary::Wary;

#[derive(Wary)]
struct Name {
  // counts the length in bytes
  #[validate(length(bytes, 5..=20))]
  bytes: String,
  // counts the length in characters
  #[validate(length(chars, 5..=20))]
  chars: String,
  // counts the length in UTF-16 code units
  #[validate(length(code_units, 5..=20))]
  code_points: String,
  // counts the length in grapheme clusters
  #[validate(length(graphemes, 5..=20))]
  graphemes: String,
}
```

### `lowercase` <a id="rule-lowercase"></a>

Validates that the input is lowercase.

```rust
use wary::Wary;

#[derive(Wary)]
struct Name {
  #[validate(lowercase)]
  left: String,
  #[validate(lowercase(ascii))]
  right: String,
}
```

### `prefix` <a id="rule-prefix"></a>

Validates that the input starts with a substring or subslice.

```rust
use wary::Wary;

#[derive(Wary)]
struct Name(
  #[validate(prefix(str = "hello"))]
  String
);
```

### `range` <a id="rule-range"></a>

Validates that the input is within a range.

```rust
use wary::Wary;

#[derive(Wary)]
struct Age {
  #[validate(range(18..=100))]
  number: u8,
  #[validate(range('a'..='z'))]
  char: char,
  #[validate(range("hello".."world"))]
  string: String,
}
```

### `regex` (requires feature `regex`) <a id="rule-regex"></a>

Validates that the input matches a regex.

```rust
use wary::Wary;

#[derive(Wary)]
struct Name(
  #[validate(regex(pat = "^[a-z]+$"))]
  String
);
```

### `required` <a id="rule-required"></a>

Validates that the input is not empty. For example, that an `Option` is `Some` or a `Vec` is not empty.

```rust
use wary::Wary;

#[derive(Wary)]
struct Name {
  #[validate(required)]
  first: String,
  #[validate(required)]
  last: Option<String>,
}
```

### `semver` (requires feature `semver`) <a id="rule-semver"></a>

Validates that the input is a semver.

```rust
use wary::Wary;

#[derive(Wary)]
struct Version(
  #[validate(semver)]
  String
);
```

### `suffix` <a id="rule-suffix"></a>

Validates that the input ends with a substring or subslice.

```rust
use wary::Wary;

#[derive(Wary)]
struct Name(
  #[validate(suffix(str = "hello"))]
  String
);
```

### `uppercase` <a id="rule-uppercase"></a>

Validates that the input is uppercase.

```rust
use wary::Wary;

#[derive(Wary)]
struct Name {
  #[validate(uppercase)]
  left: String,
  #[validate(uppercase(ascii))]
  right: String,
}
```

### `url` (requires feature `url`) <a id="rule-url"></a>

Validates that the input is a url.

```rust
use wary::Wary;

#[derive(Wary)]
struct Url(
  #[validate(url)]
  String
);
```

### Implementing `Validate` manually <a id="manual-validate"></a>

In the rare case you need to manually implement `Validate`, you will need to keep in mind about reporting errors properly.

```rust
use wary::{Validate, Error, error::{Path, Report}};

struct Name {
  value: String,
}

impl Validate for Name {
  type Context = ();

  fn validate_into(&self, _ctx: &Self::Context, parent: &Path, report: &mut Report) {
    if self.value.len() < 5 {
      report.push(
        parent.append("value"),
        Error::with_message("name_too_short", "Your name must be longer than 5 characters"),
      );
    }
  }
}

let name = Name {
  value: "Jane".to_string(),
};

assert!(name.validate(&()).is_err());

let longer = Name {
  value: "Jane Doe".to_string(),
};

assert!(longer.validate(&()).is_ok());
```

## Transformation rules

Transformation rules are applied similarly to validation rules, but are implemented in the [`Modify`](wary::Modify) trait instead.

| rule | trait | feature |
| ---- | ----- | ------- |
| [`custom`](#modifier-custom) | [`Modifier`](wary::Modifier) | - |
| [`lowercase`](#modifier-lowercase) | [`AsMut<str>`](wary::AsMut) (for `ascii` only) | - |
| [`inner`](#modifier-inner) | [`AsMutSlice`](wary::AsMutSlice) | - |
| [`uppercase`](#modifier-uppercase) | [`AsMut<str>`](wary::AsMut) (for `ascii` only) | - |

### `custom` <a id="modifier-custom"></a>

Transforms the input with a custom [`Modifier`](wary::Modifier).

```rust
use wary::{Wary, Modifier};

struct SecretModifier;

impl SecretModifier {
  fn new() -> Self {
    Self
  }
}

impl Modifier<String> for SecretModifier {
  type Context = ();

  fn modify(&self, _ctx: &Self::Context, item: &mut String) {
    item.clear();
    item.push_str("secret");
  }
}

#[allow(non_camel_case_types)]
mod modifier {
  pub type secret = super::SecretModifier;
}

#[derive(Wary)]
struct Person {
  #[modify(custom(secret))]
  name: String,
}

# fn main() {}
```

### `lowercase` <a id="modifier-lowercase"></a>

Transforms the input to lowercase.

```rust
use wary::Wary;

#[derive(Wary)]
struct Name {
  #[modify(lowercase)]
  left: String,
  #[modify(lowercase(ascii))]
  right: String,
}
```

### `inner` <a id="modifier-inner"></a>

Transforms the inner fields of a slice-like type.

```rust
use wary::Wary;

#[derive(Wary)]
struct Name {
  #[modify(inner(lowercase))]
  items: Vec<String>,
}
```

### `uppercase` <a id="modifier-uppercase"></a>

Transforms the input to uppercase.

```rust
use wary::Wary;

#[derive(Wary)]
struct Name {
  #[modify(uppercase)]
  left: String,
  #[modify(uppercase(ascii))]
  right: String,
}
```

### Implementing `Modify` manually <a id="manual-modify"></a>

```rust
use wary::Modify;

struct Name {
  value: String,
}

impl Modify for Name {
  type Context = ();

  fn modify(&mut self, _ctx: &Self::Context) {
    self.value.make_ascii_lowercase();
  }
}

let mut name = Name {
  value: "Jane".to_string(),
};

name.modify(&());

assert_eq!(name.value, "jane");
```
