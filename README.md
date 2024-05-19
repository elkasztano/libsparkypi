# libsparkypi

Control radio switch sockets and similar radio controlled devices via [Linux GPIO userspace ABI](https://docs.kernel.org/driver-api/gpio/using-gpio.html) and an appropriate radio transmitter module (e.g. FS1000A).

## Usage:

Add to Cargo.toml:

```toml
[dependencies]
libsparkypi = "0.3.0"
```

## Example:

```rust
use libsparkypi::{Transmission, P1};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    let sequence = "s001100100101100101010110";

    let radio_signal = Transmission::builder()
        .sequence(sequence)
        .pulse_length(175)
        .repeats(5)
        .protocol(P1)
        .build();

    // send via specified gpio device and pin
    // change to '/dev/gpiochip4' on a Raspberry Pi 5
    // refer to user manual on other SBC's
    radio_signal.send_to("/dev/gpiochip0", 26)?;

    // output as bytes that can be sent over UART and
    // subsequently be parsed by an appropriately
    // programmed microcontroller which in turn can
    // send the actual radio signal using a transmitter
    // module
    let bytes = radio_signal.csv_as_bytes();
    
    println!("the following data may be parsed by a \
    microcontroller\nwith a connected radio module:\n");
    println!("{}", std::str::from_utf8(&bytes)?);

    Ok(())
}
```

The above example will transmit the binary sequence `001100100101100101010110` with a leading sync bit and sync gap and a pulse length of 175 microseconds 5 times using the predefined protocol 1 (`P1`) on pin 26 of the gpio chip `/dev/gpiochip0`.

The properties of the radio signal are then displayed on the terminal in csv format. You can also send the underlying byte sequence via UART to an appropriately programmed microcontroller (e.g. using the [serialport](https://crates.io/crates/serialport) crate), which in turn can send the actual radio signal via a connected transmitter module.

## Notes

* On a *Raspberry Pi 5* the 40-pin expansion header is on `/dev/gpiochip4`. On a *Raspberry Pi 4B* it is on `/dev/gpiochip0`. Refer to the user manual of your particular device.

* There are many different protocols for 433 Mhz data transmission (LPD433).

* It is easy to implement custom protocols via the builder pattern. (see [documentation](https://docs.rs/libsparkypi))

* Currently, presets for 3 different protocols are implemented in the form of constants. However, I only have the opportunity to test two of them (predefined constants `P1` and `XEN` are tested, `P2` is untested).

* If you have difficulties finding the right protocol for your target device, you might want to use an RTL SDR dongle to decode the signal of its counterpart (e.g. remote control, magnetic door sensor etc.). This also helps a lot in finding the right pulse length. Note that many of the available transmitter modules seem to be a bit laggy, so you might need to reduce the pulse length a little bit to compensate.

* `libsparkypi` *does not* rely on wiringpi, which seems to be deprecated.

* Earlier versions of `libsparkypi` rely on the [rppal](https://crates.io/crates/rppal) crate, which is excellent. However, I wanted to make the project a little bit more platform independent, so the project now relies on [gpio-cdev](https://crates.io/crates/gpio-cdev).