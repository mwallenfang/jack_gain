use femtovg::Color as femColor;
use femtovg::{LineCap, Paint, Path, Solidity};
use itertools::cloned;
use std::collections::VecDeque;
use vizia::Color as vizColor;
use vizia::*;

/// The direction the meter bar shows the peak in.
/// The semantic is LowToHigh, so DownToUp is the standard vertical meter design
///
/// This is also used to decide the orientation of the meter
#[derive(Debug, Clone, Copy, PartialEq, Eq, Data)]
pub enum Direction {
    /// The standard vertical meter direction
    DownToUp,
    /// The inverted direction from the standard vertical meter
    UpToDown,
    /// The standard horizontal meter direction
    LeftToRight,
    /// The inverted direction from the standard horizontal meter
    RightToLeft,
}

pub struct Meter<L> {
    /// The position of the meter in [0,1]
    lens: L,
    /// The directions the meter's ends are pointing in
    direction: Direction,
}

impl<L: Lens<Target = f32>> Meter<L> {
    pub fn new(cx: &mut Context, lens: L, direction: Direction) -> Handle<Self> {
        Self {
            lens: lens.clone(),
            direction,
        }
        .build2(cx, move |cx| {
            ZStack::new(cx, move |cx| {
                MeterBar::new(cx, direction).value(lens);
            });
        })
    }
}

impl<L: Lens<Target = f32>> View for Meter<L> {
    fn element(&self) -> Option<String> {
        Some("meter".to_string())
    }
}

pub struct MeterBar {
    /// The value to show on the meter in [0,1]
    value: f32,
    /// The directions the meter's ends are pointing in
    direction: Direction,
}

impl MeterBar {
    pub fn new(cx: &mut Context, direction: Direction) -> Handle<Self> {
        Self {
            value: 0.0,
            direction,
        }
        .build(cx)
    }
}

impl View for MeterBar {
    fn element(&self) -> Option<String> {
        Some("meter_bar".to_string())
    }

    fn draw(&self, cx: &mut Context, canvas: &mut Canvas) {
        let width = cx.cache.get_width(cx.current);
        let height = cx.cache.get_height(cx.current);
        let pos_x = cx.cache.get_posx(cx.current);
        let pos_y = cx.cache.get_posy(cx.current);
        let value = self.value;

        let bar_color = cx
            .style
            .background_color
            .get(cx.current)
            .cloned()
            .unwrap_or_default()
            .into();

        // Create variables for the rectangle
        let front_x;
        let front_y;
        let front_w;
        let front_h;

        // Build the start and end positions of the back and front line
        // according to the direction the meter is going and the value the meter is showing
        match self.direction {
            Direction::DownToUp => {
                front_x = pos_x;
                front_y = pos_y + (1.0 - value) * height;

                front_w = width;
                front_h = value * height;
            }
            Direction::UpToDown => {
                front_x = pos_x;
                front_y = pos_y;

                front_w = width;
                front_h = value * height;
            }
            Direction::LeftToRight => {
                front_x = pos_x;
                front_y = pos_y;

                front_w = value * width;
                front_h = height;
            }
            Direction::RightToLeft => {
                front_x = pos_x + (1.0 - value) * width;
                front_y = pos_y;

                front_w = value * width;
                front_h = height;
            }
        };

        // Draw the bar
        if value >= 1e-3 {
            let mut front_path = Path::new();
            front_path.rect(front_x, front_y, front_w, front_h);

            let mut front_paint = Paint::color(bar_color);

            canvas.fill_path(&mut front_path, front_paint);
        }
    }
}

pub trait MeterBarHandle {
    fn value<L: Lens<Target = f32>>(self, lens: L) -> Self
    where
        L: Lens<Target = f32>;
}

impl MeterBarHandle for Handle<'_, MeterBar> {
    fn value<L: Lens<Target = f32>>(self, lens: L) -> Self {
        let entity = self.entity;
        Binding::new(self.cx, lens, move |cx, value| {
            let value = *value.get(cx);

            if let Some(view) = cx.views.get_mut(&entity) {
                if let Some(bar) = view.downcast_mut::<MeterBar>() {
                    bar.value = value;
                    cx.style.needs_redraw = true;
                }
            }
        });

        self
    }
}
