#![feature(once_cell)]

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate maplit;

mod ser;

mod web;
pub use crate::web::*;

mod core;
pub use crate::core::*;

mod error;
pub use crate::error::*;
