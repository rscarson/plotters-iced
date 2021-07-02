// plotters-iced
//
// Iced backend for Plotters
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

use super::{svg::SvgBackend, AsBumpStr};
use crate::Chart;
use dodrio::bumpalo;
use iced_web::{css, Bus, Css, Element, Font, Length, Widget};
use plotters_backend::{FontFamily, FontStyle};
use std::marker::PhantomData;

macro_rules! console_log {
    ($($t:tt)*) => (unsafe {super::log(&format_args!($($t)*).to_string()) })
}

/// Chart container, turns [`Chart`]s to [`Widget`]s
pub struct ChartWidget<'a, Message, C>
where
    C: Chart<Message>,
{
    chart: &'a mut C,
    width: u16,
    height: u16,
    _marker: PhantomData<Message>,
}

impl<'a, Message, C> ChartWidget<'a, Message, C>
where
    C: Chart<Message>,
{
    #[inline]
    pub fn new(chart: &'a mut C) -> Self {
        Self {
            chart,
            width: 100,
            height: 100,
            _marker: Default::default(),
        }
    }

    /// only support fixed size for wasm
    #[inline]
    pub fn width(mut self, width: Length) -> Self {
        match width {
            Length::Units(width) => {
                self.width = width;
            }
            _ => {
                console_log!("dynamic width not supported");
            }
        }
        self
    }

    /// only support fixed size for wasm
    #[inline]
    pub fn height(mut self, height: Length) -> Self {
        match height {
            Length::Units(height) => {
                self.height = height;
            }
            _ => {
                console_log!("dynamic height not supported");
            }
        }
        self
    }

    /// stub for API compatible
    #[inline]
    pub fn resolve_font(self, _resolver: impl Fn(FontFamily, FontStyle) -> Font + 'static) -> Self {
        self
    }
}

impl<'a, Message, C> Widget<Message> for ChartWidget<'a, Message, C>
where
    C: Chart<Message>,
    Message: 'static,
{
    fn node<'b>(
        &self,
        bump: &'b bumpalo::Bump,
        _bus: &Bus<Message>,
        _style_sheet: &mut Css<'b>,
    ) -> dodrio::Node<'b> {
        use dodrio::builder::*;
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut nodes = Vec::new();
        let backend = SvgBackend::new(bump, (self.width as u32, self.height as u32), &mut nodes);
        self.chart.draw_chart(backend.into());

        // use ptr as id
        let ptr = self.chart as *const C;
        let mut h = DefaultHasher::new();
        ptr.hash(&mut h);
        let id = bumpalo::format!(
            in bump,
            "chart-{}", h.finish()
        )
        .into_bump_str();
        let node = svg(bump)
            .attr("id", id)
            .attr(
                "style",
                bumpalo::format!(
                    in bump,
                    "width:{}; height:{}",
                    css::length(Length::Units(self.width)),
                    css::length(Length::Units(self.height))
                )
                .into_bump_str(),
            )
            .attr("class", "plotters-iced-chart".as_bump_str(bump))
            .children(nodes);

        node.finish()
    }
}

impl<'a, Message, C> From<ChartWidget<'a, Message, C>> for Element<'a, Message>
where
    Message: 'static,
    C: Chart<Message>,
{
    #[inline]
    fn from(widget: ChartWidget<'a, Message, C>) -> Self {
        Element::new(widget)
    }
}