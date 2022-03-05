use std::sync::atomic::Ordering;
use vizia::{Application, Context, Event, HStack, Knob, Model, WindowDescription, Lens, Label, VStack};
use crate::GAIN_ATOMIC;

const STYLE: &str = include_str!("style.css");

#[derive(Lens)]
pub struct UIData {
    gain: f32
}

pub enum UIEvents {
    GainChange(f32)
}

impl Model for UIData {
    fn event(&mut self, cx: &mut Context, event: &mut Event) {
        if let Some(gain_event) = event.message.downcast() {
            match gain_event {
                UIEvents::GainChange(n) => {
                    self.gain = *n;
                    GAIN_ATOMIC.store(*n, Ordering::Relaxed);
                }
            }
        }
    }
}

pub fn ui() {
    let window_description = WindowDescription::new()
        .with_inner_size(300,300)
        .with_title("jack_gain");

    Application::new(window_description, move |cx| {
        UIData{gain: 1.0}.build(cx);

        cx.add_theme(STYLE);

        HStack::new(cx, |cx| {
            VStack::new(cx, |cx| {
                Knob::new(cx, 1.0, UIData::gain, false)
                    .on_changing(move |cx, val| cx.emit(UIEvents::GainChange(val)));
                Label::new(cx, UIData::gain);
            });
        });
    }).run();
}