#![doc = include_str!("../README.md")]
#![warn(unsafe_code)]
#![cfg_attr(not(feature = "std"), no_std)]
#![allow(rustdoc::private_intra_doc_links)]

mod draw_target;
mod parser;
mod style;
mod utils;

pub use parser::{load_pcf_font, Error, PcfFont};
pub use style::{PcfFontStyle, PcfFontStyleBuilder};
