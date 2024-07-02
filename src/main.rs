#![no_std]
#![no_main]

use panic_halt as _; // panic handler
use core::fmt::Write;

use stm32wlxx_hal::{
    cortex_m,
    gpio::{Output, PortA, PortB},
    pac,
    util::new_delay,
    uart::{self, Uart1},
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
    let pb5 = gpiob.b5;
    let mut led = cortex_m::interrupt::free(|cs| Output::default(pb5, cs));

    // DELAY
    let mut delay = new_delay(cp.SYST, &p.RCC);

    // UART
    // let gpioa = PortA::split(p.GPIOA, &mut p.RCC);
    let no_tx_uart = Uart1::new(p.USART1, 115_200, uart::Clk::Sysclk, &mut p.RCC);
    let pb6 = gpiob.b6;
    let mut uart = cortex_m::interrupt::free(|cs| {
            no_tx_uart.enable_tx(pb6, cs)
            // .enable_rx(gpioa.a3, cs)
    });

    // SUBGHZ
    // AppKey: F41128AA66EEC52B25EDCF0E9503C7C7
    const MSG: &str = "Hello, World!";
    const MSG_LEN :u8 = MSG.len() as u8;
    const MSG_BYTES: &[u8] = MSG.as_bytes();

    let dma = dma::AllDma::split(p.DMAMUX, p.DMA1, p.DMA2, &mut p.RCC);
    let mut sg = subghz::SubGhz::new_with_dma(p.SPI3, dma.d1.c1, dma.d1.c2, &mut p.RCC);

    sg.set_standby(subghz::StandbyClk::Rc).unwrap();
    // let status = sg.status().unwrap();
    // assert_ne!(status.cmd(), Ok(subghz::CmdStatus::ExecutionFailure));
    // assert_eq!(status.mode(), Ok(subghz::StatusMode::StandbyRc));
    const TCXO_MODE :subghz::TcxoMode = subghz::TcxoMode::new()
        .set_txco_trim(subghz::TcxoTrim::Volts1pt7)
        .set_timeout(subghz::Timeout::from_millis_sat(10));
    sg.set_tcxo_mode(&TCXO_MODE).unwrap();
    sg.set_standby(subghz::StandbyClk::Hse).unwrap();

    sg.set_tx_rx_fallback_mode(subghz::FallbackMode::StandbyHse).unwrap();
    sg.set_regulator_mode(subghz::RegMode::Ldo).unwrap();

    const TX_BASE_ADDR: u8 = 128;
    const RX_BASE_ADDR: u8 = 0;
    sg.set_buffer_base_address(TX_BASE_ADDR, RX_BASE_ADDR).unwrap();

    const PA_CONFIG :subghz::PaConfig = subghz::PaConfig::LP_10;
    sg.set_pa_config(&PA_CONFIG).unwrap();

    sg.set_pa_ocp(subghz::Ocp::Max60m).unwrap();

    const TX_PARAMS :subghz::TxParams = subghz::TxParams::LP_10
        .set_ramp_time(subghz::RampTime::Micros40);
    sg.set_tx_params(&TX_PARAMS).unwrap();

    sg.set_packet_type(subghz::PacketType::LoRa).unwrap();
    sg.set_lora_sync_word(subghz::LoRaSyncWord::Public).unwrap();

    sg.set_lora_mod_params(
        &subghz::LoRaModParams::new()
            .set_sf(subghz::SpreadingFactor::Sf7)
            .set_bw(subghz::LoRaBandwidth::Bw125)
            .set_cr(subghz::CodingRate::Cr45)
            .set_ldro_en(true)
    ).unwrap();

    sg.set_lora_packet_params(
        &subghz::LoRaPacketParams::new()
            .set_preamble_len(5 * 8)
            .set_header_type(subghz::HeaderType::Fixed)
            .set_payload_len(MSG_LEN)
            .set_crc_en(true)
            .set_invert_iq(false)
    ).unwrap();

    sg.calibrate_image(subghz::CalibrateImage::ISM_430_440).unwrap();

    sg.set_rf_frequency(&subghz::RfFreq::from_frequency(434_000_000)).unwrap();

    sg.write_buffer(TX_BASE_ADDR, MSG_BYTES).unwrap();

    sg.set_irq_cfg(
        &subghz::CfgIrq::new()
            .irq_enable_all(subghz::Irq::TxDone)
            .irq_enable_all(subghz::Irq::Timeout)
            .irq_enable_all(subghz::Irq::Err)
    ).unwrap();


    loop {
        led.set_level_high();
        delay.delay_ms(1000);
        led.set_level_low();
        delay.delay_ms(1000);

        write!(uart, "Hello, World! From Seeed Studio E5 Dev Board\r\n").unwrap();

        sg.set_tx(subghz::Timeout::DISABLED).unwrap();

        let mut times = 0;
        loop {
            let (status, irq_status) = sg.irq_status().unwrap();
            sg.clear_irq_status(irq_status).unwrap();

            if status.cmd() == Ok(subghz::CmdStatus::Complete) {
                write!(uart, "TX DONE\r\n").unwrap();
            }

            if irq_status & subghz::Irq::TxDone.mask() != 0 {
                write!(uart, "LoRa message sent\r\n").unwrap();
                break;
            } else if irq_status & subghz::Irq::Timeout.mask() != 0 {
                write!(uart, "LoRa transmission timed out\r\n").unwrap();
                break;
            } else if irq_status & subghz::Irq::Err.mask() != 0 {
                write!(uart, "LoRa transmission error\r\n").unwrap();
                break;
            }

            times += 1;
            if times == 100 {
                write!(uart, "Tried 100 times\r\n").unwrap();
                break;
            }
            delay.delay_ms(20);
        }
    }
}
