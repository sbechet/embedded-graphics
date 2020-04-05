//! The line primitive

use crate::draw_target::DrawTarget;
use crate::drawable::Drawable;
use crate::drawable::Pixel;
use crate::geometry::Dimensions;
use crate::geometry::Size;
use crate::pixelcolor::PixelColor;
use crate::primitives::Primitive;
use crate::style::PrimitiveStyle;
use crate::style::Styled;
use crate::{
    geometry::Point,
    primitives::{line::Line, thick_line_iterator::ThickLineIterator},
};

/// Polyline primitive
///
/// Creates an unfilled chained line shape
///
/// # Examples
///
/// ## Create some lines with different styles
///
/// ```rust
/// use embedded_graphics::{
///     pixelcolor::Rgb565, prelude::*, primitives::Line, style::PrimitiveStyle,
/// };
/// # use embedded_graphics::mock_display::MockDisplay;
/// # let mut display = MockDisplay::default();
///
/// // TODO: Example
/// # Ok::<(), core::convert::Infallible>(())
/// ```
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct Polyline<'a> {
    /// All vertices in the line
    pub vertices: &'a [Point],
}

impl<'a> Polyline<'a> {
    /// Create a new polyline from a list of points or an iterator
    pub fn new(vertices: &'a [Point]) -> Self {
        Self { vertices }
    }
}

impl<'a> Primitive for Polyline<'a> {}

// TODO
impl<'a> Dimensions for Polyline<'a> {
    fn top_left(&self) -> Point {
        Point::zero()
    }

    fn bottom_right(&self) -> Point {
        self.top_left() + Point::zero()
    }

    fn size(&self) -> Size {
        Size::from_bounding_box(self.top_left(), self.bottom_right())
    }
}

/// TODO: Docs
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct PolylineIterator<'a> {
    stop: bool,
    vertices: &'a [Point],
    segment_iter: ThickLineIterator,
}

impl<'a> Iterator for PolylineIterator<'a> {
    type Item = Point;

    fn next(&mut self) -> Option<Self::Item> {
        if self.stop {
            return None;
        }

        if let Some(p) = self.segment_iter.next() {
            Some(p)
        } else {
            let (start, rest) = self.vertices.split_first()?;
            let end = rest.get(0)?;

            self.vertices = rest;

            self.segment_iter = ThickLineIterator::new(&Line::new(*start, *end), 1);

            Self::next(self)
        }
    }
}

impl<'a> IntoIterator for Polyline<'a> {
    type Item = Point;
    type IntoIter = PolylineIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.vertices
            .split_first()
            .and_then(|(start, rest)| {
                // Polyline is 2 or more vertices long, return an iterator for it
                rest.get(0).map(|end| Self::IntoIter {
                    stop: false,
                    vertices: rest,
                    segment_iter: ThickLineIterator::new(&Line::new(*start, *end), 1),
                })
            })
            .unwrap_or_else(||
                // Polyline is less than 2 vertices long. Return a dummy iterator that will short
                // circuit due to `stop: true`
                Self::IntoIter {
                    stop: true,
                    vertices: &[],
                    segment_iter: ThickLineIterator::new(&Line::new(Point::zero(), Point::zero()), 1),
                })
    }
}

impl<'a, C> IntoIterator for &'a Styled<Polyline<'a>, PrimitiveStyle<C>>
where
    C: PixelColor,
{
    type Item = Pixel<C>;
    type IntoIter = StyledPolylineIterator<'a, C>;

    fn into_iter(self) -> Self::IntoIter {
        StyledPolylineIterator {
            style: self.style,

            line_iter: self.primitive.into_iter(),
        }
    }
}

/// Pixel iterator for each pixel in the line
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct StyledPolylineIterator<'a, C>
where
    C: PixelColor,
{
    style: PrimitiveStyle<C>,

    line_iter: PolylineIterator<'a>,
}

// [Bresenham's line algorithm](https://en.wikipedia.org/wiki/Bresenham%27s_line_algorithm)
impl<'a, C: PixelColor> Iterator for StyledPolylineIterator<'a, C> {
    type Item = Pixel<C>;

    fn next(&mut self) -> Option<Self::Item> {
        // Break if stroke width is zero
        if self.style.stroke_width == 0 {
            return None;
        }

        // Return none if stroke color is none
        let stroke_color = self.style.stroke_color?;

        self.line_iter
            .next()
            .map(|point| Pixel(point, stroke_color))
    }
}

impl<'a, C: 'a> Drawable<C> for &Styled<Polyline<'a>, PrimitiveStyle<C>>
where
    C: PixelColor,
{
    fn draw<D: DrawTarget<C>>(self, display: &mut D) -> Result<(), D::Error> {
        display.draw_iter(self.into_iter())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{drawable::Pixel, pixelcolor::BinaryColor};

    // TODO
}
