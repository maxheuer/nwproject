
#![no_std]
#![no_main]

use embassy_rp::gpio;
use embassy_time::Timer;
// use embassy_time::{Ticker, Duration};
use gpio::{Level, Output, Drive};
use panic_probe as _;

use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::USB;
use embassy_rp::usb::{Driver, InterruptHandler};
use embassy_usb_logger::ReceiverHandler;
use log::info;
use core::sync::atomic::{AtomicBool, Ordering::Relaxed};

async fn shift_register_write_bit(data: &mut Output<'_>, clock: &mut Output<'_>, bit: bool) {
    Timer::after_micros(1).await;
    data.set_level(bit.into());
    Timer::after_micros(1).await;
    clock.set_high();
    Timer::after_micros(1).await;
    clock.set_low();
}

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

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
});

#[embassy_executor::task]
async fn logger_task(driver: Driver<'static, USB>) {
    embassy_usb_logger::run!(1024, log::LevelFilter::Info, driver, Handler);
}

static mut CUR: [u16; 16] = [0; 16];
static FULL: AtomicBool = AtomicBool::new(true);

#[embassy_executor::main]
async fn main(spawner: embassy_executor::Spawner) {
    let p = embassy_rp::init(Default::default());

    let driver = Driver::new(p.USB, Irqs);
    spawner.spawn(logger_task(driver)).unwrap();

    let mut data = Output::new(p.PIN_0, Level::High);
    // data.set_drive_strength(Drive::_2mA); //  730 mV
    // data.set_drive_strength(Drive::_4mA); // 1080 mV
    // data.set_drive_strength(Drive::_8mA); // 1660 mV
    data.set_drive_strength(Drive::_12mA); // 1870 mV
    let mut clock = Output::new(p.PIN_1, Level::Low);
    let mut latch = Output::new(p.PIN_2, Level::High); // Latch is active low

    loop {
        if FULL.load(Relaxed) {
            info!("updated!");
            // this is annothing that i need this
            let cur = unsafe {(&raw const CUR).as_ref().unwrap()};
            info!("{cur:x?}");
            shift_register_write(&mut data, &mut clock, &mut latch, cur).await;
            FULL.store(false, Relaxed);
        }
        embassy_futures::yield_now().await;
    }
}

struct Handler;

impl ReceiverHandler for Handler {
    async fn handle_data(&self, data: &[u8]) { 
        if data.len() != 32 {info!("invalid data length: {}", data.len()); return}
        if FULL.load(Relaxed) {info!("already got packet!"); return}
        info!("rx");
        for i in 0..16 {
            unsafe {CUR[i] = data[i * 2] as u16 | ((data[i * 2 + 1] as u16) << 8)}; // little endian
        }
        FULL.store(true, Relaxed);
    }

    fn new() -> Self {
        Self
    }
}
