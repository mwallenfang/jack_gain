//! Gain plugin based on https://mu.krj.st/mix/

use ringbuf::RingBuffer;
use std::io;
use std::str::FromStr;

fn main() {
    // 1. open a client
    let (client, _status) =
        jack::Client::new("rust_jack_sine", jack::ClientOptions::NO_START_SERVER).unwrap();

    // 2. register port
    let mut out_port = client
        .register_port("gain_out", jack::AudioOut::default())
        .unwrap();

    let mut in_port = client
        .register_port("gain_in", jack::AudioIn::default())
        .unwrap();

    // 3. define process callback handler
    let rb = RingBuffer::<f32>::new(client.sample_rate());
    let (mut prod, mut cons) = rb.split();

    // TODO: Refactor the volume value flow, as the content of the variables isn't clear
    // Define the amount of steps to be the amount of samples in 50ms
    let step_amount: i32 = (client.sample_rate() as f32 * 0.05) as i32;

    let mut volume_current: f32 = 0.0;
    let mut volume_destination: f32 = 0.0;
    let mut volume_step_counter = step_amount;
    let mut volume_step_size = (volume_destination - volume_current) / step_amount as f32;

    let process = jack::ClosureProcessHandler::new(
        move |_: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
            // Get output buffer
            let out_p = out_port.as_mut_slice(ps);

            let in_p = in_port.as_slice(ps);

            // Check volume requests
            while let Some(v) = cons.pop() {
                volume_destination = v;
                volume_step_counter = step_amount;
                volume_step_size = (volume_destination - volume_current) / step_amount as f32;
                println!("received: {}", v);
            }

            // Write output
            for (input, output) in in_p.iter().zip( out_p.iter_mut()) {
                // Check if the current volume is at the destination by checking if there's steps left
                if volume_step_counter > 0 {
                    volume_step_counter -= 1;
                    volume_current += volume_step_size;
                }
                *output = db2lin(volume_current) * input;

                // y = x^4 is an approximation to the ideal exponential function for a dB range from
                // 0 to 60, with values between 0 and 1 for the volume
                //*output = volume.powf(4.0) * input;
            }

            // Continue as normal
            jack::Control::Continue
        },
    );

    // 4. Activate the client. Also connect the ports to the system audio.
    let active_client = client.activate_async((), process).unwrap();

    // processing starts here

    // 5. wait or do some processing while your handler is running in real time.
    loop {
        if let Some(f) = read_freq() {
            prod.push(f).unwrap();
        }
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

#[inline]
/// Converts a db change into a linear value
///
/// Source: https://mu.krj.st/mix/
///
fn db2lin(input: f32) -> f32 {
    10.0_f32.powf(input * 0.05) as f32
}
