use midir::{MidiOutput, MidiOutputPort};
use midly::live::LiveEvent;
use regex::Regex;
use std::error::Error;
use std::fs::File;
use std::io::{stdin, stdout, Write};
use std::io::{BufRead, BufReader};
// http://www.gmarts.org/data/jv-midiman.htm
mod mpe {
    use std::ops::RangeInclusive;
    pub const low_range: RangeInclusive<u8> = 1..=8;
    pub const high_range: RangeInclusive<u8> = 9..=16;
    pub const full_range: RangeInclusive<u8> = 1..=16;
}

// Three numbers, delimited by ':', which represent PC:MSB:LSB. You can put 'NULL' to not set the MSB/LSB.
// PC must be between 1...128
// MSB/LSB must be between 0...127
#[derive(Debug)]
struct Patch {
    pc: u8,
    msb: u8,
    lsb: u8,
    name: String,
}

fn extract_data_from_file(file_path: &str) -> Result<Vec<Patch>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let pattern = r"(\d+):(\d+):(\d+) (.+)";
    let re = Regex::new(pattern)?;
    let mut results = Vec::new();

    for line in reader.lines() {
        let line = line?;
        for (_, [pc, msb, lsb, name]) in re.captures_iter(&line).map(|c| c.extract()) {
            results.push(Patch {
                pc: pc.parse()?,
                msb: msb.parse()?,
                lsb: lsb.parse()?,
                name: name.to_string(),
            });
        }
    }

    Ok(results)
}
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
    // cc 00 00 cc 32 81 pc 30
    let controller = midly::num::u7::new(00);
    let value = midly::num::u7::new(00);
    let control_change1 = midly::MidiMessage::Controller { controller, value };
    let controller = midly::num::u7::new(32);
    let value = midly::num::u7::new(81);
    let control_change2 = midly::MidiMessage::Controller { controller, value };
    // this should be able to change banks
    let program = midly::num::u7::new(0x07);
    let program_message = midly::MidiMessage::ProgramChange { program };
    let patches = extract_data_from_file("patches.txt")?;
    dbg!(&patches.get(3));
    for channel in mpe::low_range {
        // Create the Control Change message
        let cc_message1 = LiveEvent::Midi {
            channel: midly::num::u4::from(channel), // MIDI channel 1 (zero-based)
            message: control_change1,
        };
        let cc_message2 = LiveEvent::Midi {
            channel: midly::num::u4::from(channel), // MIDI channel 1 (zero-based)
            message: control_change2,
        };
        let pc_message = LiveEvent::Midi {
            channel: midly::num::u4::from(channel), // MIDI channel 1 (zero-based)
            message: program_message,
        };

        let mut buffer = Vec::new();
        // Convert to raw MIDI bytes
        cc_message1.write(&mut buffer)?;
        cc_message2.write(&mut buffer)?;
        pc_message.write(&mut buffer)?;

        // Send the message
        conn_out.send(&buffer)?;
        println!("Sent Control Change message: Channel {channel}, Controller {controller}, Value {value}, PC {program}");
    }
    // Keep the connection open briefly
    std::thread::sleep(std::time::Duration::from_millis(100));

    Ok(())
}
