use anyhow::Result;
use esp_hal::Async;
use esp_hal::uart::Uart;
use serde::de::DeserializeOwned;

pub struct UartDriver<'a> {
    uart: Uart<'a, Async>,
}

impl<'a> UartDriver<'a> {
    pub fn new(uart: Uart<'a, Async>) -> Self {
        Self { uart }
    }

    pub async fn read<T: DeserializeOwned>(&mut self) -> Result<Option<T>> {
        let mut buf = [0; 1024];
        let n = self.uart.read_async(&mut buf).await?;

        Ok(Some(postcard::from_bytes(&buf[..n])?))
    }
}
