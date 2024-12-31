#[cfg(feature = "email")]
pub mod email;
pub mod length;
pub mod range;
#[cfg(feature = "url")]
pub mod url;

pub mod func {}
pub mod custom {}
