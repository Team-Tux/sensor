use anyhow::Result;
use core::mem::MaybeUninit;
use core::net::Ipv4Addr;
use embassy_net::udp::UdpSocket;
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::{Channel, Receiver, Sender};
use esp_radio::wifi::Sniffer;
use ieee80211::mgmt_frame::ProbeRequestFrame;
use ieee80211::scroll::ctx::TryFromCtx;
use log::error;
use sensor_lib::{Environment, SensorConfig, SensorPacket};
use static_cell::make_static;

const SNIFFER_QUEUE_SIZE: usize = 32;

/// Global sniffer sender.
/// This must be a global static variable as `sniffer.set_receive_cb` in [`WifiSniffer::new`]
/// needs a function pointer which can only use global outer variables.
static SNIFF_RECEIVE_CONFIG: Mutex<CriticalSectionRawMutex, MaybeUninit<WifiSnifferReceiveConfig>> =
    Mutex::new(MaybeUninit::uninit());

struct WifiSnifferReceiveConfig {
    sender: Sender<'static, CriticalSectionRawMutex, SensorPacket, SNIFFER_QUEUE_SIZE>,

    pub sensor_id: u8,
    pub x: f64,
    pub y: f64,
    pub latitude: f64,
    pub longitude: f64,
    pub _environment: Environment,
}

pub struct WifiSniffer {
    receiver: Receiver<'static, CriticalSectionRawMutex, SensorPacket, SNIFFER_QUEUE_SIZE>,
}

impl WifiSniffer {
    pub fn new(mut sniffer: Sniffer, config: &SensorConfig) -> Result<Self> {
        let sniff_channel: &'static mut Channel<CriticalSectionRawMutex, SensorPacket, 32> =
            make_static!(Channel::new());

        unsafe {
            SNIFF_RECEIVE_CONFIG.lock_mut(|conf| {
                conf.write(WifiSnifferReceiveConfig {
                    sender: sniff_channel.sender(),
                    sensor_id: config.sensor_id,
                    x: config.x,
                    y: config.y,
                    latitude: config.latitude,
                    longitude: config.longitude,
                    _environment: config.environment,
                });
            });
        }

        sniffer.set_promiscuous_mode(true)?;
        sniffer.set_receive_cb(|packet| {
            let Ok((probe_req_frame, _)) = ProbeRequestFrame::try_from_ctx(packet.data, false)
            else {
                return;
            };

            let mut fingerprint = [0; 8];
            fingerprint[0..6].copy_from_slice(&probe_req_frame.header.transmitter_address.0);

            // SAFETY: The inner value of `SNIFF_RECEIVE_CONFIG` is always set - this is done above.
            SNIFF_RECEIVE_CONFIG.lock(|conf| unsafe {
                let conf = conf.assume_init_ref();
                let _ = conf.sender.try_send(SensorPacket {
                    sensor_id: conf.sensor_id,
                    x: conf.x,
                    y: conf.y,
                    latitude: conf.latitude,
                    longitude: conf.longitude,
                    environment: Environment::FreeSpace,
                    rssi: packet.rx_cntl.rssi as u8 as i8,
                    fingerprint: u64::from_be_bytes(fingerprint),
                });
            });
        });

        Ok(Self {
            receiver: sniff_channel.receiver(),
        })
    }

    pub async fn receive(&mut self) -> SensorPacket {
        self.receiver.receive().await
    }
}

#[embassy_executor::task]
pub async fn wifi_sniffer_task(
    wifi_sniffer: WifiSniffer,
    socket: UdpSocket<'static>,
    socket_endpoint: (Ipv4Addr, u16),
) {
    const SENSOR_PACKET_SIZE: usize = size_of::<SensorPacket>();

    loop {
        let packet = wifi_sniffer.receiver.receive().await;
        let packet_data = postcard::to_vec::<_, SENSOR_PACKET_SIZE>(&packet).unwrap();
        if let Err(e) = socket.send_to(&packet_data, socket_endpoint).await {
            error!("{e:?}");
        }
    }
}
