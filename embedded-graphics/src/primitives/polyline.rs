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
    line: Polyline<'a>,
    segment_iter: ThickLineIterator,
    start_idx: usize,
    end_idx: usize,
}

impl<'a> Iterator for PolylineIterator<'a> {
    type Item = Point;

    fn next(&mut self) -> Option<Self::Item> {
        // Break if we're past the end of the line
        if self.line.vertices.len() < 2 {
            return None;
        }

        if let Some(p) = self.segment_iter.next() {
            Some(p)
        } else {
            self.start_idx += 1;
            self.end_idx += 1;

            // Break if we've gone past the end of the line
            if self.end_idx == self.line.vertices.len() {
                return None;
            }

            self.segment_iter = ThickLineIterator::new(
                &Line::new(
                    self.line.vertices[self.start_idx],
                    self.line.vertices[self.end_idx],
                ),
                1,
            );

            Self::next(self)
        }
    }
}

impl<'a> IntoIterator for Polyline<'a> {
    type Item = Point;
    type IntoIter = PolylineIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        let start_idx = 0;
        let end_idx = 1;

        Self::IntoIter {
            start_idx,
            end_idx,
            line: self,
            segment_iter: ThickLineIterator::new(
                &Line::new(self.vertices[start_idx], self.vertices[end_idx]),
                1,
            ),
        }
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
