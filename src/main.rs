#![no_std]
#![no_main]

use panic_halt as _; // panic handler
use core::fmt::Write;

use stm32wlxx_hal::{
    cortex_m,
    gpio::{Output, PortA, PortB},
    pac,
    util::new_delay,
    uart::{self, LpUart},
};

use stm32wlxx_hal::cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    let mut p = pac::Peripherals::take().unwrap();
    let cp = pac::CorePeripherals::take().unwrap();

    let gpiob = PortB::split(p.GPIOB, &mut p.RCC);
    let mut led = cortex_m::interrupt::free(|cs| Output::default(gpiob.b15, cs));

    let mut delay = new_delay(cp.SYST, &p.RCC);

    let gpioa = PortA::split(p.GPIOA, &mut p.RCC);
    let no_tx_uart = LpUart::new(p.LPUART, 115_200, uart::Clk::Sysclk, &mut p.RCC);
    let mut uart = cortex_m::interrupt::free(|cs| {
            no_tx_uart.enable_tx(gpioa.a2, cs)
            // .enable_rx(gpioa.a3, cs)
    });

    loop {
        led.set_level_high();
        delay.delay_ms(1000);
        led.set_level_low();
        delay.delay_ms(1000);

        write!(uart, "Hello, World!\r\n").unwrap();
    }
}
