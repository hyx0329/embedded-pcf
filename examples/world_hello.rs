use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay, Window};

use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::*,
    text::{Alignment, Baseline, Text, TextStyleBuilder},
};

use embedded_pcf::{load_pcf_font, PcfFontStyleBuilder};

use std::io::Cursor;

/// Big endian, glyph row padded to int(4 bytes)
const FONT_VARIABLE: &[u8] =
    include_bytes!("../test-fonts/fusion-pixel-12px-proportional-zh_hans-pad_to_int.pcf");
/// Mono font, big endian, glyph row padded to single byte
const FONT_MONO: &[u8] = include_bytes!("../test-fonts/fusion-pixel-12px-monospaced-zh_hans.pcf");

fn main() {
    let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(200, 60));

    let cn_font_vari = load_pcf_font(Cursor::new(FONT_VARIABLE)).unwrap();
    let cn_font_vari_style = PcfFontStyleBuilder::new(&cn_font_vari)
        .text_color(Rgb565::WHITE)
        .background_color(Rgb565::BLACK)
        .build();

    let cn_font_mono = load_pcf_font(Cursor::new(FONT_MONO)).unwrap();
    let cn_font_mono_style = PcfFontStyleBuilder::new(&cn_font_mono)
        .text_color(Rgb565::WHITE)
        .background_color(Rgb565::BLACK)
        .build();

    let text_style = TextStyleBuilder::new()
        .baseline(Baseline::Middle)
        .alignment(Alignment::Center)
        .build();

    let world_hello = "世界，你好！World, hello!";
    let vari_position = display.bounding_box().center() - Point::new(0, 15);
    let mono_position = display.bounding_box().center() + Point::new(0, 15);

    Text::with_text_style(world_hello, vari_position, cn_font_vari_style, text_style)
        .draw(&mut display)
        .unwrap();

    Text::with_text_style(world_hello, mono_position, cn_font_mono_style, text_style)
        .draw(&mut display)
        .unwrap();

    let output_settings = OutputSettingsBuilder::new()
        .scale(2)
        .pixel_spacing(2)
        .build();

    let mut window = Window::new("World, hello!", &output_settings);
    window.show_static(&display);
}
