use embedded_graphics_simulator::{
    BinaryColorTheme, OutputSettingsBuilder, SimulatorDisplay, Window,
};

use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::*,
    text::{Alignment, Baseline, Text, TextStyleBuilder},
};

use embedded_pcf::{load_pcf_font, PcfFont, PcfFontStyle};

use std::io::Cursor;

/// Big endian, glyph row padded to int(4 bytes)
const FONT_VARIABLE: &[u8] =
    include_bytes!("../test-fonts/fusion-pixel-12px-proportional-zh_hans-pad_to_int.pcf");
/// Mono font, big endian, glyph row padded to byte
const FONT_MONO: &[u8] = include_bytes!("../test-fonts/fusion-pixel-12px-monospaced-zh_hans.pcf");

fn main() {
    let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(320, 240));

    let font = load_pcf_font(Cursor::new(FONT_VARIABLE)).unwrap();
    let mut font_style = PcfFontStyle::<_, Rgb565>::new(&font);
    font_style.set_text_color(Rgb565::WHITE);
    // font_style.set_background_color(Rgb565::BLACK);

    let centered = TextStyleBuilder::new()
        .baseline(Baseline::Alphabetic)
        .alignment(Alignment::Center)
        .build();

    Text::with_text_style("World, hello! 世界，你好！", display.bounding_box().center(), font_style, centered)
        .draw(&mut display)
        .unwrap();

    // Uncomment one of the `theme` lines to use a different theme.
    let output_settings = OutputSettingsBuilder::new()
        //.theme(BinaryColorTheme::LcdGreen)
        //.theme(BinaryColorTheme::LcdWhite)
        .theme(BinaryColorTheme::LcdBlue)
        //.theme(BinaryColorTheme::OledBlue)
        //.theme(BinaryColorTheme::OledWhite)
        .build();

    let mut window = Window::new("World, hello!", &output_settings);
    window.show_static(&display);
}
