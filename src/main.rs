//! Gain plugin based on https://mu.krj.st/mix/

use atomic_float::AtomicF32;
use itertools::izip;
use std::sync::atomic::Ordering;
use std::sync::atomic::Ordering::Relaxed;

static GAIN_ATOMIC: AtomicF32 = AtomicF32::new(1.0);
static IN_L_ATOMIC: AtomicF32 = AtomicF32::new(0.0);
static IN_R_ATOMIC: AtomicF32 = AtomicF32::new(0.0);
static OUT_L_ATOMIC: AtomicF32 = AtomicF32::new(0.0);
static OUT_R_ATOMIC: AtomicF32 = AtomicF32::new(0.0);

const DYNAMIC_RANGE: f32 = -80.0;

mod meter;
mod ui;

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

    // Define the amount of steps to be the amount of samples in 50ms
    let step_amount: i32 = (client.sample_rate() as f32 * 0.05) as i32;

    let mut db_current: f32 = 0.0;
    let mut db_destination: f32 = 0.0;
    let mut db_step_counter = step_amount;
    let mut db_step_size = (db_destination - db_current) / step_amount as f32;
    let mut last_value = 1.0;


    let process = jack::ClosureProcessHandler::new(
        move |_: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
            // Get output buffer
            let out_p_l = out_port_l.as_mut_slice(ps);
            let out_p_r = out_port_r.as_mut_slice(ps);

            let in_p_l = in_port_l.as_slice(ps);
            let in_p_r = in_port_r.as_slice(ps);

            // TODO: Exponential smoothing
            // Calculate new volume settings if the parameter value has changed
            db_destination = GAIN_ATOMIC.load(Ordering::Relaxed);
            if db_destination != last_value {
                db_step_counter = step_amount;
                db_step_size = (db_destination - db_current) / step_amount as f32;
                last_value = db_destination
            }

            // Write output
            for (input_l, input_r, output_l, output_r) in izip!(in_p_l, in_p_r, out_p_l, out_p_r) {
                // Check if the current volume is at the destination by checking if there's steps left
                IN_L_ATOMIC.store((*input_l).abs(), Relaxed);
                IN_R_ATOMIC.store((*input_r).abs(), Relaxed);

                if db_step_counter > 0 {
                    db_step_counter -= 1;
                    db_current += db_step_size;
                }

                let lin_change = db2lin((1.0 - db_current) * DYNAMIC_RANGE);
                let out_l_val = lin_change * input_l;
                let out_r_val = lin_change * input_r;

                OUT_L_ATOMIC.store(out_l_val.abs(), Relaxed);
                OUT_R_ATOMIC.store(out_r_val.abs(), Relaxed);

                *output_l = out_l_val;
                *output_r = out_r_val;
            }

            // Continue as normal
            jack::Control::Continue
        },
    );

    // 4. Activate the client. Also connect the ports to the system audio.
    let _active_client = client.activate_async((), process).unwrap();

    // 5. Start the GUI with the producer to send the parameters
    ui::ui();
}

#[inline]
/// Converts a db change into a linear value
///
/// Source: https://mu.krj.st/mix/
///
fn db2lin(input: f32) -> f32 {
    10.0_f32.powf(input * 0.05) as f32
}
