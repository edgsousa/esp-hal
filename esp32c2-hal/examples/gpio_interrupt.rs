//! GPIO interrupt
//!
//! This prints "Interrupt" when the boot button is pressed.
//! It also blinks an LED like the blinky example.

#![no_std]
#![no_main]

use core::cell::RefCell;

use critical_section::Mutex;
use esp32c2_hal::{
    clock::ClockControl,
    gpio::{Gpio9, IO},
    gpio_types::{Event, Input, Pin, PullDown},
    interrupt,
    pac::{self, Peripherals},
    prelude::*,
    timer::TimerGroup,
    Delay,
    Rtc,
};
use esp_backtrace as _;
use riscv_rt::entry;

static BUTTON: Mutex<RefCell<Option<Gpio9<Input<PullDown>>>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take().unwrap();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    // Disable the watchdog timers. For the ESP32-C2, this includes the Super WDT,
    // the RTC WDT, and the TIMG WDTs.
    let mut rtc = Rtc::new(peripherals.RTC_CNTL);
    let timer_group0 = TimerGroup::new(peripherals.TIMG0, &clocks);
    let mut wdt0 = timer_group0.wdt;

    rtc.swd.disable();
    rtc.rwdt.disable();
    wdt0.disable();

    // Set GPIO5 as an output
    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let mut led = io.pins.gpio5.into_push_pull_output();

    // Set GPIO9 as an input
    let mut button = io.pins.gpio9.into_pull_down_input();
    button.listen(Event::FallingEdge);

    critical_section::with(|cs| BUTTON.borrow_ref_mut(cs).replace(button));

    interrupt::enable(pac::Interrupt::GPIO, interrupt::Priority::Priority3).unwrap();

    unsafe {
        riscv::interrupt::enable();
    }

    let mut delay = Delay::new(&clocks);
    loop {
        led.toggle().unwrap();
        delay.delay_ms(500u32);
    }
}

#[interrupt]
fn GPIO() {
    critical_section::with(|cs| {
        esp_println::println!("GPIO interrupt");
        BUTTON
            .borrow_ref_mut(cs)
            .as_mut()
            .unwrap()
            .clear_interrupt();
    });
}