#[cfg(feature = "email")]
pub mod email;
#[cfg(feature = "email")]
pub use email::{Email, EmailRule};
pub mod length;
pub use length::{Length, LengthError, LengthRule};
pub mod range;
pub use range::{RangeError, RangeRule};
#[cfg(feature = "url")]
pub mod url;
#[cfg(feature = "url")]
pub use url::{Url, UrlRule};
