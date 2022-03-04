//! Gain plugin based on https://mu.krj.st/mix/

use itertools::izip;
use ringbuf::{Producer, RingBuffer};
use std::io;
use std::str::FromStr;
use vizia::{Application, Context, Event, HStack, Knob, Model, WindowDescription, Lens};

#[derive(Lens)]
pub struct UIData {
    gain: f32,
    #[lens(ignore)]
    producer: Producer<f32>
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
                    self.producer.push(*n);
                }
            }
        }
    }
}

fn ui(prod: Producer<f32>) {
    Application::new(WindowDescription::new(), move |cx| {
        UIData{gain: 1.0, producer: prod}.build(cx);

        HStack::new(cx, |cx| {
            Knob::new(cx, 1.0, UIData::gain, false)
                .on_changing(move |cx, val| cx.emit(UIEvents::GainChange(val)));
        });
    }).run();
}

fn main() {
    // 1. open a client
    let (client, _status) =
        jack::Client::new("jack_gain", jack::ClientOptions::NO_START_SERVER).unwrap();

    // 2. register port
    let mut out_port_l = client
        .register_port("gain_out_l", jack::AudioOut::default())
        .unwrap();

    let mut out_port_r = client
        .register_port("gain_out_r", jack::AudioOut::default())
        .unwrap();

    let in_port_l = client
        .register_port("gain_in_l", jack::AudioIn::default())
        .unwrap();

    let in_port_r = client
        .register_port("gain_in_r", jack::AudioIn::default())
        .unwrap();

    // 3. define process callback handler
    let rb = RingBuffer::<f32>::new(client.sample_rate());
    let (mut prod, mut cons) = rb.split();

    // Define the amount of steps to be the amount of samples in 50ms
    let step_amount: i32 = (client.sample_rate() as f32 * 0.05) as i32;

    let mut db_current: f32 = 0.0;
    let mut db_destination: f32 = 0.0;
    let mut db_step_counter = step_amount;
    let mut db_step_size = (db_destination - db_current) / step_amount as f32;

    let process = jack::ClosureProcessHandler::new(
        move |_: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
            // Get output buffer
            let out_p_l = out_port_l.as_mut_slice(ps);
            let out_p_r = out_port_r.as_mut_slice(ps);

            let in_p_l = in_port_l.as_slice(ps);
            let in_p_r = in_port_r.as_slice(ps);

            // TODO: Exponential smoothing
            // Check volume requests
            while let Some(v) = cons.pop() {
                db_destination = v;
                db_step_counter = step_amount;
                db_step_size = (db_destination - db_current) / step_amount as f32;
            }

            // Write output
            for (input_l, input_r, output_l, output_r) in izip!(in_p_l, in_p_r, out_p_l, out_p_r) {
                // Check if the current volume is at the destination by checking if there's steps left
                if db_step_counter > 0 {
                    db_step_counter -= 1;
                    db_current += db_step_size;
                }

                let lin_change = db2lin(db_current);
                *output_l = lin_change * input_l;
                *output_r = lin_change * input_r;
            }

            // Continue as normal
            jack::Control::Continue
        },
    );

    // 4. Activate the client. Also connect the ports to the system audio.
    let _active_client = client.activate_async((), process).unwrap();

    // 5. Start the GUI with the producer to send the parameters
    ui(prod);

    // 6. Optional deactivate. Not required since active_client will deactivate on
    // drop, though explicit deactivate may help you identify errors in
    // deactivate.
    //_active_client.deactivate().unwrap();
}

/// Attempt to read a frequency from standard in. Will block until there is
/// user input. `None` is returned if there was an error reading from standard
/// in, or the retrieved string wasn't a compatible u16 integer.
fn read_freq() -> Option<f32> {
    let mut user_input = String::new();
    match io::stdin().read_line(&mut user_input) {
        Ok(_) => f32::from_str(user_input.trim()).ok(),
        Err(_) => None,
    }
}

#[inline]
/// Converts a db change into a linear value
///
/// Source: https://mu.krj.st/mix/
///
fn db2lin(input: f32) -> f32 {
    10.0_f32.powf(input * 0.05) as f32
}
