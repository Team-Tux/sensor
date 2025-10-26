use alloc::boxed::Box;
use alloc::string::String;
use anyhow::Result;
use core::net::Ipv4Addr;
use embassy_executor::Spawner;
use embassy_net::udp::{PacketMetadata, UdpSocket};
use embassy_net::{Runner, Stack, StackResources};
use embassy_time::Timer;
use esp_radio::wifi;
use esp_radio::wifi::{ClientConfig, ModeConfig, WifiController, WifiDevice, WifiError};
use log::{info, warn};
use static_cell::make_static;

pub struct WifiStaDriver {
    address: Ipv4Addr,
    stack: Stack<'static>,
}

impl WifiStaDriver {
    pub async fn new(
        spawner: &Spawner,
        mut controller: WifiController<'static>,
        sta_interface: WifiDevice<'static>,
        ssid: String,
        password: String,
    ) -> Result<Self> {
        let sta_config = embassy_net::Config::dhcpv4(Default::default());

        let (sta_stack, sta_runner) = embassy_net::new(
            sta_interface,
            sta_config,
            make_static!(StackResources::<4>::new()),
            69,
        );

        let client_config = ModeConfig::Client(
            ClientConfig::default()
                .with_ssid(ssid)
                .with_password(password),
        );
        controller.set_mode(wifi::WifiMode::Sta)?;
        controller.set_config(&client_config)?;
        controller.start()?;

        spawner.spawn(wifi_runner(sta_runner))?;

        while !controller.is_started()? {
            info!("Waiting for controller to start");
            Timer::after_secs(1).await;
        }

        while let Err(e) = controller.connect_async().await {
            match e {
                WifiError::Disconnected => {
                    warn!("Couldn't connect to network")
                }
                _ => return Err(anyhow::anyhow!(e)),
            }
        }

        let _pls_not_drop = Box::leak(Box::new(controller));

        let address = loop {
            if let Some(config) = sta_stack.config_v4() {
                let address = config.address.address();
                info!("Got IP: {address}");
                break address;
            }
            info!("Waiting for dhcp ip");
            Timer::after_secs(1).await;
        };

        Ok(Self {
            address,
            stack: sta_stack,
        })
    }

    pub fn udp_socket(self) -> Result<UdpSocket<'static>> {
        let rx_meta = make_static!([PacketMetadata::EMPTY; 16]);
        let rx_buffer = make_static!([0; 1024]);
        let tx_meta = make_static!([PacketMetadata::EMPTY; 16]);
        let tx_buffer = make_static!([0; 1024]);

        let mut socket = UdpSocket::new(self.stack, rx_meta, rx_buffer, tx_meta, tx_buffer);
        socket
            .bind((self.address, 0))
            .map_err(|e| anyhow::anyhow!("{e:?}"))?;

        Ok(socket)
    }
}

#[embassy_executor::task]
async fn wifi_runner(mut runner: Runner<'static, WifiDevice<'static>>) {
    runner.run().await
}
