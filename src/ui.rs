use std::collections::VecDeque;
use std::sync::atomic::Ordering;
use std::sync::atomic::Ordering::Relaxed;
use vizia::{Application, Context, Event, HStack, Knob, Model, WindowDescription, Lens, Label, VStack, Pixels, Color, Units, Percentage, Stretch};
use crate::{GAIN_ATOMIC, IN_L_ATOMIC, IN_R_ATOMIC, OUT_L_ATOMIC, OUT_R_ATOMIC};
use crate::meter::{Meter, Direction};

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
    out_r_buffer: VecDeque<f32>
}

pub enum UIEvents {
    GainChange(f32),
    InLUpdate(f32),
    InRUpdate(f32),
    OutLUpdate(f32),
    OutRUpdate(f32),
}

impl Model for UIData {
    fn event(&mut self, cx: &mut Context, event: &mut Event) {
        if let Some(gain_event) = event.message.downcast() {
            match gain_event {
                UIEvents::GainChange(n) => {
                    self.gain = *n;
                    GAIN_ATOMIC.store(*n, Ordering::Relaxed);
                },
                UIEvents::InLUpdate(n) => {
                    self.in_l_buffer.push_front((*n).abs());
                    if self.in_l_buffer.len() > self.buffer_size as usize {
                        self.in_l_buffer.pop_back();
                    }
                    let new_pos = self.in_l_buffer.iter().sum::<f32>() / self.buffer_size as f32;
                    self.in_l = new_pos;
                },
                UIEvents::InRUpdate(n) => {

                    self.in_r_buffer.push_front((*n).abs());
                    if self.in_r_buffer.len() > self.buffer_size as usize {
                        self.in_r_buffer.pop_back();
                    }
                    let new_pos = self.in_r_buffer.iter().sum::<f32>() / self.buffer_size as f32;
                    self.in_r = new_pos;
                },
                UIEvents::OutLUpdate(n) => {

                    self.out_l_buffer.push_front((*n).abs());
                    if self.out_l_buffer.len() > self.buffer_size as usize {
                        self.out_l_buffer.pop_back();
                    }
                    let new_pos = self.out_l_buffer.iter().sum::<f32>() / self.buffer_size as f32;
                    self.out_l = new_pos;
                },
                UIEvents::OutRUpdate(n) => {

                    self.out_r_buffer.push_front((*n).abs());
                    if self.out_r_buffer.len() > self.buffer_size as usize {
                        self.out_r_buffer.pop_back();
                    }
                    let new_pos = self.out_r_buffer.iter().sum::<f32>() / self.buffer_size as f32;
                    self.out_r = new_pos;
                }
            }
        }
    }
}

pub fn ui() {
    let mut window_description = WindowDescription::new()
        .with_inner_size(300,300)
        .with_title("jack_gain");
    window_description.resizable = false;

    Application::new(window_description, move |cx| {
        UIData{
            gain: 1.0,
            buffer_size: 4,
            in_l: 0.0,
            in_l_buffer: VecDeque::new(),
            in_r: 0.0,
            in_r_buffer: VecDeque::new(),
            out_l: 0.0,
            out_l_buffer: VecDeque::new(),
            out_r: 0.0,
            out_r_buffer: VecDeque::new()
        }.build(cx);

        cx.add_theme(STYLE);

        HStack::new(cx, |cx| {
            Meter::new(cx, UIData::in_l, Direction::DownToUp);
            Meter::new(cx, UIData::in_r, Direction::DownToUp);
            VStack::new(cx, |cx| {
                Knob::new(cx, 1.0, UIData::gain, false)
                    .on_changing(move |cx, val| cx.emit(UIEvents::GainChange(val)));
                Label::new(cx, UIData::gain);
            })
                .child_space(Stretch(1.0));
            Meter::new(cx, UIData::out_l, Direction::DownToUp);
            Meter::new(cx, UIData::out_r, Direction::DownToUp);
        })
            .class("main");
    })
        .on_idle(|cx| {
            cx.emit(UIEvents::InLUpdate(IN_L_ATOMIC.load(Relaxed)));
            cx.emit(UIEvents::InRUpdate(IN_R_ATOMIC.load(Relaxed)));
            cx.emit(UIEvents::OutLUpdate(OUT_L_ATOMIC.load(Relaxed)));
            cx.emit(UIEvents::OutRUpdate(OUT_R_ATOMIC.load(Relaxed)));
        })
        .run();
}