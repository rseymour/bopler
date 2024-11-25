use midir::{MidiOutput, MidiOutputPort};
use midly::live::LiveEvent;
use rand::seq::SliceRandom;
use std::error::Error;
use std::io::{stdin, stdout, Write};

mod boplib;
// http://www.gmarts.org/data/jv-midiman.htm
fn main() -> Result<(), Box<dyn Error>> {
    // Create a new MIDI output connection
    let midi_out = MidiOutput::new("My MIDI Output")?;

    // Get available ports
    let out_ports = midi_out.ports();

    // No ports available?
    if out_ports.is_empty() {
        println!("No MIDI output ports available!");
        return Ok(());
    }

    // List available ports
    println!("\nAvailable MIDI output ports:");
    for (i, p) in out_ports.iter().enumerate() {
        println!("{}: {}", i, midi_out.port_name(p)?);
    }

    // Ask user to select a port
    print!("Please select output port: ");
    stdout().flush()?;
    let mut input = String::new();
    stdin().read_line(&mut input)?;
    let port_number = input.trim().parse::<usize>()?.min(out_ports.len() - 1);

    // Connect to the selected port
    let port = &out_ports[port_number];
    println!("\nOpening connection");
    let mut conn_out = midi_out.connect(port, "midir-test")?;
    let patches = boplib::extract_data_from_file("all_patches.tsv")?;

    let rp = patches
        .choose(&mut rand::thread_rng())
        .expect("the world works");
    dbg!(&rp);
    let mpe_range = boplib::mpe::FULL_RANGE;
    boplib::set_patch(rp, mpe_range, &mut conn_out)?;
    // Keep the connection open briefly
    std::thread::sleep(std::time::Duration::from_millis(100));

    Ok(())
}
