//! The ellipse primitive

use crate::{
    drawable::{Drawable, Pixel},
    geometry::{Dimensions, Point, Size},
    pixelcolor::PixelColor,
    primitives::{ContainsPoint, Primitive, Rectangle, Styled},
    style::PrimitiveStyle,
    transform::Transform,
    DrawTarget,
};

/// Ellipse primitive
///
/// # Examples
///
/// The [macro examples](../../macro.egellipse.html) make for more concise code.
///
/// ## Create some ellipses with different styles
///
/// ```rust
/// use embedded_graphics::{
///     pixelcolor::Rgb565,
///     prelude::*,
///     primitives::Ellipse,
///     style::{PrimitiveStyle, PrimitiveStyleBuilder},
/// };
/// # use embedded_graphics::mock_display::MockDisplay;
/// # let mut display = MockDisplay::default();
///
/// // Ellipse with 1 pixel wide white stroke with top-left point at (10, 20) with a size of (30, 50)
/// Ellipse::new(Point::new(10, 20), Size::new(30, 50))
///     .into_styled(PrimitiveStyle::with_stroke(Rgb565::WHITE, 1))
///     .draw(&mut display)?;
///
/// // Ellipse with styled stroke and fill with top-left point at (50, 20) with a size of (40, 30)
/// let style = PrimitiveStyleBuilder::new()
///     .stroke_color(Rgb565::RED)
///     .stroke_width(3)
///     .fill_color(Rgb565::GREEN)
///     .build();
///
/// Ellipse::new(Point::new(50, 20), Size::new(40, 30))
///     .into_styled(style)
///     .draw(&mut display)?;
///
/// // Ellipse with blue fill and no stroke with a translation applied
/// Ellipse::new(Point::new(10, 20), Size::new(20, 40))
///     .translate(Point::new(65, 35))
///     .into_styled(PrimitiveStyle::with_fill(Rgb565::BLUE))
///     .draw(&mut display)?;
/// # Ok::<(), core::convert::Infallible>(())
/// ```
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct Ellipse {
    /// Top-left point of ellipse's bounding box
    pub top_left: Point,

    /// Size of the ellipse
    pub size: Size,
}

impl Ellipse {
    /// Create a new ellipse delimited with a top-left point with a specific size
    pub const fn new(top_left: Point, size: Size) -> Self {
        Ellipse { top_left, size }
    }

    /// Create a new ellipse centered around a given point with a specific size
    pub fn with_center(center: Point, size: Size) -> Self {
        let offset = size.saturating_sub(Size::new(1, 1)) / 2;
        let top_left = center - offset;

        Ellipse { top_left, size }
    }

    /// Return the center point of the ellipse
    pub fn center(&self) -> Point {
        self.bounding_box().center()
    }

    /// Return the center point of the ellipse scaled by a factor of 2
    ///
    /// This method is used to accurately calculate the outside edge of the ellipse.
    /// The result is not equivalent to `self.center() * 2` because of rounding.
    fn center_2x(&self) -> Point {
        let radius = self.size.saturating_sub(Size::new(1, 1));

        self.top_left * 2 + radius
    }
}

impl Primitive for Ellipse {
    type PointsIter = Points;

    fn points(&self) -> Self::PointsIter {
        Points::new(self)
    }
}

impl ContainsPoint for Ellipse {
    fn contains(&self, point: Point) -> bool {
        size_to_threshold(
            Size::new(self.size.width.pow(2), self.size.height.pow(2)),
            point * 2 - self.center_2x(),
            self.size.width,
        )
    }
}

impl Dimensions for Ellipse {
    fn bounding_box(&self) -> Rectangle {
        Rectangle::new(self.top_left, self.size)
    }
}

impl Transform for Ellipse {
    /// Translate the ellipse from its current position to a new position by (x, y) pixels,
    /// returning a new `Ellipse`. For a mutating transform, see `translate_mut`.
    ///
    /// ```
    /// # use embedded_graphics::primitives::Ellipse;
    /// # use embedded_graphics::prelude::*;
    /// let ellipse = Ellipse::new(Point::new(5, 10), Size::new(10, 15));
    /// let moved = ellipse.translate(Point::new(10, 10));
    ///
    /// assert_eq!(moved.top_left, Point::new(15, 20));
    /// ```
    fn translate(&self, by: Point) -> Self {
        Self {
            top_left: self.top_left + by,
            ..*self
        }
    }

    /// Translate the ellipse from its current position to a new position by (x, y) pixels.
    ///
    /// ```
    /// # use embedded_graphics::primitives::Ellipse;
    /// # use embedded_graphics::prelude::*;
    /// let mut ellipse = Ellipse::new(Point::new(5, 10), Size::new(10, 15));
    /// ellipse.translate_mut(Point::new(10, 10));
    ///
    /// assert_eq!(ellipse.top_left, Point::new(15, 20));
    /// ```
    fn translate_mut(&mut self, by: Point) -> &mut Self {
        self.top_left += by;

        self
    }
}

/// Iterator over all points inside the ellipse
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Points {
    ellipse: Ellipse,
    iter: super::rectangle::Points,
}

impl Points {
    fn new(ellipse: &Ellipse) -> Self {
        Self {
            ellipse: *ellipse,
            iter: ellipse.bounding_box().points(),
        }
    }

    fn empty() -> Self {
        Self {
            ellipse: Ellipse::new(Point::zero(), Size::zero()),
            iter: Rectangle::new(Point::zero(), Size::zero()).points(),
        }
    }
}

impl Iterator for Points {
    type Item = Point;

    fn next(&mut self) -> Option<Self::Item> {
        let ellipse = self.ellipse;

        self.iter.find(|point| ellipse.contains(*point))
    }
}

/// Pixel iterator for each pixel in the ellipse border
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct StyledEllipseIterator<C>
where
    C: PixelColor,
{
    iter: Points,
    outer_size: Size,
    outer_color: Option<C>,
    inner_size: Size,
    inner_color: Option<C>,
    center: Point,
    inner_width: u32,
}

impl<C> StyledEllipseIterator<C>
where
    C: PixelColor,
{
    fn new(styled: &Styled<Ellipse, PrimitiveStyle<C>>) -> Self {
        // Always use a stroke width of 0 if no stroke color was set.
        let stroke_width = styled.style.effective_stroke_width();

        let outer_size = styled.primitive.size;
        let inner_size = outer_size.saturating_sub(Size::new(2 * stroke_width, 2 * stroke_width));

        let inner_width = inner_size.width;

        let inner_size = Size::new(inner_size.width.pow(2), inner_size.height.pow(2));
        let iter = if !styled.style.is_transparent() {
            Points::new(&styled.primitive)
        } else {
            Points::empty()
        };

        Self {
            iter,
            outer_size,
            outer_color: styled.style.stroke_color,
            inner_size,
            inner_color: styled.style.fill_color,
            center: styled.primitive.center_2x(),
            inner_width,
        }
    }
}

impl<C> Iterator for StyledEllipseIterator<C>
where
    C: PixelColor,
{
    type Item = Pixel<C>;

    fn next(&mut self) -> Option<Self::Item> {
        let Self {
            inner_size,
            inner_color,
            outer_color,
            center,
            inner_width,
            ..
        } = *self;

        self.iter.find_map(|point| {
            let inner_threshold = size_to_threshold(inner_size, point * 2 - center, inner_width);

            let color = if inner_threshold {
                inner_color
            } else {
                outer_color
            };

            color.map(|color| Pixel(point, color))
        })
    }
}

impl<'a, C: 'a> Drawable<C> for &Styled<Ellipse, PrimitiveStyle<C>>
where
    C: PixelColor,
{
    fn draw<D: DrawTarget<C>>(self, display: &mut D) -> Result<(), D::Error> {
        display.draw_iter(self)
    }
}

/// Uses the ellipse equation b^2 * x^2 + a^2 * y^2 - a^2 * b^2 to return a value signifying whether
/// a given point lies inside (`true`) or outside (`false`) an ellipse centered around `(0, 0)` with
/// width and height defined by the `size` parameter.
fn size_to_threshold(size: Size, point: Point, diameter: u32) -> bool {
    let Size {
        width: a,
        height: b,
    } = size;

    let Point { x, y } = point;

    let x = x.pow(2);
    let y = y.pow(2);

    // Special case for circles, where width and height are equal
    if a == b {
        let threshold = if diameter <= 4 {
            diameter.pow(2) - diameter / 2
        } else {
            diameter.pow(2)
        } as i32;

        (x + y) < threshold
    } else {
        let a = a as i32;
        let b = b as i32;

        b * x + a * y - b * a <= 0
    }
}

impl<'a, C> IntoIterator for &'a Styled<Ellipse, PrimitiveStyle<C>>
where
    C: PixelColor,
{
    type Item = Pixel<C>;
    type IntoIter = StyledEllipseIterator<C>;

    fn into_iter(self) -> Self::IntoIter {
        StyledEllipseIterator::new(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        mock_display::MockDisplay, pixelcolor::BinaryColor, primitives::Circle,
        style::PrimitiveStyleBuilder,
    };

    fn test_ellipse(size: Size, style: PrimitiveStyle<BinaryColor>, pattern: &[&str]) {
        let mut display = MockDisplay::new();

        Ellipse::new(Point::new(0, 0), size)
            .into_styled(style)
            .draw(&mut display)
            .unwrap();

        assert_eq!(display, MockDisplay::from_pattern(pattern));
    }

    fn test_circles(style: PrimitiveStyle<BinaryColor>) {
        for diameter in 1..50 {
            let mut expected = MockDisplay::new();
            Circle::new(Point::new(0, 0), diameter)
                .into_styled(style)
                .draw(&mut expected)
                .unwrap();

            let mut display = MockDisplay::new();
            Ellipse::new(Point::new(0, 0), Size::new(diameter, diameter))
                .into_styled(style)
                .draw(&mut display)
                .unwrap();

            assert_eq!(display, expected, "diameter = {}", diameter);
        }
    }

    #[test]
    fn contains() {
        let mut expected = MockDisplay::new();
        let ellipse = Ellipse::new(Point::zero(), Size::new(40, 20));

        let mut display = MockDisplay::new();

        Rectangle::new(Point::zero(), Size::new(40, 20))
            .points()
            .filter(|p| ellipse.contains(*p))
            .map(|p| Pixel(p, BinaryColor::On))
            .draw(&mut display)
            .unwrap();

        ellipse
            .points()
            .map(|p| Pixel(p, BinaryColor::On))
            .draw(&mut expected)
            .unwrap();

        assert_eq!(display, expected);
    }

    #[test]
    fn circles_points() {
        for diameter in 1..50 {
            let circle_points = Circle::new(Point::new(0, 0), diameter).points();

            let ellipse_points =
                Ellipse::new(Point::new(0, 0), Size::new(diameter, diameter)).points();

            assert!(circle_points.eq(ellipse_points), "diameter = {}", diameter);
        }
    }

    #[test]
    fn ellipse_equals_circle_fill() {
        test_circles(PrimitiveStyle::with_fill(BinaryColor::On));
    }

    #[test]
    fn ellipse_equals_circle_stroke_1px() {
        test_circles(PrimitiveStyle::with_stroke(BinaryColor::On, 1));
    }

    #[test]
    fn ellipse_equals_circle_stroke_10px() {
        test_circles(PrimitiveStyle::with_stroke(BinaryColor::On, 10));
    }

    #[test]
    fn filled_ellipse() {
        #[rustfmt::skip]
        test_ellipse(Size::new(20, 10), PrimitiveStyle::with_fill(BinaryColor::On), &[
            "      ########      ",
            "   ##############   ",
            " ################## ",
            "####################",
            "####################",
            "####################",
            "####################",
            " ################## ",
            "   ##############   ",
            "      ########      ",
        ],);
    }

    #[test]
    fn thin_stroked_ellipse() {
        #[rustfmt::skip]
        test_ellipse(Size::new(20, 10), PrimitiveStyle::with_stroke(BinaryColor::On, 1), &[
            "      ########      ",
            "   ###        ###   ",
            " ##              ## ",
            "##                ##",
            "#                  #",
            "#                  #",
            "##                ##",
            " ##              ## ",
            "   ###        ###   ",
            "      ########      ",
        ],);
    }

    #[test]
    fn fill_and_stroke() {
        test_ellipse(
            Size::new(20, 10),
            PrimitiveStyleBuilder::new()
                .stroke_width(3)
                .stroke_color(BinaryColor::Off)
                .fill_color(BinaryColor::On)
                .build(),
            &[
                "      ........      ",
                "   ..............   ",
                " .................. ",
                ".....##########.....",
                "...##############...",
                "...##############...",
                ".....##########.....",
                " .................. ",
                "   ..............   ",
                "      ........      ",
            ],
        );
    }
}
