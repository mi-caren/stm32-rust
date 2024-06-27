#![no_std]
#![no_main]

use panic_halt as _; // panic handler

use stm32wlxx_hal::{
    cortex_m,
    gpio::{Output, PortB},
    pac,
    util::new_delay,
};

use stm32wlxx_hal::cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    let mut p = pac::Peripherals::take().unwrap();
    let gpiob = PortB::split(p.GPIOB, &mut p.RCC);
    let mut led = cortex_m::interrupt::free(|cs| Output::default(gpiob.b15, cs));

    let cp = pac::CorePeripherals::take().unwrap();
    let mut delay = new_delay(cp.SYST, &p.RCC);

    loop {
        led.set_level_high();
        delay.delay_ms(1000);
        led.set_level_low();
        delay.delay_ms(1000);
    }
}
