use anyhow::Result;
use embassy_executor::Spawner;
use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use embassy_usb::{Builder, UsbDevice};
use esp_hal::otg_fs::Usb;
use esp_hal::otg_fs::asynch::Driver;
use serde::de::DeserializeOwned;
use static_cell::make_static;

const VID: u16 = 0xFFFF;
const PID: u16 = 0xFFFF;

const CLASS_VENDOR_SPECIFIC: u8 = 0xFF;
const MANUFACTURER: &str = "teamtux";
const PRODUCT: &str = "sensor-01";
const SERIAL_NUMBER: &str = "5735294812347145";

pub struct UsbDriver {
    class: CdcAcmClass<'static, Driver<'static>>,
}

impl UsbDriver {
    pub fn new(spawner: &Spawner, usb: Usb<'static>) -> Result<Self> {
        let ep_out_buffer = make_static!([0; 1024]);

        let driver = Driver::new(usb, ep_out_buffer, Default::default());

        let mut config = embassy_usb::Config::new(VID, PID);
        config.device_class = CLASS_VENDOR_SPECIFIC;
        config.manufacturer = Some(MANUFACTURER);
        config.product = Some(PRODUCT);
        config.serial_number = Some(SERIAL_NUMBER);

        // Required for windows compatibility.
        // https://developer.nordicsemi.com/nRF_Connect_SDK/doc/1.9.1/kconfig/CONFIG_CDC_ACM_IAD.html#help
        config.device_class = 0xEF;
        config.device_sub_class = 0x02;
        config.device_protocol = 0x01;
        config.composite_with_iads = true;

        let config_descriptor = make_static!([0; 256]);
        let bos_descriptor = make_static!([0; 256]);
        let control_buf = make_static!([0; 64]);

        let state = make_static!(State::new());
        let mut builder = Builder::new(
            driver,
            config,
            config_descriptor,
            bos_descriptor,
            &mut [],
            control_buf,
        );

        let class = CdcAcmClass::new(&mut builder, state, 64);
        let usb = builder.build();

        spawner.spawn(usb_runner(usb))?;

        Ok(Self { class })
    }

    pub async fn read<T: DeserializeOwned>(&mut self) -> Result<Option<T>> {
        self.class.wait_connection().await;

        let mut buf = [0; 1024];
        let mut total: usize = 0;

        let mut n = self.class.max_packet_size() as usize;
        while n == self.class.max_packet_size() as usize {
            n = self
                .class
                .read_packet(&mut buf[total..(total + n)])
                .await
                .map_err(|e| anyhow::anyhow!("{e:?}"))?;

            total += n;
        }

        Ok(Some(postcard::from_bytes(&buf[..total])?))
    }
}

#[embassy_executor::task]
async fn usb_runner(mut usb: UsbDevice<'static, Driver<'static>>) {
    usb.run().await
}
