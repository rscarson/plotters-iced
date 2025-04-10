// plotters-iced
//
// Iced backend for Plotters
// Copyright: 2022, Joylei <leingliu@gmail.com>
// License: MIT

use iced_graphics::core::text::Paragraph;
use iced_graphics::core::{Degrees, Point, Vector};
use iced_widget::core::Rectangle;
use iced_widget::{
    canvas,
    core::{
        alignment::{Horizontal, Vertical},
        font, text, Font, Size,
    },
    image,
    text::Shaping,
};
use plotters_backend::{
    text_anchor,
    //FontTransform,
    BackendColor,
    BackendCoord,
    BackendStyle,
    BackendTextStyle,
    DrawingBackend,
    DrawingErrorKind,
    FontFamily,
    FontStyle,
    FontTransform,
};

use dashmap::DashSet;
use std::sync::LazyLock;

use crate::error::Error;
use crate::utils::{cvt_color, cvt_stroke, CvtPoint};

/// The Iced drawing backend
pub(crate) struct IcedChartBackend<'a, B> {
    frame: &'a mut canvas::Frame,
    backend: &'a B,
    shaping: Shaping,
}

impl<'a, B> IcedChartBackend<'a, B>
where
    B: text::Renderer<Font = Font>,
{
    pub fn new(frame: &'a mut canvas::Frame, backend: &'a B, shaping: Shaping) -> Self {
        Self {
            frame,
            backend,
            shaping,
        }
    }
}

impl<B> DrawingBackend for IcedChartBackend<'_, B>
where
    B: text::Renderer<Font = Font>,
{
    type ErrorType = Error;

    fn get_size(&self) -> (u32, u32) {
        let Size { width, height } = self.frame.size();
        (width as u32, height as u32)
    }

    fn ensure_prepared(&mut self) -> Result<(), DrawingErrorKind<Error>> {
        Ok(())
    }

    fn present(&mut self) -> Result<(), DrawingErrorKind<Error>> {
        Ok(())
    }

    #[inline]
    fn draw_pixel(
        &mut self,
        point: BackendCoord,
        color: BackendColor,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        if color.alpha == 0.0 {
            return Ok(());
        }
        self.frame
            .fill_rectangle(point.cvt_point(), Size::new(1.0, 1.0), cvt_color(&color));
        Ok(())
    }

    #[inline]
    fn draw_line<S: BackendStyle>(
        &mut self,
        from: BackendCoord,
        to: BackendCoord,
        style: &S,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        if style.color().alpha == 0.0 {
            return Ok(());
        }
        let line = canvas::Path::line(from.cvt_point(), to.cvt_point());
        self.frame.stroke(&line, cvt_stroke(style));
        Ok(())
    }

    #[inline]
    fn draw_rect<S: BackendStyle>(
        &mut self,
        upper_left: BackendCoord,
        bottom_right: BackendCoord,
        style: &S,
        fill: bool,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        if style.color().alpha == 0.0 {
            return Ok(());
        }
        let height = (bottom_right.1 - upper_left.1) as f32;
        let width = (bottom_right.0 - upper_left.0) as f32;
        let upper_left = upper_left.cvt_point();
        if fill {
            self.frame.fill_rectangle(
                upper_left,
                Size::new(width, height),
                cvt_color(&style.color()),
            );
        } else {
            let rect = canvas::Path::rectangle(upper_left, Size::new(width, height));
            self.frame.stroke(&rect, cvt_stroke(style));
        }

        Ok(())
    }

    #[inline]
    fn draw_path<S: BackendStyle, I: IntoIterator<Item = BackendCoord>>(
        &mut self,
        path: I,
        style: &S,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        if style.color().alpha == 0.0 {
            return Ok(());
        }
        let path = canvas::Path::new(move |builder| {
            for (i, point) in path.into_iter().enumerate() {
                if i > 0 {
                    builder.line_to(point.cvt_point());
                } else {
                    builder.move_to(point.cvt_point());
                }
            }
        });

        self.frame.stroke(&path, cvt_stroke(style));
        Ok(())
    }

    #[inline]
    fn draw_circle<S: BackendStyle>(
        &mut self,
        center: BackendCoord,
        radius: u32,
        style: &S,
        fill: bool,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        if style.color().alpha == 0.0 {
            return Ok(());
        }

        let circle = canvas::Path::circle(center.cvt_point(), radius as f32);

        if fill {
            self.frame.fill(&circle, cvt_color(&style.color()));
        } else {
            self.frame.stroke(&circle, cvt_stroke(style));
        }

        Ok(())
    }

    #[inline]
    fn fill_polygon<S: BackendStyle, I: IntoIterator<Item = BackendCoord>>(
        &mut self,
        vert: I,
        style: &S,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        if style.color().alpha == 0.0 {
            return Ok(());
        }
        let path = canvas::Path::new(move |builder| {
            for (i, point) in vert.into_iter().enumerate() {
                if i > 0 {
                    builder.line_to(point.cvt_point());
                } else {
                    builder.move_to(point.cvt_point());
                }
            }
            builder.close();
        });
        self.frame.fill(&path, cvt_color(&style.color()));
        Ok(())
    }

    #[inline]
    fn draw_text<S: BackendTextStyle>(
        &mut self,
        text: &str,
        style: &S,
        position: BackendCoord,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        if style.color().alpha == 0.0 {
            return Ok(());
        }
        let horizontal_alignment = match style.anchor().h_pos {
            text_anchor::HPos::Left => Horizontal::Left,
            text_anchor::HPos::Right => Horizontal::Right,
            text_anchor::HPos::Center => Horizontal::Center,
        };
        let vertical_alignment = match style.anchor().v_pos {
            text_anchor::VPos::Top => Vertical::Top,
            text_anchor::VPos::Center => Vertical::Center,
            text_anchor::VPos::Bottom => Vertical::Bottom,
        };
        let font = style_to_font(style);
        let pos = position.cvt_point();

        //TODO: fix rotation until text rotation is supported by Iced
        let rotate = match style.transform() {
            FontTransform::None => None,
            FontTransform::Rotate90 => Some(90.0),
            FontTransform::Rotate180 => Some(180.0),
            FontTransform::Rotate270 => Some(270.0),
        };

        if let Some(rotate) = rotate {
            self.frame.with_save(|frame| {
                frame.translate(Vector::new(pos.x, pos.y));
                frame.rotate(Degrees(rotate));

                let text_canvas = canvas::Text {
                    content: text.to_string(),
                    position: Point::new(0.0, 0.0),
                    color: cvt_color(&style.color()),
                    size: (style.size() as f32).into(),
                    line_height: Default::default(),
                    font,
                    horizontal_alignment,
                    vertical_alignment,
                    shaping: self.shaping,
                };

                frame.fill_text(text_canvas);
            });
        } else {
            let text_canvas = canvas::Text {
                content: text.to_string(),
                position: pos,
                color: cvt_color(&style.color()),
                size: (style.size() as f32).into(),
                line_height: Default::default(),
                font,
                horizontal_alignment,
                vertical_alignment,
                shaping: self.shaping,
            };

            self.frame.fill_text(text_canvas);
        }

        Ok(())
    }

    #[inline]
    fn estimate_text_size<S: BackendTextStyle>(
        &self,
        text: &str,
        style: &S,
    ) -> Result<(u32, u32), DrawingErrorKind<Self::ErrorType>> {
        let font = style_to_font(style);
        let bounds = self.frame.size();
        let horizontal_alignment = match style.anchor().h_pos {
            text_anchor::HPos::Left => Horizontal::Left,
            text_anchor::HPos::Right => Horizontal::Right,
            text_anchor::HPos::Center => Horizontal::Center,
        };
        let vertical_alignment = match style.anchor().v_pos {
            text_anchor::VPos::Top => Vertical::Top,
            text_anchor::VPos::Center => Vertical::Center,
            text_anchor::VPos::Bottom => Vertical::Bottom,
        };

        let p = B::Paragraph::with_text(iced_widget::core::text::Text {
            content: text,
            bounds,
            size: self.backend.default_size(),
            line_height: Default::default(),
            font,
            horizontal_alignment,
            vertical_alignment,
            shaping: self.shaping,
            wrapping: iced_widget::core::text::Wrapping::Word,
        });
        let size = p.min_bounds();
        Ok(((size.width * 1.1) as u32, size.height as u32))
    }

    #[inline]
    fn blit_bitmap(
        &mut self,
        (x, y): BackendCoord,
        (iw, ih): (u32, u32),
        src: &[u8],
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        let image = image::Handle::from_rgba(iw, ih, src.to_owned());

        let pos = (x - iw as i32 / 2, y - ih as i32 / 2) as BackendCoord;

        let bounds = Rectangle::new(pos.cvt_point(), Size::new(iw as f32, ih as f32));
        self.frame.draw_image(bounds, &image);
        Ok(())
    }
}

fn style_to_font<S: BackendTextStyle>(style: &S) -> Font {
    //
    // iced requires &'static str for font names, but plotters uses String
    // So we need a static registry to convert String to &'static str
    static FONT_REGISTRY: FontNameRegistry = FontNameRegistry::new();

    Font {
        family: match style.family() {
            FontFamily::Serif => font::Family::Serif,
            FontFamily::SansSerif => font::Family::SansSerif,
            FontFamily::Monospace => font::Family::Monospace,
            FontFamily::Name(s) => font::Family::Name(FONT_REGISTRY.register(s)),
        },
        weight: match style.style() {
            FontStyle::Bold => font::Weight::Bold,
            _ => font::Weight::Normal,
        },
        ..Font::DEFAULT
    }
}

/// Inner mutable registry of font names, for `iced` to use
struct FontNameRegistry {
    inner: LazyLock<DashSet<&'static str>>,
}
impl FontNameRegistry {
    /// Create a new empty registry
    pub const fn new() -> Self {
        Self {
            inner: LazyLock::new(DashSet::new),
        }
    }

    /// Convert a name to a static string  
    /// WARNING: Will leak the string in memory, use with caution
    pub fn register(&self, name: impl AsRef<str>) -> &'static str {
        if let Some(name) = self.inner.get(name.as_ref()) {
            *name
        } else {
            let name = name.as_ref().to_string();
            let name = Box::leak(name.into_boxed_str());
            self.inner.insert(name);
            name
        }
    }
}
