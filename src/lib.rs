//! Generate and transmit radio signals that can be used to control
//! mains socket switches, doorbells and similar radio controlled
//! devices.

use std::{thread, time};
use gpio_cdev::{Chip, LineRequestFlags, LineHandle};

pub mod builder;
use crate::builder::{TransmissionBuilder, ProtocolBuilder};

/// A transmission consists of a sequence of short and long radio pulses.
/// The binary sequence is represented by a string of literal
/// '0' and '1' characters.
/// That way you can compose the signal simply by concatenating
/// multiple strings.
///
/// The pulse length is the smallest time unit of the signal.
/// It will be multiplied by the respective values specified
/// in the `ProtocolProperties` struct. The pulse length is
/// given in microseconds. Usually, a lot of fine tuning is 
/// required.
///
/// Usually, sending the signal once is not enough,
/// so we need to specify the number of repeats.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Transmission {
    pub sequence: String,
    pub pulse_length: u16,
    pub repeats: u8,
    pub protocol: ProtocolProperties,
}

/// In the protocol we define the smallest parts of the radio signal.
/// Usually a short pulse with a long pause resembles a binary zero,
/// and a long pulse followed by a short pause resembles a binary one.
/// A sync bit / sync gap combination marks the beginning of the radio transmission.
#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct ProtocolProperties {
    pub short: u8,
    pub long: u8,
    pub sync_bit: u8,
    pub sync_gap: u8,
}

impl ProtocolProperties {
    
    /// Initialize a custom protocol. Every field is set to zero.    
    pub fn new() -> Self {
        Default::default()
    }

    /// Invoke the builder pattern.
    pub fn builder() -> ProtocolBuilder {
        ProtocolBuilder::default()
    }
}

/// Resembles 'protocol1' of sui77's brilliant
/// [rc-switch library](https://github.com/sui77/rc-switch) 
/// for the arduino - probably the most common protocol.
pub const P1: ProtocolProperties = ProtocolProperties {
    short: 1,
    long: 3,
    sync_bit: 1,
    sync_gap: 31,
};

/// Resembles 'protocol2'.
/// Untested - your mileage may vary.
pub const P2: ProtocolProperties = ProtocolProperties {
    short: 1,
    long: 2,
    sync_bit: 1,
    sync_gap: 10,
};

/// Protocol for the 'Gmornxen' RC socket switches, very similar to 'protocol2'.
pub const XEN: ProtocolProperties = ProtocolProperties {
    short: 1,
    long: 2,
    sync_bit: 1,
    sync_gap: 11,
};

impl Transmission {
    
    /// Output the signal on a gpio pin on the specified gpio device via the
    /// [gpio character device ABI](https://www.kernel.org/doc/Documentation/ABI/testing/gpio-cdev).
    /// Finding the right device can sometimes be a bit tricky.
    /// Further information might be found in the documentation of your SBC.
    /// On a Raspberry Pi 4B it is `/dev/gpiochip0`, but on a
    /// Raspberry Pi 5 the appropiate device is `/dev/gpiochip4`.
    /// `gpio_pin` is the number of the gpio pin the radio transmitter module,
    /// e.g. an FS1000A module, is connected to.
    ///
    /// The sequence is transmitted by iterating over the characters of a string slice.
    /// If the character is '1' a binary one will be transmitted, or a binary zero if the character is
    /// '0', respectively.
    /// All other characters will result in a sync bit.
    ///
    /// # Examples
    /// ```rust
    /// use libsparkypi::*;
    ///
    /// let my_signal = Transmission::builder()
    ///     .sequence("s000000000000010101010001")
    ///     .pulse_length(320)
    ///     .repeats(10)
    ///     .protocol(P1)
    ///     .build();
    ///
    /// // output on device /dev/gpiochip0, gpio pin 18
    /// my_signal.send_to("/dev/gpiochip0", 18).unwrap();
    /// ```
    pub fn send_to(&self, gpio_dev: &str, gpio_pin: u8) -> Result<(), gpio_cdev::Error> {
        
        let mut chip = Chip::new(gpio_dev)?;

        let lh = chip
            .get_line(gpio_pin as u32)?
            .request(LineRequestFlags::OUTPUT, 0, "tx")?;

        for _ in 0..self.repeats {
            
            for c in self.sequence.chars() {
                
                if c == '1' {
                    send_bit(&lh, true, self.pulse_length, self.protocol.long, self.protocol.short)?;
                } else if c == '0' {
                    send_bit(&lh, false, self.pulse_length, self.protocol.long, self.protocol.short)?;
                } else {
                    send_sync_bit(&lh, self.pulse_length, self.protocol.sync_gap, self.protocol.sync_bit)?;
                }
            
            }
        
        }
    
        Ok(())
    }

    /// Modifies the binary sequence.
    pub fn sequence(&mut self, seq: &str) {
        self.sequence = String::from(seq);
    }

    /// Creates a new instance with default values, e.g. all fields
    /// with numbers will be zero, and the string will be
    /// empty.
    pub fn new() -> Self {
        Default::default()
    }

    /// Invokes the builder.
    pub fn builder() -> TransmissionBuilder {
        TransmissionBuilder::default()
    }

    /// Creates a byte vector containing the data from the transmission
    /// in the form of literal characters in the following comma
    /// separated pattern:
    ///
    /// `SEQUENCE,PULSE_LENGTH,REPEATS,SHORT,LONG,SYNC_BIT,SYNC_GAP`
    ///
    /// You can send those bytes e.g. via UART to an appropriately
    /// programmed microcontroller, which subsequently can parse the values,
    /// and transmit the binary sequence accordingly via a radio module.
    /// You can for example connect an Arduino Nano via USB to a regular
    /// x86 PC and make use of the [serialport](https://crates.io/crates/serialport)
    /// crate.
    pub fn csv_as_bytes(&self) -> Vec<u8> {
        let output = String::from(&format!("{},{},{},{},{},{},{}",
                                           self.sequence,
                                           self.pulse_length,
                                           self.repeats,
                                           self.protocol.short,
                                           self.protocol.long,
                                           self.protocol.sync_bit,
                                           self.protocol.sync_gap));
        output.as_bytes().to_vec()
    }
}

// Send one single bit
// Long pulse and short pause results in a binary one.
// Short pulse and long pause results in a binary zero.
// The relation between short and long period is defined in the 'ProtocolProperties' struct.

fn send_bit(lh: &LineHandle, bit: bool, pulse_length: u16, factor1: u8, factor2: u8) -> Result<(), gpio_cdev::Error> {
    if bit {
        lh.set_value(1)?;
        thread::sleep(time::Duration::from_micros(pulse_length as u64 * factor1 as u64));
        lh.set_value(0)?;
        thread::sleep(time::Duration::from_micros(pulse_length as u64 * factor2 as u64));
    } else {
        lh.set_value(1)?;
        thread::sleep(time::Duration::from_micros(pulse_length as u64 * factor2 as u64));
        lh.set_value(0)?;
        thread::sleep(time::Duration::from_micros(pulse_length as u64 * factor1 as u64));
    }
    Ok(())
}

// A so called sync bit must be transmitted before the actual binary sequence.
// The relation between 'high' and 'low' status is defined in the 'ProtocolProperties' struct.

fn send_sync_bit(lh: &LineHandle, pulse_length: u16, factor1: u8, factor2: u8) -> Result<(), gpio_cdev::Error> {
    lh.set_value(1)?;
    thread::sleep(time::Duration::from_micros(pulse_length as u64 * factor2 as u64));
    lh.set_value(0)?;
    thread::sleep(time::Duration::from_micros(pulse_length as u64 * factor1 as u64));
    Ok(())
}
