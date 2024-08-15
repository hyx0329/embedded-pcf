use embedded_graphics_simulator::{
    BinaryColorTheme, OutputSettingsBuilder, SimulatorDisplay, Window,
};

use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle}, pixelcolor::Rgb565, prelude::*, primitives::{Line, PrimitiveStyle}, text::{Alignment, Baseline, Text, TextStyleBuilder}
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

    let cn_font = load_pcf_font(Cursor::new(FONT_VARIABLE)).unwrap();
    let mut cn_font_style = PcfFontStyle::<_, Rgb565>::new(&cn_font);
    cn_font_style.set_text_color(Rgb565::WHITE);
    cn_font_style.set_background_color(Rgb565::BLACK);

    let en_font_style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);

    let centered_bottom = TextStyleBuilder::new()
        .baseline(Baseline::Bottom)
        .alignment(Alignment::Center)
        .build();

    let centered_middle = TextStyleBuilder::new()
        .baseline(Baseline::Middle)
        .alignment(Alignment::Center)
        .build();

    let centered_top = TextStyleBuilder::new()
        .baseline(Baseline::Top)
        .alignment(Alignment::Center)
        .build();

    let centered_alpha = TextStyleBuilder::new()
        .baseline(Baseline::Alphabetic)
        .alignment(Alignment::Center)
        .build();

    let cn_center = 50;
    for (i, style) in [centered_bottom, centered_middle, centered_top, centered_alpha].iter().enumerate() {
        Text::with_text_style(
            "World! 嗨！",
            Point::new(64 * (i as i32 +1), cn_center),
            cn_font_style.clone(),
            *style,
        )
        .draw(&mut display)
        .unwrap();
    }

    let en_center = 190;
    for (i, style) in [centered_bottom, centered_middle, centered_top, centered_alpha].iter().enumerate() {
            Text::with_text_style(
                "World! Hi!",
                Point::new(64 * (i as i32 +1), en_center),
                en_font_style.clone(),
                *style,
            )
            .draw(&mut display)
            .unwrap();
        }

    Line::new(Point::new(0, cn_center), Point::new(320, cn_center))
        .into_styled(PrimitiveStyle::with_stroke(Rgb565::RED, 1))
        .draw(&mut display)
        .unwrap();

    Line::new(Point::new(0, en_center), Point::new(320, en_center))
        .into_styled(PrimitiveStyle::with_stroke(Rgb565::RED, 1))
        .draw(&mut display)
        .unwrap();

    Pixel(display.bounding_box().center(), Rgb565::GREEN)
        .draw(&mut display)
        .unwrap();

    // Uncomment one of the `theme` lines to use a different theme.
    let output_settings = OutputSettingsBuilder::new()
        //.theme(BinaryColorTheme::LcdGreen)
        //.theme(BinaryColorTheme::LcdWhite)
        // .theme(BinaryColorTheme::LcdBlue)
        //.theme(BinaryColorTheme::OledBlue)
        //.theme(BinaryColorTheme::OledWhite)
        .scale(4)
        .build();

    let mut window = Window::new("World, hello!", &output_settings);
    window.show_static(&display);
}
