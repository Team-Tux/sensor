//! embassy hello world
//!
//! This is an example of running the embassy executor with multiple tasks
//! concurrently.

#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]
#![feature(type_alias_impl_trait)]

mod driver;
mod sniffer;

extern crate alloc;

use crate::driver::wifi::WifiStaDriver;
use crate::sniffer::wifi::WifiSniffer;
use alloc::string::ToString;
use embassy_executor::Spawner;
use embassy_net::Runner;
use esp_alloc as _;
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
#[cfg(target_arch = "riscv32")]
use esp_hal::interrupt::software::SoftwareInterruptControl;
use esp_hal::timer::timg::TimerGroup;
use esp_radio::wifi;
use esp_radio::wifi::WifiDevice;
use log::{LevelFilter, info, trace};
use sensor_lib::{SensorConfig, SensorPacket};
use static_cell::make_static;

const SENSOR_PACKET_SIZE: usize = size_of::<SensorPacket>();

esp_bootloader_esp_idf::esp_app_desc!();

#[esp_rtos::main]
async fn main(spawner: Spawner) {
    esp_println::logger::init_logger(LevelFilter::Info);
    trace!("logger initialized");

    let peripherals = esp_hal::init(esp_hal::Config::default().with_cpu_clock(CpuClock::max()));

    esp_alloc::heap_allocator!(size: 128 * 1024);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    #[cfg(target_arch = "riscv32")]
    let sw_int = SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(
        timg0.timer0,
        #[cfg(target_arch = "riscv32")]
        sw_int.software_interrupt0,
    );

    info!("sensor-node firmware v{}", env!("CARGO_PKG_VERSION"),);

    #[cfg(feature = "esp32s3")]
    let mut config_receiver = {
        let usb =
            esp_hal::otg_fs::Usb::new(peripherals.USB0, peripherals.GPIO20, peripherals.GPIO19);
        driver::usb::UsbDriver::new(&spawner, usb).unwrap()
    };
    #[cfg(not(feature = "esp32s3"))]
    let mut config_receiver = {
        #[cfg(feature = "esp32c6")]
        let rx_pin = peripherals.GPIO17;

        let uart =  esp_hal::uart::Uart::new(
            peripherals.UART0,
            esp_hal::uart::Config::default()
                .with_rx(esp_hal::uart::RxConfig::default().with_fifo_full_threshold(64))
                .with_baudrate(9600),
        )
        .unwrap();
        driver::uart::UartDriver::new(uart.with_rx(rx_pin).into_async())
    };

    let config: SensorConfig = loop {
        info!("Waiting for initial config upload via usb");

        if let Some(config) = config_receiver.read().await.unwrap() {
            info!("Received initial config");
            break config;
        }
    };

    let esp_radio_ctrl = make_static!(esp_radio::init().unwrap());
    let (controller, interfaces) =
        wifi::new(esp_radio_ctrl, peripherals.WIFI, Default::default()).unwrap();

    let wifi_driver = WifiStaDriver::new(
        &spawner,
        controller,
        interfaces.sta,
        config.collector_network_ssid.to_string(),
        config.collector_network_password.to_string(),
    )
    .await
    .unwrap();

    let udp_socket = wifi_driver.udp_socket().unwrap();

    let mut wifi_sniffer = WifiSniffer::new(interfaces.sniffer, &config).unwrap();
    loop {
        let sensor_packet = wifi_sniffer.receive().await;
        let sensor_packet_data = postcard::to_vec::<_, SENSOR_PACKET_SIZE>(&sensor_packet).unwrap();

        let _ = udp_socket
            .send_to(
                &sensor_packet_data,
                (config.collector_service_ip, config.collector_service_port),
            )
            .await;
    }
}

#[embassy_executor::task]
async fn sta_run(mut runner: Runner<'static, WifiDevice<'static>>) {
    runner.run().await
}
