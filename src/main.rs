use midir::{MidiOutput, MidiOutputPort};
use midly::live::LiveEvent;
use std::error::Error;
use std::io::{stdin, stdout, Write};

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
    let controller = midly::num::u7::new(00);
    let value = midly::num::u7::new(0x51);
    let control_change = midly::MidiMessage::Controller { controller, value };
    let low_range = 1..=8;
    for channel in low_range {
        // Create the Control Change message
        let cc_message = LiveEvent::Midi {
            channel: midly::num::u4::from(channel), // MIDI channel 1 (zero-based)
            message: control_change,
        };

        // Convert to raw MIDI bytes
        let mut buffer = Vec::new();
        cc_message.write(&mut buffer)?;

        // Send the message
        conn_out.send(&buffer)?;
        println!("Sent Control Change message: Channel {channel}, Controller {controller}, Value {value}");
    }
    // Keep the connection open briefly
    std::thread::sleep(std::time::Duration::from_millis(100));

    Ok(())
}
