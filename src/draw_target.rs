//! copied from embedded_graphics/mono_font/draw_target.rs

use embedded_graphics::{
    draw_target::DrawTarget, geometry::Dimensions, iterator::ContiguousIteratorExt,
    pixelcolor::BinaryColor, primitives::Rectangle, Pixel,
};

pub struct MonoFontDrawTarget<'a, T, C> {
    parent: &'a mut T,
    colors: C,
}

impl<'a, T: DrawTarget, C> MonoFontDrawTarget<'a, T, C> {
    pub fn new(parent: &'a mut T, colors: C) -> Self {
        Self { parent, colors }
    }
}

impl<T: DrawTarget> DrawTarget for MonoFontDrawTarget<'_, T, Foreground<T::Color>> {
    type Color = BinaryColor;
    type Error = T::Error;

    fn fill_contiguous<I>(&mut self, area: &Rectangle, colors: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Self::Color>,
    {
        let foreground_color = self.colors.0;

        self.parent.draw_iter(
            colors
                .into_iter()
                .into_pixels(area)
                .filter(|Pixel(_, color)| color.is_on())
                .map(|Pixel(pos, _)| Pixel(pos, foreground_color)),
        )
    }

    fn draw_iter<I>(&mut self, _pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        unreachable!()
    }

    fn fill_solid(&mut self, area: &Rectangle, color: Self::Color) -> Result<(), Self::Error> {
        match color {
            BinaryColor::On => self.parent.fill_solid(area, self.colors.0),
            BinaryColor::Off => Ok(()),
        }
    }

    fn clear(&mut self, _color: Self::Color) -> Result<(), Self::Error> {
        unreachable!()
    }
}

impl<T: DrawTarget> DrawTarget for MonoFontDrawTarget<'_, T, Background<T::Color>> {
    type Color = BinaryColor;
    type Error = T::Error;

    fn fill_contiguous<I>(&mut self, _area: &Rectangle, _colors: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Self::Color>,
    {
        Ok(()) // dont do anything, the background is filled by styling code
    }

    fn draw_iter<I>(&mut self, _pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        unreachable!()
    }

    fn fill_solid(&mut self, area: &Rectangle, color: Self::Color) -> Result<(), Self::Error> {
        match color {
            BinaryColor::On => Ok(()),
            BinaryColor::Off => self.parent.fill_solid(area, self.colors.0),
        }
    }

    fn clear(&mut self, _color: Self::Color) -> Result<(), Self::Error> {
        unreachable!()
    }
}

impl<T: DrawTarget> DrawTarget for MonoFontDrawTarget<'_, T, Both<T::Color>> {
    type Color = BinaryColor;
    type Error = T::Error;

    fn fill_contiguous<I>(&mut self, area: &Rectangle, colors: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Self::Color>,
    {
        let foreground_color = self.colors.0;

        // only fill foreground color as background is handled by styling code
        self.parent.draw_iter(
            colors
                .into_iter()
                .into_pixels(area)
                .filter(|Pixel(_, color)| color.is_on())
                .map(|Pixel(pos, _)| Pixel(pos, foreground_color)),
        )
    }

    fn draw_iter<I>(&mut self, _pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        unreachable!()
    }

    fn fill_solid(&mut self, area: &Rectangle, color: Self::Color) -> Result<(), Self::Error> {
        match color {
            BinaryColor::On => self.parent.fill_solid(area, self.colors.0),
            BinaryColor::Off => self.parent.fill_solid(area, self.colors.1),
        }
    }

    fn clear(&mut self, _color: Self::Color) -> Result<(), Self::Error> {
        unreachable!()
    }
}

impl<T: DrawTarget, C> Dimensions for MonoFontDrawTarget<'_, T, C> {
    fn bounding_box(&self) -> Rectangle {
        self.parent.bounding_box()
    }
}

pub struct Foreground<C>(pub C);
pub struct Background<C>(pub C);
pub struct Both<C>(pub C, pub C);
