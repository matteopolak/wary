# Wary

[![<https://img.shields.io/crates/v/wary>](https://img.shields.io/crates/v/wary)](https://crates.io/crates/wary)
[![<https://img.shields.io/docsrs/wary>](https://img.shields.io/docsrs/wary)](https://docs.rs/wary/latest/wary/)
[![ci status](https://github.com/matteopolak/wary/workflows/ci/badge.svg)](https://github.com/matteopolak/wary/actions)

An optionally `no_std` and `no_alloc` validation and transformation library.

### Why use `wary` over other libraries?

| - | `wary` | `garde` | `validator` | `validify` |
| - | - | - | - | - |
| `no_std` | ✅ | ❌ | ❌ || ❌ |
| `no_alloc` | ✅ | ❌ | ❌ | ❌ |
| async | ✅ (optional) | ❌ | ❌ | ❌ |
| enums | ✅ | ✅ | ❌ | ✅ |
| transform input | ✅ | ❌ | ❌ | ✅ |
| custom rules | ✅ | ✅ | ✅ | ✅ |
| pass context | ✅ | ✅ | ✅ | ❌ |
| respect `serde` field attributes | ✅ | ❌ | ❌ | ❌ |

- Basic usage
  - [Basic struct example](#basic-struct-example)
  - [Basic enum example](#basic-enum-example)
- [Accessing context](#context)
- [Validation rules](#validation-rules)
  - [Implementing custom `Rule`s](#rule-custom)
  - [Implementing `Validate` manually](#manual-validate)
- [Transformation rules](#transformation-rules)
  - [Implementing custom `Transformer`s](#transformer-custom)
  - [Implementing `Transform` manually](#manual-transform)
- [Async support](#async)

### Basic struct example

```rust
use std::borrow::Cow;
use wary::Wary;

#[derive(Wary)]
#[wary(transparent)]
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
  #[transform(lowercase(ascii))]
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

| rule | trait | feature | dependency |
| ---- | ----- | ------- | ---------- |
| [`addr`](#rule-addr) | [`AsRef<str>`](wary::AsRef) | - | - |
| [`alphanumeric`](#rule-alphanumeric) | [`AsRef<str>`](wary::AsRef) | - | - |
| [`and`](#rule-and) | - | - | - |
| [`ascii`](#rule-ascii) | [`AsRef<str>`](wary::AsRef) | - | - |
| [`contains`](#rule-contains) | [`AsSlice`](wary::AsSlice) | - | - |
| [`credit_card`](#rule-credit-card) | [`AsRef<str>`](wary::AsRef) | `credit_card` | [`creditcard`](https://github.com/matteopolak/creditcard) |
| [`custom`](#rule-custom) | [`Rule<T>`](wary::Rule) | - | - |
| [`dive`](#rule-dive) | [`Validate`](wary::Validate) | - | - |
| [`email`](#rule-email) | [`AsRef<str>`](wary::AsRef) | `email` | [`email_address`](https://github.com/johnstonskj/rust-email_address) |
| [`equals`](#rule-equals) | [`std::cmp::PartialEq`](std::cmp::PartialEq) | - | - |
| [`func`](#rule-func) | `Fn(&T) -> Result<(), wary::Error>` | - | - |
| [`inner`](#rule-inner) | [`AsSlice`](wary::AsSlice) | - | - |
| [`length`](#rule-length) | [`Length`](wary::Length) | `graphemes`\* | [`unicode-segmentation`](https://github.com/unicode-rs/unicode-segmentation) |
| [`lowercase`](#rule-lowercase) | [`AsRef<str>`](wary::AsRef) | - | - |
| [`or`](#rule-or) | - | - | - |
| [`prefix`](#rule-prefix) | [`AsSlice`](wary::AsSlice) | - | - |
| [`range`](#rule-range) | [`Compare`](wary::Compare) | - | - |
| [`regex`](#rule-regex) | [`AsRef<str>`](wary::AsRef) | `regex` | [`regex`](https://github.com/rust-lang/regex) |
| [`required`](#rule-required) | [`AsSlice`](wary::AsSlice) | - | - |
| [`semver`](#rule-semver) | [`AsRef<str>`](wary::AsRef) | `semver` | [`semver`](https://github.com/dtolnay/semver) |
| [`suffix`](#rule-suffix) | [`AsSlice`](wary::AsSlice) | - | - |
| [`time`](#rule-time) | - | - | [`jiff`](https://github.com/BurntSushi/jiff) or [`chrono`](https://github.com/chronotope/chrono) |
| [`uppercase`](#rule-uppercase) | [`AsRef<str>`](wary::AsRef) | - | - |
| [`url`](#rule-url) | [`AsRef<str>`](wary::AsRef) | `url` | [`url`](https://github.com/servo/rust-url) |
| [`uuid`](#rule-uuid) | [`AsRef<str>`](wary::AsRef) | `uuid` | [`uuid`](https://github.com/uuid-rs/uuid) |

\* optional

### `addr` <a id="rule-addr"></a>

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

### `alphanumeric` <a id="rule-alphanumeric"></a>

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

### `and` <a id="rule-and"></a>

Meta-rule that combines multiple rules. Unlike other rule lists, this one **short-circuits on the first error**.

```rust
use wary::{Wary, Validate};

#[derive(Wary)]
struct NameAnd {
  #[validate(and(equals(other = 1), range(2..=2)))]
  value: u8
}

let name = NameAnd {
  value: 3,
};

let report = name.validate(&()).unwrap_err();

assert_eq!(report.len(), 1);

#[derive(Wary)]
struct Name {
  #[validate(equals(other = 1), range(2..=2))]
  value: u8
}

let name = Name {
  value: 3,
};

let report = name.validate(&()).unwrap_err();

assert_eq!(report.len(), 2);
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

### `credit_card` (requires feature `credit_card`) <a id="rule-credit-card"></a>

Validates that the input is a credit card number (PAN).

```rust
use wary::Wary;

#[derive(Wary)]
struct Card(
  #[validate(credit_card)]
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

### `or` <a id="rule-or"></a>

Meta-rule that combines multiple rules. Short-circuits on the first success.

```rust
use wary::{Wary, Validate};
use std::sync::atomic::{AtomicUsize, Ordering};

mod rule {
  pub type debug = super::DebugRule;
}

struct DebugRule;

impl DebugRule {
  fn new() -> Self {
    Self
  }
}

static DEBUG_COUNTER: AtomicUsize = AtomicUsize::new(0);

impl<I> wary::Rule<I> for DebugRule {
  type Context = ();

  fn validate(&self, _ctx: &Self::Context, item: &I) -> Result<(), wary::Error> {
    DEBUG_COUNTER.fetch_add(1, Ordering::Relaxed);
    Ok(())
  }
}

#[derive(Wary)]
struct NameOr {
  #[validate(or(equals(other = 1), custom(debug)))]
  value: u8
}

# fn main() {
let name = NameOr {
  value: 1,
};

let report = name.validate(&()).unwrap();

assert_eq!(DEBUG_COUNTER.load(Ordering::Relaxed), 0);
# }
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

### `time` (requires feature `chrono` or `jiff`) <a id="rule-time"></a>

Validates that the input time is within a range.

```rust
use wary::Wary;
use jiff::Zoned;

#[derive(Wary)]
struct Time(
  #[validate(time(after = Zoned::now()))]
  Zoned
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

### `uuid` (requires feature `uuid`) <a id="rule-uuid"></a>

Validates that the input is a uuid.

```rust
use wary::Wary;

#[derive(Wary)]
struct Uuid(
  #[validate(uuid)]
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

Transformation rules are applied similarly to validation rules, but are implemented in the [`Transform`](wary::Transform) trait instead.

| rule | trait | feature | dependency |
| ---- | ----- | ------- | ---------- |
| [`custom`](#transformer-custom) | [`Transformer`](wary::Transformer) | - | - |
| [`dive`](#transformer-dive) | [`Transform`](wary::Transform) | - | - |
| [`lowercase`](#transformer-lowercase) | [`AsMut<str>`](wary::AsMut) (for `ascii` only) | - | - |
| [`inner`](#transformer-inner) | [`AsMutSlice`](wary::AsMutSlice) | - | - |
| [`uppercase`](#transformer-uppercase) | [`AsMut<str>`](wary::AsMut) (for `ascii` only) | - | - |

### `custom` <a id="transformer-custom"></a>

Transforms the input with a custom [`Transformer`](wary::transformer).

```rust
use wary::{Wary, Transformer};

struct SecretTransformer;

impl SecretTransformer {
  fn new() -> Self {
    Self
  }
}

impl Transformer<String> for SecretTransformer {
  type Context = ();

  fn transform(&self, _ctx: &Self::Context, item: &mut String) {
    item.clear();
    item.push_str("secret");
  }
}

#[allow(non_camel_case_types)]
mod transformer {
  pub type secret = super::SecretTransformer;
}

#[derive(Wary)]
struct Person {
  #[transform(custom(secret))]
  name: String,
}

# fn main() {}
```

### `dive` <a id="transformer-dive"></a>

Transforms the inner fields of a struct or enum.

```rust
use wary::Wary;

#[derive(Wary)]
struct Item {
  #[transform(lowercase)]
  name: String,
}

#[derive(Wary)]
struct Name {
  #[transform(dive)]
  item: Item,
}
```

### `lowercase` <a id="transformer-lowercase"></a>

Transforms the input to lowercase.

```rust
use wary::Wary;

#[derive(Wary)]
struct Name {
  #[transform(lowercase)]
  left: String,
  #[transform(lowercase(ascii))]
  right: String,
}
```

### `inner` <a id="transformer-inner"></a>

Transforms the inner fields of a slice-like type.

```rust
use wary::Wary;

#[derive(Wary)]
struct Name {
  #[transform(inner(lowercase))]
  items: Vec<String>,
}
```

### `uppercase` <a id="transformer-uppercase"></a>

Transforms the input to uppercase.

```rust
use wary::Wary;

#[derive(Wary)]
struct Name {
  #[transform(uppercase)]
  left: String,
  #[transform(uppercase(ascii))]
  right: String,
}
```

### Implementing `Transform` manually <a id="manual-transform"></a>

```rust
use wary::Transform;

struct Name {
  value: String,
}

impl Transform for Name {
  type Context = ();

  fn transform(&mut self, _ctx: &Self::Context) {
    self.value.make_ascii_lowercase();
  }
}

let mut name = Name {
  value: "Jane".to_string(),
};

name.transform(&());

assert_eq!(name.value, "jane");
```

### Async support <a id="async"></a>

Wary supports async validation and transformation out of the box. This is useful for cases where a validation step may need to reach out to a database or an external service.

All traits have an async variant:

- [`Wary`](wary::Wary) -> [`AsyncWary`](wary::AsyncWary)
- [`Validate`](wary::Validate) -> [`AsyncValidate`](wary::AsyncValidate)
- [`Rule`](wary::Rule) -> [`AsyncRule`](wary::AsyncRule)
- [`Transform`](wary::Transform) -> [`AsyncTransform`](wary::AsyncTransform)
- [`Transformer`](wary::Transformer) -> [`AsyncTransformer`](wary::AsyncTransformer)

```rust
use wary::{Wary, AsyncWary, AsyncTransformer};

struct SecretTransformer;

impl SecretTransformer {
  const fn new() -> Self {
    Self
  }
}

impl AsyncTransformer<String> for SecretTransformer {
  type Context = ();

  async fn transform_async(&self, _ctx: &Self::Context, item: &mut String) {
    item.clear();
    item.push_str("secret");
  }
}

#[allow(non_camel_case_types)]
mod transformer {
  pub type secret = super::SecretTransformer;
}

#[derive(Wary)]
struct Person {
  #[transform(custom_async(secret))]
  name: String,
}

#[pollster::main]
async fn main() {
  let mut person = Person {
    name: "hello".into(),
  };

  person.wary_async(&()).await;

  assert_eq!(person.name, "secret");
}
```
