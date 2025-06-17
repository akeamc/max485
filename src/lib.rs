#![no_std]
use core::fmt::Debug;
use embedded_hal::digital::OutputPin;
use embedded_io_async::{ErrorType, Read, Write};

/// Custom Error type
#[derive(Debug)]
pub enum Error<S: embedded_io_async::Error, P: Debug> {
    Serial(S),
    Pin(P),
}

impl<S, P> embedded_io_async::Error for Error<S, P>
where
    S: embedded_io_async::Error,
    P: Debug,
{
    fn kind(&self) -> embedded_io_async::ErrorKind {
        match self {
            Error::Serial(s) => s.kind(),
            Error::Pin(_) => embedded_io_async::ErrorKind::Other,
        }
    }
}

/// Represents the module itself Uses a normal serial port +  a pin
/// to control wether the module is in read or write mode.
pub struct Max485<RIDO, REDE>
where
    RIDO: Read + Write,
    REDE: OutputPin,
{
    serial: RIDO,
    pin: REDE,
}

impl<RIDO, REDE> Max485<RIDO, REDE>
where
    RIDO: Read + Write,
    REDE: OutputPin,
{
    pub fn new(serial: RIDO, pin: REDE) -> Self {
        Self { serial, pin }
    }
    pub fn take_peripherals(self) -> (RIDO, REDE) {
        (self.serial, self.pin)
    }
    /// Provide a configuration function to be applied to the underlying serial port.
    pub fn reconfig_port<F>(&mut self, config: F)
    where
        F: Fn(&mut RIDO),
    {
        config(&mut self.serial);
    }
}

impl<RIDO, REDE> ErrorType for Max485<RIDO, REDE>
where
    RIDO: Read + Write,
    REDE: OutputPin,
{
    type Error = crate::Error<RIDO::Error, REDE::Error>;
}

impl<RIDO, REDE> Write for Max485<RIDO, REDE>
where
    RIDO: Read + Write,
    REDE: OutputPin,
{
    async fn write(&mut self, bytes: &[u8]) -> Result<usize, Self::Error> {
        self.pin.set_high().map_err(Error::Pin)?;

        let n = self.serial.write(bytes).await.map_err(Error::Serial)?;
        self.serial.flush().await.map_err(Error::Serial)?;

        self.pin.set_low().map_err(Error::Pin)?;

        Ok(n)
    }
}

impl<RIDO, REDE> Read for Max485<RIDO, REDE>
where
    RIDO: Read + Write,
    REDE: OutputPin,
{
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.pin.set_low().map_err(Error::Pin)?;
        self.serial.read(buf).await.map_err(Error::Serial)
    }
}
