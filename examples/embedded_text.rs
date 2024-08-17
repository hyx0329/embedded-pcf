use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay, Window};
use embedded_graphics::{
    pixelcolor::BinaryColor, prelude::*, primitives::Rectangle
};
use embedded_text::{
    alignment::HorizontalAlignment,
    style::{HeightMode, TextBoxStyleBuilder},
    TextBox,
};

use embedded_pcf::{load_pcf_font, PcfFontStyleBuilder};

use std::io::Cursor;

/// Big endian, glyph row padded to int(4 bytes)
const FONT: &[u8] =
    include_bytes!("../test-fonts/fusion-pixel-12px-proportional-zh_hans-pad_to_int.pcf");
/// Mono font, big endian, glyph row padded to single byte
// const FONT: &[u8] = include_bytes!("../test-fonts/fusion-pixel-12px-monospaced-zh_hans.pcf");

fn main() {

    let text = "世界，你好！\n\
    我是说，我是世界，你好！这是一个自然段。每个自然段用换行符分隔。\n\
    那么，下次再见！";
    // let text = "Hello, World!\n\
    // A paragraph is a number of lines that end with a manual newline. Paragraph spacing is the \
    // number of pixels between two paragraphs.\n\
    // Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when \
    // an unknown printer took a galley of type and scrambled it to make a type specimen book.";

    let cn_font_vari = load_pcf_font(Cursor::new(FONT)).unwrap();
    let character_style = PcfFontStyleBuilder::new(&cn_font_vari)
        .text_color(BinaryColor::On)
        // .background_color(Rgb565::BLACK)
        .build();

    let textbox_style = TextBoxStyleBuilder::new()
        .height_mode(HeightMode::FitToText)
        .alignment(HorizontalAlignment::Justified)
        .paragraph_spacing(6)
        .build();

    let bounds = Rectangle::new(Point::zero(), Size::new(128, 0));
    let text_box = TextBox::with_textbox_style(text, bounds, character_style, textbox_style);

    let mut display = SimulatorDisplay::<BinaryColor>::new(text_box.bounding_box().size);
    text_box.draw(&mut display).unwrap();

    let output_settings = OutputSettingsBuilder::new()
        .scale(2)
        .pixel_spacing(2)
        .build();

    let mut window = Window::new("World, hello!", &output_settings);
    window.show_static(&display);
}
