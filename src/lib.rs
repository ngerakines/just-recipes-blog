#![forbid(unsafe_code, future_incompatible)]
#![deny(missing_debug_implementations, bad_style)]

extern crate pretty_env_logger;
#[macro_use] extern crate log;

extern crate slugify;

pub mod model;
pub mod site;
pub mod template;

#[cfg(feature = "validate")]
pub mod validate;
