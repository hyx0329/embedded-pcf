use az::SaturatingAs as _;
use embedded_graphics::{
    image::{Image, ImageRaw},
    pixelcolor::BinaryColor,
    prelude::{DrawTarget, Drawable as _, PixelColor, Point, Size},
    primitives::Rectangle,
    text::{
        renderer::{CharacterStyle, TextMetrics, TextRenderer},
        Baseline, DecorationColor,
    },
};

use crate::{
    draw_target::{Background, Both, Foreground, MonoFontDrawTarget},
    parser::MetricsEntry,
    Error, PcfFont,
};

#[cfg(feature = "std")]
use std::io;

#[derive(Debug, PartialEq, Clone, Copy)]
#[non_exhaustive]
pub struct PcfFontStyle<'a, T, C> {
    pub text_color: Option<C>,
    pub background_color: Option<C>,
    pub underline_color: DecorationColor<C>,
    pub strikethrough_color: DecorationColor<C>,
    pub font: &'a PcfFont<T>,
}

impl<'a, T, C> PcfFontStyle<'a, T, C>
where
    T: io::Read + io::Seek + Clone,
    C: PixelColor,
{
    /// Initialize a PcfFontStyle, default all transparent/disabled
    pub fn new(font: &'a PcfFont<T>) -> Self {
        Self {
            text_color: None,
            background_color: None,
            underline_color: DecorationColor::None,
            strikethrough_color: DecorationColor::None,
            font,
        }
    }

    pub fn is_transparent(&self) -> bool {
        self.text_color.is_none()
            && self.background_color.is_none()
            && self.underline_color.is_none()
            && self.strikethrough_color.is_none()
    }

    /// the the glyphs drawing offset based on current baseline configuration.
    fn baseline_offset(&self, baseline: Baseline) -> i32 {
        // The `1`s to add are required to use lower edge as the alphabetic baseline,
        // matching other fonts behavior.
        match baseline {
            // Bounding box top pixel coincide with position pixel
            Baseline::Top => self.font.bounding_box.max_ascent as i32,
            // Bounding box bottom pixel coincide with position pixel
            Baseline::Bottom => (1 + self.font.bounding_box.max_descent) as i32,
            // The bottom edge of the position pixel split the bounding box to 2 halves, and the lower half may be bigger
            Baseline::Middle => {
                (1 + self.font.bounding_box.height / 2 + self.font.bounding_box.max_descent) as i32
            }
            // position pixel's lower edge coincide with font's baseline
            Baseline::Alphabetic => 1,
        }
    }

    fn draw_decorations<D>(
        &self,
        width: u32,
        position: Point,
        target: &mut D,
    ) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = C>,
    {
        let _ = width;
        let _ = position;
        let _ = target;
        // TODO: draw strike through
        // TODO: draw underline

        // strike through
        if let Some(color) = match self.strikethrough_color {
            DecorationColor::None => None,
            DecorationColor::Custom(custom_color) => Some(custom_color),
            DecorationColor::TextColor => self.text_color,
        } {
            let offset = Point::new(0, -self.baseline_offset(Baseline::Middle));
            let rect = Rectangle::new(position + offset, Size::new(width, 1));
            target.fill_solid(&rect, color)?;
        }

        // underline is drawn at the bounding box bottom edge
        if let Some(color) = match self.underline_color {
            DecorationColor::None => None,
            DecorationColor::Custom(custom_color) => Some(custom_color),
            DecorationColor::TextColor => self.text_color,
        } {
            let offset = Point::new(0, -self.baseline_offset(Baseline::Bottom));
            let rect = Rectangle::new(position + offset, Size::new(width, 1));
            target.fill_solid(&rect, color)?;
        }

        Ok(())
    }

    /// draw a single character at given position.
    #[inline]
    fn draw_single_char_binary<D>(
        &self,
        glyph_data: &[u8],
        metrics: MetricsEntry,
        position: Point,
        target: &mut D,
    ) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = BinaryColor>,
    {
        // draw glyph only if it has data
        if glyph_data.len() > 0 {
            // map a glyph and paint it
            let glyph = ImageRaw::<BinaryColor>::new(glyph_data, metrics.glyph_width() as u32);
            // per-glyph offset
            let offset = Point::new(
                metrics.left_side_bearing as i32,
                (-metrics.character_ascent) as i32,
            );
            Image::new(&glyph, position + offset).draw(target)?;
        }

        Ok(())
    }

    /// measure background rectangle
    /// this is somehow overlapped with `measure_string`
    fn text_bbox(&self, text: &str, position: Point) -> Option<Rectangle> {
        if text.is_empty() {
            None
        } else {
            // be careful about the drawing baseline 1px offset
            let offset = Point::new(0, -self.font.bounding_box.max_ascent as i32);
            let default_width = self.font.bounding_box.width as u32;
            // FIXME: for variable italic/styled fonts, the character_width may be smaller than right_side_bearing
            // Glyphs may exceed the right border.
            let bb_width = text
                .chars()
                .map(|c| match self.font.get_glyph_metrics(c as u16) {
                    Ok(metrics) => metrics.character_width as u32,
                    Err(_) => default_width,
                })
                .sum();

            let bb_size = Size::new(bb_width, self.font.bounding_box.height as u32);
            Some(Rectangle::new(position + offset, bb_size))
        }
    }

    /// Batch fill the background of the given string
    ///
    /// Glyphs doesn't necessarily contains full empty border to overwrite the old content.
    #[inline]
    fn fill_string_background<D>(
        &self,
        text: &str,
        position: Point,
        target: &mut D,
    ) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = BinaryColor>,
    {
        if self.background_color.is_some() {
            if let Some(background_bbox) = self.text_bbox(text, position) {
                target.fill_solid(&background_bbox, BinaryColor::Off)
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }

    /// Draw the string, binary color, alphabetic baseline is the upper edge of the given pixel/location.
    ///
    /// Be careful that embedded-graphics actually uses the lower edge of
    /// the given pixel/location as the alphabetic baseline.
    fn draw_string_binary<D>(
        &self,
        text: &str,
        mut position: Point,
        mut target: D,
    ) -> Result<Point, D::Error>
    where
        D: DrawTarget<Color = BinaryColor>,
    {
        /*
        We have only ascending & descending information and glyphs need offsets to be aligned
        at font's baseline(not the top of the visible pixels). This is done by substracting
        character_ascent(absolute value) from the Y-Axis while drawing each character.
        */

        // this buffer should be sufficient for glyphs size below 16*16
        // TODO: adapt STD
        let mut buf: [u8; 40] = [0; 40];
        self.fill_string_background(text, position, &mut target)?;
        for c in text.chars() {
            match self.font.read_glyph_raw(c as u16, &mut buf) {
                Ok((length, metrics)) => {
                    self.draw_single_char_binary(&buf[..length], metrics, position, &mut target)?;
                    position.x += metrics.character_width as i32;
                }
                Err(Error::NotFound) => {
                    // look for the default character to use
                    // TODO: add a switch to check default font
                    match self.font.read_glyph_raw(self.font.default_char, &mut buf) {
                        Ok((length, metrics)) => {
                            self.draw_single_char_binary(
                                &buf[..length],
                                metrics,
                                position,
                                &mut target,
                            )?;
                            position.x += metrics.character_width as i32;
                        }
                        _ => { /* Just ignore the rest, assuming those are 0-width */ }
                    }
                }
                _ => { /* Just ignore the rest, assuming those are 0-width */ }
            };
        }
        Ok(position)
    }
}

impl<T, C> TextRenderer for PcfFontStyle<'_, T, C>
where
    C: PixelColor,
    T: io::Read + io::Seek + Clone,
{
    type Color = C;

    fn draw_string<D>(
        &self,
        text: &str,
        position: Point,
        baseline: Baseline,
        target: &mut D,
    ) -> Result<Point, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        // apply baseline offset
        let position = position + Point::new(0, self.baseline_offset(baseline));

        let next = match (self.text_color, self.background_color) {
            (Some(text_color), Some(background_color)) => self.draw_string_binary(
                text,
                position,
                MonoFontDrawTarget::new(target, Both(text_color, background_color)),
            )?,
            (Some(text_color), None) => self.draw_string_binary(
                text,
                position,
                MonoFontDrawTarget::new(target, Foreground(text_color)),
            )?,
            (None, Some(background_color)) => self.draw_string_binary(
                text,
                position,
                MonoFontDrawTarget::new(target, Background(background_color)),
            )?,
            (None, None) => {
                let default_width = self.font.bounding_box.width as u32;
                let dx = text
                    .chars()
                    .map(|c| match self.font.get_glyph_metrics(c as u16) {
                        Ok(metrics) => metrics.character_width as u32,
                        Err(_) => default_width,
                    })
                    .sum();

                position + Size::new(dx, 0)
            }
        };

        if next.x > position.x {
            let width = (next.x - position.x) as u32;
            self.draw_decorations(width, position, target)?;
        }

        // restore baseline offset
        Ok(next - Point::new(0, self.baseline_offset(baseline)))
    }

    fn draw_whitespace<D>(
        &self,
        width: u32,
        mut position: Point,
        baseline: Baseline,
        target: &mut D,
    ) -> Result<Point, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        if width != 0 {
            let max_ascent =
                (self.font.bounding_box.height + self.font.bounding_box.max_descent) as i32;
            position.y += self.baseline_offset(baseline) - max_ascent;
            if let Some(background_color) = self.background_color {
                target.fill_solid(
                    &Rectangle::new(
                        position,
                        Size::new(width, self.font.bounding_box.height as u32),
                    ),
                    background_color,
                )?;
            }

            position.y += max_ascent;
            self.draw_decorations(width, position, target)?;
            position.y -= self.baseline_offset(baseline);
            position.x += width.saturating_as::<i32>();
            Ok(position)
        } else {
            Ok(position)
        }
    }

    fn measure_string(
        &self,
        text: &str,
        position: Point,
        baseline: Baseline,
    ) -> embedded_graphics::text::renderer::TextMetrics {
        let bbox = if let Some(mut bbox) = self.text_bbox(text, position) {
            bbox.top_left += Point::new(0, self.baseline_offset(baseline));
            bbox
        } else {
            let bb_position = position
                + Point::new(
                    0,
                    self.baseline_offset(baseline) - self.baseline_offset(Baseline::Top),
                );
            Rectangle::new(bb_position, Size::new(0, 0))
        };

        // current decoration(underline etc.) implementation doesn't affect height

        TextMetrics {
            bounding_box: bbox,
            next_position: position + bbox.size.x_axis(),
        }
    }

    fn line_height(&self) -> u32 {
        self.font.bounding_box.height as u32
    }
}

impl<T, C> CharacterStyle for PcfFontStyle<'_, T, C>
where
    C: PixelColor,
    T: io::Read + io::Seek + Clone,
{
    type Color = C;

    fn set_text_color(&mut self, text_color: Option<Self::Color>) {
        self.text_color = text_color;
    }

    fn set_background_color(&mut self, background_color: Option<Self::Color>) {
        self.background_color = background_color;
    }

    fn set_underline_color(&mut self, underline_color: DecorationColor<Self::Color>) {
        self.underline_color = underline_color;
    }

    fn set_strikethrough_color(&mut self, strikethrough_color: DecorationColor<Self::Color>) {
        self.strikethrough_color = strikethrough_color;
    }
}

/// Text style builder for PCF fonts.
///
/// Mostly copied from embedded_graphics/mono_font/mono_text_style.rs to maintain
/// API consistency.
#[derive(Copy, Clone, Debug)]
pub struct PcfFontStyleBuilder<'a, T, C> {
    style: PcfFontStyle<'a, T, C>,
}

impl<'a, T, C> PcfFontStyleBuilder<'a, T, C>
where
    C: PixelColor,
{
    /// Create a style builder with existing font
    ///
    /// Due to the implementation limit, a font must be provided.
    pub const fn new(font: &'a PcfFont<T>) -> Self {
        Self {
            style: PcfFontStyle {
                text_color: None,
                background_color: None,
                underline_color: DecorationColor::None,
                strikethrough_color: DecorationColor::None,
                font,
            },
        }
    }

    /// Enables underline using the text color.
    pub const fn underline(mut self) -> Self {
        self.style.underline_color = DecorationColor::TextColor;

        self
    }

    /// Enables strikethrough using the text color.
    pub const fn strikethrough(mut self) -> Self {
        self.style.strikethrough_color = DecorationColor::TextColor;

        self
    }

    /// Resets the text color to transparent.
    pub const fn reset_text_color(mut self) -> Self {
        self.style.text_color = None;

        self
    }

    /// Resets the background color to transparent.
    pub const fn reset_background_color(mut self) -> Self {
        self.style.background_color = None;

        self
    }

    /// Removes the underline decoration.
    pub const fn reset_underline(mut self) -> Self {
        self.style.underline_color = DecorationColor::None;

        self
    }

    /// Removes the strikethrough decoration.
    pub const fn reset_strikethrough(mut self) -> Self {
        self.style.strikethrough_color = DecorationColor::None;

        self
    }

    /// Sets the text color.
    pub const fn text_color(mut self, text_color: C) -> Self {
        self.style.text_color = Some(text_color);

        self
    }

    /// Sets the background color.
    pub const fn background_color(mut self, background_color: C) -> Self {
        self.style.background_color = Some(background_color);

        self
    }

    /// Enables underline with a custom color.
    pub const fn underline_with_color(mut self, underline_color: C) -> Self {
        self.style.underline_color = DecorationColor::Custom(underline_color);

        self
    }

    /// Enables strikethrough with a custom color.
    pub const fn strikethrough_with_color(mut self, strikethrough_color: C) -> Self {
        self.style.strikethrough_color = DecorationColor::Custom(strikethrough_color);

        self
    }

    /// Builds the text style.
    pub const fn build(self) -> PcfFontStyle<'a, T, C> {
        self.style
    }
}
