//! Turning a selection drawn on the screen into a rectangle of pixels in a captured image.
//!
//! This is the arithmetic that decides which part of a screenshot the user actually gets, and it
//! is the part of capture that has nothing to do with a screen. It is therefore here, where it is
//! tested, rather than in the application.
//!
//! # Why the scale is measured rather than asked for
//!
//! A display at 150% has more pixels than it has points. Something has to convert between the
//! two, and the obvious source - what the capture library reports as the monitor's width - is not
//! trustworthy: `xcap` returns a **logical** width on Linux (it divides by the scale factor) and a
//! **physical** one on Windows (`dmPelsWidth`). Code written against either is wrong on the other,
//! and both look correct at 100% where the two are equal.
//!
//! So nothing here takes a scale factor as an input. The caller passes the size of the surface the
//! user drew on and the size of the image that was captured, and the ratio between them is the
//! scale - measured from what actually happened rather than from what a platform says about
//! itself.

use serde::{Deserialize, Serialize};

/// A rectangle in the coordinates the user drew in: points, not pixels, with the origin at the
/// top left of the surface they drew on.
///
/// Width and height may be negative. Dragging up and to the left is as ordinary as dragging down
/// and to the right, and the two produce the same rectangle.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Selection {
    /// Where the drag started, horizontally.
    pub x: f64,
    /// Where the drag started, vertically.
    pub y: f64,
    /// How far it went horizontally. Negative when it went left.
    pub width: f64,
    /// How far it went vertically. Negative when it went up.
    pub height: f64,
}

/// The size of something in points: a window, or a screen as the windowing system describes it.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LogicalSize {
    /// Width in points.
    pub width: f64,
    /// Height in points.
    pub height: f64,
}

/// The size of an image in pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PixelSize {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
}

/// Where a monitor sits on the desktop, and how many pixels it draws for each point.
///
/// A windowing system describes a monitor in physical pixels; the windows drawn on it, and the
/// pointer events they receive, are in points. Both are needed, so both are worked out from one
/// description rather than gathered separately, where they could disagree.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MonitorRect {
    /// Left edge, in physical pixels, in the desktop's coordinates.
    pub x: i32,
    /// Top edge, in physical pixels, in the desktop's coordinates.
    pub y: i32,
    /// Width in physical pixels.
    pub width: u32,
    /// Height in physical pixels.
    pub height: u32,
    /// Physical pixels per point. One on a display that is not scaled up.
    pub scale: f64,
}

impl MonitorRect {
    /// The middle of this monitor, in physical pixels.
    ///
    /// The middle rather than a corner, wherever a single point has to stand for the monitor: a
    /// corner is shared with the monitor next to it, and which of the two owns it is exactly the
    /// sort of thing platforms disagree about.
    #[must_use]
    pub fn centre_in_pixels(self) -> (i32, i32) {
        (
            self.x.saturating_add_unsigned(self.width / 2),
            self.y.saturating_add_unsigned(self.height / 2),
        )
    }

    /// The middle of this monitor, in points.
    #[must_use]
    pub fn centre_in_points(self) -> (i32, i32) {
        let scale = self.usable_scale();
        let (x, y) = self.centre_in_pixels();
        (to_whole_points(x, scale), to_whole_points(y, scale))
    }

    /// The top left corner of this monitor, in points.
    #[must_use]
    pub fn origin_in_points(self) -> (f64, f64) {
        let scale = self.usable_scale();
        (f64::from(self.x) / scale, f64::from(self.y) / scale)
    }

    /// The size of a window covering this monitor, in the points its pointer events use.
    #[must_use]
    pub fn size_in_points(self) -> LogicalSize {
        let scale = self.usable_scale();
        LogicalSize {
            width: f64::from(self.width) / scale,
            height: f64::from(self.height) / scale,
        }
    }

    /// The scale to divide by, with a nonsensical one treated as no scaling at all.
    ///
    /// Zero, a negative number or a NaN would turn every conversion here into an infinity or a
    /// NaN, and a monitor whose points are its pixels is a far better guess at what was meant
    /// than a coordinate no platform can answer about.
    fn usable_scale(self) -> f64 {
        if self.scale.is_finite() && self.scale > 0.0 {
            self.scale
        } else {
            1.0
        }
    }
}

/// A coordinate in physical pixels, as the whole number of points nearest the same place.
fn to_whole_points(pixels: i32, scale: f64) -> i32 {
    let points = f64::from(pixels) / scale;
    // Saturating, so a monitor at an absurd coordinate cannot wrap round to the opposite edge of
    // the desktop.
    points.round() as i32
}

/// A rectangle of pixels within a captured image, ready to be cut out of it.
///
/// Always inside the image it was worked out for, and never empty.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PixelRect {
    /// Distance from the left edge of the image, in pixels.
    pub x: u32,
    /// Distance from the top edge of the image, in pixels.
    pub y: u32,
    /// Width in pixels, at least 1.
    pub width: u32,
    /// Height in pixels, at least 1.
    pub height: u32,
}

/// Which pixels of `image` the user selected, or `None` if they selected nothing usable.
///
/// `surface` is the size, in points, of what they drew on; `image` is the size, in pixels, of what
/// was captured from the same screen. The ratio between the two is the scale, and it is worked out
/// separately for each axis because nothing guarantees they match.
///
/// The result is always inside the image and at least one pixel in each direction. `None` means
/// there is nothing to capture:
///
/// - a click rather than a drag, or a drag so short it rounds away to nothing;
/// - a selection entirely outside the surface;
/// - a surface or an image with no area, which is a caller's bug rather than a user's.
///
/// A selection that runs off the edge is not refused. It is cut down to what is there, because
/// dragging past the edge of the screen to take everything up to it is how a person selects the
/// edge of the screen.
#[must_use]
pub fn pixels_for(
    selection: Selection,
    surface: LogicalSize,
    image: PixelSize,
) -> Option<PixelRect> {
    if !(surface.width > 0.0 && surface.height > 0.0) {
        return None;
    }
    if image.width == 0 || image.height == 0 {
        return None;
    }
    if !(selection.x.is_finite()
        && selection.y.is_finite()
        && selection.width.is_finite()
        && selection.height.is_finite())
    {
        return None;
    }

    // Normalise: a drag up and to the left describes the same rectangle as one down and to the
    // right.
    let left = selection.x.min(selection.x + selection.width);
    let top = selection.y.min(selection.y + selection.height);
    let right = selection.x.max(selection.x + selection.width);
    let bottom = selection.y.max(selection.y + selection.height);

    // Clip to the surface before scaling, so that a drag off the edge keeps what was on screen.
    let left = left.max(0.0);
    let top = top.max(0.0);
    let right = right.min(surface.width);
    let bottom = bottom.min(surface.height);
    if right <= left || bottom <= top {
        return None;
    }

    let scale_x = f64::from(image.width) / surface.width;
    let scale_y = f64::from(image.height) / surface.height;

    // Round outwards. Half a pixel of screen the user included is better kept than dropped, and
    // rounding both edges the same way would make a selection one pixel narrower than it looked.
    let px_left = (left * scale_x).floor();
    let px_top = (top * scale_y).floor();
    let px_right = (right * scale_x).ceil();
    let px_bottom = (bottom * scale_y).ceil();

    let px_left = clamp_to_u32(px_left, image.width);
    let px_top = clamp_to_u32(px_top, image.height);
    let px_right = clamp_to_u32(px_right, image.width);
    let px_bottom = clamp_to_u32(px_bottom, image.height);

    let width = px_right.checked_sub(px_left)?;
    let height = px_bottom.checked_sub(px_top)?;
    if width == 0 || height == 0 {
        return None;
    }

    Some(PixelRect {
        x: px_left,
        y: px_top,
        width,
        height,
    })
}

/// Brings a scaled coordinate into the image, as a whole number of pixels.
///
/// The conversion is deliberate rather than an `as` cast: `as` on a value larger than `u32::MAX`
/// saturates silently, and a coordinate that large means the arithmetic above went wrong.
fn clamp_to_u32(value: f64, limit: u32) -> u32 {
    if value <= 0.0 {
        return 0;
    }
    if value >= f64::from(limit) {
        return limit;
    }
    // Between 0 and limit, so it fits.
    value as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    fn surface(width: f64, height: f64) -> LogicalSize {
        LogicalSize { width, height }
    }

    fn monitor(x: i32, y: i32, width: u32, height: u32, scale: f64) -> MonitorRect {
        MonitorRect {
            x,
            y,
            width,
            height,
            scale,
        }
    }

    #[test]
    fn centre_of_an_unscaled_monitor_is_the_same_in_both_spaces() {
        let screen = monitor(0, 0, 1920, 1080, 1.0);
        assert_eq!(screen.centre_in_pixels(), (960, 540));
        assert_eq!(screen.centre_in_points(), (960, 540));
    }

    #[test]
    fn centre_in_points_is_inside_a_retina_monitor() {
        // A MacBook Pro's built-in display: 1440x900 points, drawn at 2880x1800 pixels. The
        // centre in pixels is (1440, 900), which in points is off the bottom right corner
        // entirely - the coordinate that made macOS answer that no monitor was there (#40).
        let screen = monitor(0, 0, 2880, 1800, 2.0);
        assert_eq!(screen.centre_in_pixels(), (1440, 900));

        let (x, y) = screen.centre_in_points();
        assert_eq!((x, y), (720, 450));

        let size = screen.size_in_points();
        assert!(f64::from(x) < size.width, "{x} is not inside {size:?}");
        assert!(f64::from(y) < size.height, "{y} is not inside {size:?}");
    }

    #[test]
    fn a_second_monitor_keeps_its_own_scale() {
        // An unscaled monitor to the right of the Retina one above. Its pixel origin is where the
        // first monitor's pixels end; its point origin is where the first monitor's points end.
        let screen = monitor(2880, 0, 1920, 1080, 1.0);
        assert_eq!(screen.centre_in_pixels(), (3840, 540));
        assert_eq!(screen.centre_in_points(), (3840, 540));
        assert_eq!(screen.origin_in_points(), (2880.0, 0.0));
    }

    #[test]
    fn a_monitor_left_of_the_main_one_has_negative_coordinates() {
        let screen = monitor(-2880, -200, 2880, 1800, 2.0);
        assert_eq!(screen.centre_in_pixels(), (-1440, 700));
        assert_eq!(screen.centre_in_points(), (-720, 350));
        assert_eq!(screen.origin_in_points(), (-1440.0, -100.0));
    }

    #[test]
    fn a_scale_of_one_and_a_half_rounds_to_a_whole_point() {
        // 2560x1440 pixels at 150% is 1706.67x960 points. The centre lands between two points and
        // has to pick one; either neighbour is inside the monitor, which is all that is asked.
        let screen = monitor(0, 0, 2560, 1440, 1.5);
        let (x, y) = screen.centre_in_points();
        assert_eq!((x, y), (853, 480));

        let size = screen.size_in_points();
        assert!(f64::from(x) < size.width, "{x} is not inside {size:?}");
        assert!(f64::from(y) < size.height, "{y} is not inside {size:?}");
    }

    #[test]
    fn a_nonsensical_scale_is_treated_as_no_scaling() {
        for scale in [0.0, -2.0, f64::NAN, f64::INFINITY] {
            let screen = monitor(0, 0, 1920, 1080, scale);
            assert_eq!(
                screen.centre_in_points(),
                (960, 540),
                "a scale of {scale} should fall back to 1"
            );
            assert_eq!(screen.origin_in_points(), (0.0, 0.0));
            assert_eq!(screen.size_in_points(), surface(1920.0, 1080.0));
        }
    }

    fn image(width: u32, height: u32) -> PixelSize {
        PixelSize { width, height }
    }

    fn selection(x: f64, y: f64, width: f64, height: f64) -> Selection {
        Selection {
            x,
            y,
            width,
            height,
        }
    }

    #[test]
    fn at_one_to_one_the_selection_is_the_rectangle() {
        let rect = pixels_for(
            selection(10.0, 20.0, 100.0, 50.0),
            surface(800.0, 600.0),
            image(800, 600),
        );
        assert_eq!(
            rect,
            Some(PixelRect {
                x: 10,
                y: 20,
                width: 100,
                height: 50
            })
        );
    }

    // The case the platforms disagree about. Nothing here asks either of them.
    #[test]
    fn a_display_at_150_percent_doubles_nothing_and_scales_everything() {
        let rect = pixels_for(
            selection(100.0, 100.0, 200.0, 100.0),
            surface(1280.0, 720.0),
            image(1920, 1080),
        );
        assert_eq!(
            rect,
            Some(PixelRect {
                x: 150,
                y: 150,
                width: 300,
                height: 150
            })
        );
    }

    #[test]
    fn a_display_at_200_percent_scales_the_same_way() {
        let rect = pixels_for(
            selection(0.0, 0.0, 50.0, 25.0),
            surface(1000.0, 500.0),
            image(2000, 1000),
        );
        assert_eq!(
            rect,
            Some(PixelRect {
                x: 0,
                y: 0,
                width: 100,
                height: 50
            })
        );
    }

    // A drag up and to the left is as ordinary as one down and to the right.
    #[test]
    fn dragging_backwards_selects_the_same_rectangle() {
        let forwards = pixels_for(
            selection(10.0, 20.0, 100.0, 50.0),
            surface(800.0, 600.0),
            image(800, 600),
        );
        let backwards = pixels_for(
            selection(110.0, 70.0, -100.0, -50.0),
            surface(800.0, 600.0),
            image(800, 600),
        );
        assert_eq!(forwards, backwards);
        assert!(forwards.is_some());
    }

    #[test]
    fn a_drag_past_the_edge_keeps_what_was_on_screen() {
        let rect = pixels_for(
            selection(700.0, 500.0, 500.0, 500.0),
            surface(800.0, 600.0),
            image(800, 600),
        );
        assert_eq!(
            rect,
            Some(PixelRect {
                x: 700,
                y: 500,
                width: 100,
                height: 100
            })
        );
    }

    #[test]
    fn a_drag_starting_off_screen_keeps_what_was_on_screen() {
        let rect = pixels_for(
            selection(-50.0, -50.0, 100.0, 100.0),
            surface(800.0, 600.0),
            image(800, 600),
        );
        assert_eq!(
            rect,
            Some(PixelRect {
                x: 0,
                y: 0,
                width: 50,
                height: 50
            })
        );
    }

    #[test]
    fn a_click_selects_nothing() {
        assert_eq!(
            pixels_for(
                selection(100.0, 100.0, 0.0, 0.0),
                surface(800.0, 600.0),
                image(800, 600)
            ),
            None
        );
    }

    #[test]
    fn a_selection_entirely_off_screen_is_nothing() {
        assert_eq!(
            pixels_for(
                selection(900.0, 700.0, 100.0, 100.0),
                surface(800.0, 600.0),
                image(800, 600)
            ),
            None
        );
    }

    // A drag of a fraction of a point still touches one pixel, and that is what comes back.
    //
    // Refusing it here would be the wrong place for that rule: how far the pointer has to move
    // before a drag counts as a drag is a question about the pointer, and the overlay answers it
    // by treating anything shorter than a few points as a click. What is left for this function
    // is the geometry, and geometrically one pixel was selected.
    #[test]
    fn a_drag_of_a_fraction_of_a_point_still_selects_the_pixel_it_touched() {
        let rect = pixels_for(
            selection(10.4, 10.4, 0.0001, 0.0001),
            surface(800.0, 600.0),
            image(800, 600),
        );
        assert_eq!(
            rect,
            Some(PixelRect {
                x: 10,
                y: 10,
                width: 1,
                height: 1
            })
        );
    }

    // Rounding outwards: a selection from 10.5 to 15.7 touches pixels 10 through 15, so it keeps
    // six of them rather than the five a symmetrical rounding would leave.
    #[test]
    fn a_selection_between_pixels_keeps_the_pixels_it_touched() {
        let rect = pixels_for(
            selection(10.5, 10.5, 5.2, 5.2),
            surface(800.0, 600.0),
            image(800, 600),
        );
        assert_eq!(
            rect,
            Some(PixelRect {
                x: 10,
                y: 10,
                width: 6,
                height: 6
            })
        );
    }

    #[test]
    fn the_whole_screen_is_the_whole_image() {
        let rect = pixels_for(
            selection(0.0, 0.0, 1280.0, 720.0),
            surface(1280.0, 720.0),
            image(1920, 1080),
        );
        assert_eq!(
            rect,
            Some(PixelRect {
                x: 0,
                y: 0,
                width: 1920,
                height: 1080
            })
        );
    }

    #[test]
    fn a_surface_or_image_with_no_area_is_refused() {
        let ok = surface(800.0, 600.0);
        let sel = selection(10.0, 10.0, 10.0, 10.0);

        assert_eq!(pixels_for(sel, surface(0.0, 600.0), image(800, 600)), None);
        assert_eq!(pixels_for(sel, surface(800.0, 0.0), image(800, 600)), None);
        assert_eq!(pixels_for(sel, ok, image(0, 600)), None);
        assert_eq!(pixels_for(sel, ok, image(800, 0)), None);
    }

    // A window system that hands over a NaN is a window system having a bad day; it must not
    // become a crop of unpredictable size.
    #[test]
    fn a_selection_that_is_not_a_number_is_refused() {
        let ok = surface(800.0, 600.0);
        let img = image(800, 600);

        assert_eq!(
            pixels_for(selection(f64::NAN, 0.0, 10.0, 10.0), ok, img),
            None
        );
        assert_eq!(
            pixels_for(selection(0.0, 0.0, f64::INFINITY, 10.0), ok, img),
            None
        );
        assert_eq!(
            pixels_for(selection(0.0, f64::NEG_INFINITY, 10.0, 10.0), ok, img),
            None
        );
    }

    // The result is used to cut a rectangle out of an image, so it must never name a pixel the
    // image does not have.
    #[test]
    fn the_result_always_fits_inside_the_image() {
        let img = image(1920, 1080);
        let ok = surface(1280.0, 720.0);

        for sel in [
            selection(-1000.0, -1000.0, 5000.0, 5000.0),
            selection(1279.9, 719.9, 100.0, 100.0),
            selection(0.0, 0.0, 1280.0, 720.0),
            selection(640.0, 360.0, -2000.0, -2000.0),
        ] {
            let rect = pixels_for(sel, ok, img).expect("each of these selects something");
            assert!(
                rect.x + rect.width <= img.width,
                "{rect:?} runs off the right"
            );
            assert!(
                rect.y + rect.height <= img.height,
                "{rect:?} runs off the bottom"
            );
            assert!(rect.width > 0 && rect.height > 0, "{rect:?} is empty");
        }
    }
}
