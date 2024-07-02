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
    subghz,
    dma,
};

use stm32wlxx_hal::cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    let mut p = pac::Peripherals::take().unwrap();
    let cp = pac::CorePeripherals::take().unwrap();

    // LED
    let gpiob = PortB::split(p.GPIOB, &mut p.RCC);
    let mut led = cortex_m::interrupt::free(|cs| Output::default(gpiob.b15, cs));

    // DELAY
    let mut delay = new_delay(cp.SYST, &p.RCC);

    // UART
    let gpioa = PortA::split(p.GPIOA, &mut p.RCC);
    let no_tx_uart = LpUart::new(p.LPUART, 115_200, uart::Clk::Sysclk, &mut p.RCC);
    let mut uart = cortex_m::interrupt::free(|cs| {
            no_tx_uart.enable_tx(gpioa.a2, cs)
            // .enable_rx(gpioa.a3, cs)
    });

    // SUBGHZ
    // AppKey: F41128AA66EEC52B25EDCF0E9503C7C7
    let dma = dma::AllDma::split(p.DMAMUX, p.DMA1, p.DMA2, &mut p.RCC);
    let mut sg = subghz::SubGhz::new_with_dma(p.SPI3, dma.d1.c1, dma.d1.c2, &mut p.RCC);
    let status = sg.status().unwrap();
    let mut _status_mode = status.mode().unwrap();
    let mut _status_cmd = status.cmd().unwrap();
    sg.set_standby(subghz::StandbyClk::Rc).unwrap();
    const TX_BASE_ADDR: u8 = 0;
    const RX_BASE_ADDR: u8 = 128;
    sg.set_buffer_base_address(TX_BASE_ADDR, RX_BASE_ADDR).unwrap();
    sg.set_packet_type(subghz::PacketType::LoRa).unwrap();
    sg.set_lora_packet_params(
        &subghz::LoRaPacketParams::new()
            .set_preamble_len(1234)
            .set_header_type(subghz::HeaderType::Fixed)
            .set_payload_len(16)
            .set_crc_en(true)
            .set_invert_iq(false)
    ).unwrap();
    sg.set_lora_sync_word(subghz::LoRaSyncWord::Private).unwrap();
    sg.set_rf_frequency(&subghz::RfFreq::F868).unwrap();
    sg.set_pa_config(&subghz::PaConfig::LP_10).unwrap();
    sg.set_tx_params(&subghz::TxParams::LP_10.set_ramp_time(subghz::RampTime::Micros40)).unwrap();
    sg.set_lora_mod_params(
        &subghz::LoRaModParams::new()
            .set_sf(subghz::SpreadingFactor::Sf7)
            .set_bw(subghz::LoRaBandwidth::Bw125)
            .set_cr(subghz::CodingRate::Cr45)
            .set_ldro_en(false)
    ).unwrap();
    sg.set_irq_cfg(
        &subghz::CfgIrq::new()
            .irq_enable_all(subghz::Irq::TxDone)
            .irq_enable_all(subghz::Irq::Timeout)
    ).unwrap();
    sg.write_buffer(TX_BASE_ADDR, b"Hello, World!").unwrap();

    loop {
        led.set_level_high();
        delay.delay_ms(1000);
        led.set_level_low();
        delay.delay_ms(1000);

        write!(uart, "Hello, World!\r\n").unwrap();

        sg.set_tx(subghz::Timeout::from_millis_sat(100)).unwrap();

        loop {
            let (status, irq_status) = sg.irq_status().unwrap();
            _status_mode = status.mode().unwrap();
            _status_cmd = status.cmd().unwrap();
            sg.clear_irq_status(irq_status).unwrap();

            if irq_status & subghz::Irq::TxDone.mask() != 0 {
                write!(uart, "LoRa message sent\r\n").unwrap();
                break;
            } else if irq_status & subghz::Irq::Timeout.mask() != 0 {
                write!(uart, "LoRa transmission timed out\r\n").unwrap();
                break;
            }
        }
    }
}
