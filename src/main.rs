//! Sine wave generator with frequency configuration exposed through standard
//! input.

use crossbeam_channel::{bounded, Sender};
use std::io;
use std::str::FromStr;

fn main() {
    // 1. open a client
    let (client, _status) =
        jack::Client::new("rust_jack_sine", jack::ClientOptions::NO_START_SERVER).unwrap();

    // 2. register port
    let mut out_port = client
        .register_port("fader_out", jack::AudioOut::default())
        .unwrap();

    let mut in_port = client
        .register_port("fader_in", jack::AudioIn::default())
        .unwrap();

    // 3. define process callback handler
    let (tx, rx) = bounded(1_000_000);
    let mut volume = db2lin(0.0);
    let process = jack::ClosureProcessHandler::new(
        move |_: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
            // Get output buffer
            let out_p = out_port.as_mut_slice(ps);

            let in_p = in_port.as_slice(ps);

            // Check volume requests
            while let Ok(v) = rx.try_recv() {
                volume = v;
                println!("received: {} {}", v, volume);
            }

            // Write output
            for (input, output) in in_p.iter().zip( out_p.iter_mut()) {
                *output = db2lin(volume) * input;
            }

            // Continue as normal
            jack::Control::Continue
        },
    );

    // 4. Activate the client. Also connect the ports to the system audio.
    let active_client = client.activate_async((), process).unwrap();

    // processing starts here

    // 5. wait or do some processing while your handler is running in real time.
    let mut egui_ctx = egui::CtxRef::default();

    loop {

    }

    // 6. Optional deactivate. Not required since active_client will deactivate on
    // drop, though explicit deactivate may help you identify errors in
    // deactivate.
    active_client.deactivate().unwrap();
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

fn db2lin(input: f32) -> f32 {
    10.0_f32.powf(input * 0.05) as f32
}

fn lin2db(input: f32) -> f32 {
    20_f32 * input.log10()
}