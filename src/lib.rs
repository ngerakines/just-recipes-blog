#![forbid(unsafe_code, future_incompatible)]
#![deny(missing_debug_implementations, bad_style)]

#[macro_use]
extern crate log;

extern crate slugify;

pub mod model;
pub mod site;
pub mod template;
pub mod when;

#[cfg(feature = "validate")]
pub mod validate;

#[cfg(feature = "convert")]
pub mod image;
