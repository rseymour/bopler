use midly::live::LiveEvent;
use regex::Regex;
use std::fs::File;
use std::io::{BufRead, BufReader};

use std::error::Error;

pub fn set_patch(
    rp: &Patch,
    mpe_range: std::ops::RangeInclusive<u8>,
    conn_out: &mut midir::MidiOutputConnection,
) -> Result<(), Box<dyn Error>> {
    Ok(for channel in mpe_range {
        //let buffer = choose_patch(rp.pc.into(),  rp.msb.into(),rp.lsb.into(), channel)?;
        //pc seems off by one
        // and some don't load? 213 84 3 (card)
        // see google sheets for real:
        // https://docs.google.com/spreadsheets/d/1F2HihOomA8cItVsjR-6l4r20PYHsWU9bU6SEac03thA/edit?gid=989494382#gid=989494382
        let buffer = choose_patch(rp.pc.into(), rp.msb.into(), rp.lsb.into(), channel)?;
        conn_out.send(&buffer)?;

        let buffer = bend_params(channel)?;
        conn_out.send(&buffer)?;
    })
}

pub fn bend_params(channel: u8) -> Result<Vec<u8>, Box<dyn Error>> {
    let bend_change1 = midly::MidiMessage::Controller {
        controller: 0x31.into(),
        value: 12.into(),
    };
    let cc_message1 = LiveEvent::Midi {
        channel: midly::num::u4::from(channel), // MIDI channel 1 (zero-based)
        message: bend_change1,
    };
    let bend_change2 = midly::MidiMessage::Controller {
        controller: 0x32.into(),
        value: 12.into(),
    };
    let cc_message2 = LiveEvent::Midi {
        channel: midly::num::u4::from(channel), // MIDI channel 1 (zero-based)
        message: bend_change2,
    };
    let bend_range = vec![
        midly::MidiMessage::Controller {
            controller: 0x65.into(),
            value: 0x00.into(),
        },
        midly::MidiMessage::Controller {
            controller: 0x64.into(),
            value: 0x00.into(),
        },
        midly::MidiMessage::Controller {
            controller: 0x26.into(),
            value: 0x00.into(),
        },
        midly::MidiMessage::Controller {
            controller: 0x06.into(),
            value: 0x0C.into(),
        },
        midly::MidiMessage::Controller {
            controller: 0x64.into(),
            value: 0x7f.into(),
        },
        midly::MidiMessage::Controller {
            controller: 0x65.into(),
            value: 0x7f.into(),
        },
    ];
    let mut buffer = Vec::new();
    cc_message1.write(&mut buffer)?;
    cc_message2.write(&mut buffer)?;
    for b in bend_range.into_iter() {
        let msg = LiveEvent::Midi {
            channel: midly::num::u4::from(channel), // MIDI channel 1 (zero-based)
            message: b,
        };
        msg.write(&mut buffer)?;
    }
    Ok(buffer)
}

pub fn choose_patch(
    program: midly::num::u7,
    value_msb: midly::num::u7,
    value_lsb: midly::num::u7,
    channel: u8,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let program_message = midly::MidiMessage::ProgramChange { program };
    let controller = midly::num::u7::new(00);
    let control_change1 = midly::MidiMessage::Controller {
        controller,
        value: value_msb,
    };
    let controller = midly::num::u7::new(32);
    let control_change2 = midly::MidiMessage::Controller {
        controller,
        value: value_lsb,
    };
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
    cc_message1.write(&mut buffer)?;
    cc_message2.write(&mut buffer)?;
    pc_message.write(&mut buffer)?;
    //println!("Sent Control Change message: Channel {channel}, Controller {controller}, Value {value}, PC {program}");
    Ok(buffer)
}
pub mod mpe {
    use std::ops::RangeInclusive;
    pub const LOW_RANGE: RangeInclusive<u8> = 1..=8;
    pub const HIGH_RANGE: RangeInclusive<u8> = 9..=16;
    pub const FULL_RANGE: RangeInclusive<u8> = 1..=16;
}

// Three numbers, delimited by ':', which represent PC:MSB:LSB. You can put 'NULL' to not set the MSB/LSB.
// PC must be between 1...128
// MSB/LSB must be between 0...127
#[derive(Debug)]
pub struct Patch {
    pc: u8,
    msb: u8,
    lsb: u8,
    name: String,
    category: String,
}

pub fn extract_data_from_file(file_path: &str) -> Result<Vec<Patch>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let pattern = r"(\d+)\t(\d+)\t(\d+)\t(.+?)\t(.+)";
    let re = Regex::new(pattern)?;
    let mut results = Vec::new();

    for line in reader.lines() {
        let line = line?;
        for (_, [pc, msb, lsb, name, category]) in re.captures_iter(&line).map(|c| c.extract()) {
            results.push(Patch {
                pc: pc.parse()?,
                msb: msb.parse()?,
                lsb: lsb.parse()?,
                name: name.to_string(),
                category: category.to_string(),
            });
        }
    }

    Ok(results)
}
