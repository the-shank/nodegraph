#![warn(clippy::std_instead_of_alloc)]
#![warn(clippy::std_instead_of_core)]

// #![no_std]

mod error;
mod graph;

pub use self::{error::*, graph::*};
