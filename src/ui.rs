use crate::meter::{Direction, Meter};
use crate::{DYNAMIC_RANGE, GAIN_ATOMIC, IN_L_ATOMIC, IN_R_ATOMIC, OUT_L_ATOMIC, OUT_R_ATOMIC};
use std::collections::vec_deque::VecDeque;
use std::sync::atomic::Ordering;
use std::sync::atomic::Ordering::Relaxed;
use vizia::{Application, Color, Context, Event, HStack, Knob, Label, Lens, Model, Percentage, Pixels, Stretch, Textbox, Units, VStack, WindowDescription};

const STYLE: &str = include_str!("style.css");

#[derive(Lens)]
pub struct UIData {
    gain: f32,
    buffer_size: i32,
    in_l: f32,
    in_l_buffer: VecDeque<f32>,
    in_r: f32,
    in_r_buffer: VecDeque<f32>,
    out_l: f32,
    out_l_buffer: VecDeque<f32>,
    out_r: f32,
    out_r_buffer: VecDeque<f32>,
}

pub enum GainEvents {
    GainChange(f32),
}

pub enum MeterEvents {
    InLUpdate(f32),
    InRUpdate(f32),
    OutLUpdate(f32),
    OutRUpdate(f32),
}

impl Model for UIData {
    fn event(&mut self, cx: &mut Context, event: &mut Event) {
        if let Some(gain_event) = event.message.downcast() {
            match gain_event {
                GainEvents::GainChange(n) => {
                    self.gain = *n;
                    GAIN_ATOMIC.store(*n, Ordering::Relaxed);
                }
            }
        }

        if let Some(meter_event) = event.message.downcast() {
            // Add the value to the buffer and return the corresponding average value
            let mut changed_value = match meter_event {
                MeterEvents::InLUpdate(n) => {
                    self.in_l_buffer.push_front((*n).abs());
                    if self.in_l_buffer.len() > self.buffer_size as usize {
                        self.in_l_buffer.pop_back();
                    }
                    self.in_l_buffer.iter().sum::<f32>() / self.buffer_size as f32
                },
                MeterEvents::InRUpdate(n) => {
                    self.in_r_buffer.push_front((*n).abs());
                    if self.in_r_buffer.len() > self.buffer_size as usize {
                        self.in_r_buffer.pop_back();
                    }
                    self.in_r_buffer.iter().sum::<f32>() / self.buffer_size as f32
                },
                MeterEvents::OutLUpdate(n) => {
                    self.out_l_buffer.push_front((*n).abs());
                    if self.out_l_buffer.len() > self.buffer_size as usize {
                        self.out_l_buffer.pop_back();
                    }
                    self.out_l_buffer.iter().sum::<f32>() / self.buffer_size as f32
                },
                MeterEvents::OutRUpdate(n) => {
                    self.out_r_buffer.push_front((*n).abs());
                    if self.out_r_buffer.len() > self.buffer_size as usize {
                        self.out_r_buffer.pop_back();
                    }
                    self.out_r_buffer.iter().sum::<f32>() / self.buffer_size as f32
                }
            };

            // Convert the linear scale to a db scale
            changed_value = lin2db(changed_value);
            if changed_value < DYNAMIC_RANGE {
                changed_value = 0.0;
            } else {
                changed_value = 1.0 - (changed_value / DYNAMIC_RANGE);
            }

            // Update the changed value
            match meter_event {
                MeterEvents::InLUpdate(n) => {
                    self.in_l = changed_value;
                }
                MeterEvents::InRUpdate(n) => {
                    self.in_r = changed_value;
                }
                MeterEvents::OutLUpdate(n) => {
                    self.out_l = changed_value;
                }
                MeterEvents::OutRUpdate(n) => {
                    self.out_r = changed_value;
                }
            }
        }
    }
}

pub fn ui() {
    let mut window_description = WindowDescription::new()
        .with_inner_size(300, 300)
        .with_title("jack_gain");
    window_description.resizable = false;

    Application::new(window_description, move |cx| {
        UIData {
            gain: 1.0,
            buffer_size: 8,
            in_l: 0.0,
            in_l_buffer: VecDeque::new(),
            in_r: 0.0,
            in_r_buffer: VecDeque::new(),
            out_l: 0.0,
            out_l_buffer: VecDeque::new(),
            out_r: 0.0,
            out_r_buffer: VecDeque::new(),
        }
        .build(cx);

        cx.add_theme(STYLE);

        HStack::new(cx, |cx| {
            Meter::new(cx, UIData::in_l, Direction::DownToUp);
            Meter::new(cx, UIData::in_r, Direction::DownToUp);
            VStack::new(cx, |cx| {
                Knob::new(cx, 1.0, UIData::gain, false)
                    .on_changing(move |cx, val| cx.emit(GainEvents::GainChange(val)));
                Label::new(cx, UIData::gain);
            })
            .child_space(Stretch(1.0));
            Meter::new(cx, UIData::out_l, Direction::DownToUp);
            Meter::new(cx, UIData::out_r, Direction::DownToUp);
        })
        .class("main");
    })
    .on_idle(|cx| {
        cx.emit(MeterEvents::InLUpdate(IN_L_ATOMIC.load(Relaxed)));
        cx.emit(MeterEvents::InRUpdate(IN_R_ATOMIC.load(Relaxed)));
        cx.emit(MeterEvents::OutLUpdate(OUT_L_ATOMIC.load(Relaxed)));
        cx.emit(MeterEvents::OutRUpdate(OUT_R_ATOMIC.load(Relaxed)));
    })
    .run();
}

#[inline]
/// Converts a linear value into a db one
///
///
///
/// Source: https://mu.krj.st/mix/
fn lin2db(input: f32) -> f32 {
    20.0 * input.log10()
}
