# embedded-psf

A toy lib loading X11 Portable Compiled Font for embedded-graphics.

`no-std` is currently WIP. It's not working without std because of
the dependency of some IO traits([std::io::Seek], [std::io::Read]).

It's not a complete implementation. Some PCF fonts may not be supported.

Due to trait limitation on mutability in embedded-graphics, this lib cannot work with [embedded_graphics::text::Text].

## Motivation

I want to use bitmap fonts like what it does [on CircuitPython](https://github.com/adafruit/Adafruit_CircuitPython_Bitmap_Font).

## About PCF font

It contains:

- font metadata(name, description, etc.)
- major glyph information table(accelerator)
- the mapping table from code point to internal glyph index
- glyph lookup table
- per-glyph metrics table
- glyph names lookup table
- and others

*Read <https://formats.kaitai.io/pcf_font/index.html> and that's really an interesting tool.*

To read matching glyphs from PCF font file:

1. convert the code point to internal glyph index using the lookup table
    - it may be invalid though
1. use internal glyph index to read data
    - glyph location
    - glyph metrics
1. read glyph data according to the size information in glyph metrics data
1. convert the glyph data so it can be used by other programs

Glyphs in PCF may be orphaned, and so may the code points. Examples:

- Looking for a glyph index matching a code point in the lookup table may return 0xFFFF, indicating no matching glyph.
- The glyphs may be not associated with a code point at all.
    - the glyph tofu(.notdef) is not in unicode and thus cannot be matched directly
    - the glyphs have names, like tofu's name ".notdef"

In such cases, PCF is really not a compact font to be used on embedded devices.
However, it makes prototyping with full featured fonts much easier.

## Notes

This project only aims to read the glyphs in PCF fonts and interface with embedded-graphics.
Not all features are implemented.
