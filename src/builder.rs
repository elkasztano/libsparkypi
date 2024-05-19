//! Construct a radio transmission struct with calls to a builder helper.

use crate::{Transmission, ProtocolProperties};

#[derive(Default)]
/// Creates an instance of a transmission struct.
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
///```
pub struct TransmissionBuilder {
    sequence: String,
    pulse_length: u16,
    repeats: u8,
    protocol: ProtocolProperties,
}

impl TransmissionBuilder {

    /// Creates a new builder with default values.
    pub fn new() -> TransmissionBuilder {
        TransmissionBuilder {
            sequence: String::from(""),
            pulse_length: 0,
            repeats: 0,
            protocol: ProtocolProperties::default()
        }
    }

    /// Specify the binary sequence of the radio transmission by
    /// a string slice containing literal '0' and '1' characters.
    /// Every character other than that will result in a sync bit,
    /// so you can e.g. put an 's' at the very beginning of the 
    /// string slice to get a leading sync bit / sync gap.
    pub fn sequence(mut self, seq: &str) -> TransmissionBuilder {
        self.sequence = String::from(seq);
        self
    }

    /// Specify the pulse length of the radio transmission. The pulse
    /// length can be seen as the smallest time unit of the transmission.
    /// It will be multiplied by the values from the fields in the
    /// protocol struct.
    pub fn pulse_length(mut self, pl: u16) -> TransmissionBuilder {
        self.pulse_length = pl;
        self
    }

    /// Sending a binary sequence once is not enough in most cases.
    /// You can specify the number of repeats here. Finding the
    /// right number may require a bit of try and error.
    /// If the number is very high the transmission will take
    /// inappropriately long and may block other devices operating
    /// on the same frequency. If the number is too low the
    /// target device will probably not react at all.
    /// Typical values are in the range of `3..=10`.
    pub fn repeats(mut self, rep: u8) -> TransmissionBuilder {
        self.repeats = rep;
        self
    }

    /// Specify the protocol of the radio transmission, i.e. specify the
    /// multiples of the pulse length that resemble a binary one or zero
    /// and the sync bit and sync gap respectively.
    pub fn protocol(mut self, pr: ProtocolProperties) -> TransmissionBuilder {
        self.protocol = pr;
        self
    }

    /// Finalizes the build and creates a `Transmission` struct.
    pub fn build(&self) -> Transmission {
        Transmission {
            sequence: self.sequence.clone(),
            pulse_length: self.pulse_length,
            repeats: self.repeats,
            protocol: self.protocol
        }
    }

}

#[derive(Default)]
/// Creates an instance of a transmission protocol struct.
///
/// # Examples
/// ```rust
/// use libsparkypi::*;
///
/// let my_protocol = ProtocolProperties::builder()
///     .short(1)
///     .long(3)
///     .sync_bit(1)
///     .sync_gap(31)
///     .build();
/// 
/// assert_eq!(P1, my_protocol);
/// ```
pub struct ProtocolBuilder {
    short: u8,
    long: u8,
    sync_bit: u8,
    sync_gap: u8,
}

impl ProtocolBuilder {
    
    /// Creates a new instance with default values,
    /// e.g. every field is zero.
    pub fn new() -> Self {
        Self::default()
    }

    /// Specify the factor that results in a short
    /// pulse when multiplied with the pulse length.
    pub fn short(mut self, short: u8) -> Self {
        self.short = short;
        self
    }

    /// Specify the factor that results in a long
    /// pulse when multiplied with the pulse length.
    pub fn long(mut self, long: u8) -> Self {
        self.long = long;
        self
    }

    /// Specify the factor that results in the sync
    /// bit length when multiplied with the pulse length.
    pub fn sync_bit(mut self, sync_bit: u8) -> Self {
        self.sync_bit = sync_bit;
        self
    }

    /// Specifiy the factor that results in the sync
    /// gap length when multiplied with the pulse length.
    pub fn sync_gap(mut self, sync_gap: u8) -> Self {
        self.sync_gap = sync_gap;
        self
    }

    /// Creates a new instance of `ProtocolProperties`
    pub fn build(&self) -> ProtocolProperties {
        ProtocolProperties {
            short: self.short,
            long: self.long,
            sync_bit: self.sync_bit,
            sync_gap: self.sync_gap
        }
    }

}
