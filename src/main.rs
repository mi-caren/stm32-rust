#![no_std]
#![no_main]

use panic_halt as _;

use cortex_m::peripheral::syst;
use cortex_m_rt::entry;
use stm32wl::stm32wl5x_cm4;

#[entry]
fn main() -> ! {
    let p = stm32wl5x_cm4::Peripherals::take().unwrap();

    let rcc = &p.RCC;
    // Enable clock for GPIOB peripheral
    rcc.ahb2enr.write(|w| w.gpioben().set_bit());

    let gpiob = &p.GPIOB;
    // Put GPIOB PIN 15 in OUTPUT MODE
    gpiob.moder.write(|w| w.moder15().bits(0b01));

    let cp = cortex_m::peripheral::Peripherals::take().unwrap();
    let mut systick = cp.SYST;
    systick.set_clock_source(syst::SystClkSource::Core);
    systick.set_reload(4_000_000);
    systick.clear_current();
    systick.enable_counter();

    let mut led_on = false;
    loop {
        while !systick.has_wrapped() {}

        if led_on {
            gpiob.bsrr.write(|w| w.br15().set_bit());
        } else {
            gpiob.bsrr.write(|w| w.bs15().set_bit());
        }
        led_on = !led_on;
    }
}
