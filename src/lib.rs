#![doc = include_str!("../README.md")]
#![warn(unsafe_code)]
#![cfg_attr(not(feature = "std"), no_std)]
#![allow(rustdoc::private_intra_doc_links)]

mod draw_target;
pub mod pcf;
mod utils;

pub use pcf::{load_pcf_font, Error, PcfFont, PcfFontStyle};
