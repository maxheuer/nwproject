
#![no_std]
#![no_main]

use embassy_rp::gpio;
use embassy_time::Timer;
use embassy_time::{Ticker, Duration};
use gpio::{Level, Output, Drive};
use panic_probe as _;

async fn shift_register_write_bit(data: &mut Output<'_>, clock: &mut Output<'_>, bit: bool) {
    Timer::after_micros(1).await;
    data.set_level(bit.into());
    Timer::after_micros(1).await;
    clock.set_high();
    Timer::after_micros(1).await;
    clock.set_low();
}

async fn shift_register_write_all(data: &mut Output<'_>, clock: &mut Output<'_>, latch: &mut Output<'_>, bit: bool) {
    for _i in 0..256 {
        shift_register_write_bit(data, clock, bit).await;
    }
    latch.set_low();
    Timer::after_micros(1).await;
    latch.set_high();
}

// cur is a 16x16 bit array
// each row corresponds to Y connections (cur[0] => Y0)
// each bit in each row corresponds to Y-X connections (cur[0] bit 0 => Y0-X0)
async fn shift_register_write(data: &mut Output<'_>, clock: &mut Output<'_>, latch: &mut Output<'_>, cur: &[u16; 16]) {
    for i in 0..16 { 
        for j in 0..16 {
            // cur in reverse because the device takes Y15 -> Y0
            // output MSB first (that's what device wants)
            let bit = 0 != (cur[15 - i] & (0x8000 >> j));
            shift_register_write_bit(data, clock, bit).await;
        }
    }
    Timer::after_micros(1).await;
    latch.set_low();
    Timer::after_micros(1).await;
    latch.set_high();
}

#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) {
    let p = embassy_rp::init(Default::default()); // peripherals

    let mut data = Output::new(p.PIN_0, Level::High); // Initial high to check output voltage
    // data.set_drive_strength(Drive::_2mA); //  730 mV
    // data.set_drive_strength(Drive::_4mA); // 1080 mV
    // data.set_drive_strength(Drive::_8mA); // 1660 mV
    data.set_drive_strength(Drive::_12mA); // 1870 mV
    let mut clock = Output::new(p.PIN_1, Level::Low);
    let mut latch = Output::new(p.PIN_2, Level::High); // Latch is active low

    // reset all to 0
    shift_register_write_all(&mut data, &mut clock, &mut latch, false).await;

    let mut ticker = Ticker::every(Duration::from_secs(2)); // 2 second delay between each tick to measure results w/ a simple ohmmeter
    let mut cur = [ 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 ]; // Initial Y0 - X0 + Y1 - X0
    loop {
        // All 0 -> All 1 loop
        // shift_register_write_all(&mut data, &mut clock, &mut latch, false).await;
        // ticker.next().await;
        // shift_register_write_all(&mut data, &mut clock, &mut latch, true).await;
        // ticker.next().await;

        // Shift loop
        shift_register_write(&mut data, &mut clock, &mut latch, &cur).await;
        cur[1] <<= 1; // move Y1 - X0 -> X1 -> X2 etc...
        ticker.next().await;
    }
}
