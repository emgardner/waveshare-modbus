#[derive(Debug, Copy, Clone)]
#[repr(u16)]
pub enum CommonHoldingRegisters {
    UartParameters = 0x2000,
    DeviceAddress = 0x4000,
    SoftwareVersion = 0x8000,
}

#[derive(Debug, Copy, Clone)]
#[repr(u16)]
pub enum Baudrates {
    B4800 = 0x00,
    B9600 = 0x01,
    B19200 = 0x02,
    B38400 = 0x03,
    B57600 = 0x04,
    B115200 = 0x05,
    B128000 = 0x06,
    B256000 = 0x07,
}

#[derive(Debug, Copy, Clone)]
#[repr(u16)]
pub enum Parity {
    None = 0x00,
    Even = 0x01,
    Odd = 0x02,
}

#[derive(Debug, Copy, Clone)]
#[repr(u16)]
pub enum Channel {
    Channel0 = 0x0000,
    Channel1 = 0x0001,
    Channel2 = 0x0002,
    Channel3 = 0x0003,
    Channel4 = 0x0004,
    Channel5 = 0x0005,
    Channel6 = 0x0006,
    Channel7 = 0x0007,
}

impl TryFrom<u8> for Channel {
    type Error = &'static str;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Channel::Channel0),
            1 => Ok(Channel::Channel1),
            2 => Ok(Channel::Channel2),
            3 => Ok(Channel::Channel3),
            4 => Ok(Channel::Channel4),
            5 => Ok(Channel::Channel5),
            6 => Ok(Channel::Channel6),
            7 => Ok(Channel::Channel7),
            _ => Err("Invalid Channel"),
        }
    }
}

pub trait WaveshareModbus {
    type Error;
    #[allow(async_fn_in_trait)]
    async fn set_uart_parameters(
        &mut self,
        baudrate: Baudrates,
        parity: Parity,
    ) -> Result<(), Self::Error>;
    #[allow(async_fn_in_trait)]
    async fn set_device_address(&mut self, address: u8) -> Result<(), Self::Error>;
    #[allow(async_fn_in_trait)]
    async fn read_software_version(&mut self) -> Result<u16, Self::Error>;
}
/*
macro_rules! impl_waveshare_modbus {
    ($struct_name:ident, $error_type:ty) => {
        impl WaveshareModbus for $struct_name {
            type Error = $error_type;

            async fn set_uart_parameters(
                &mut self,
                baud: Baudrates,
                parity: Parity,
            ) -> Result<(), Self::Error> {
                self.set_slave_id().await;
                let value = ((parity as u16) << 8) | (baud as u16);
                let result = self
                    .context
                    .write_single_register(CommonHoldingRegisters::UartParameters as u16, value)
                    .await
                    .map_err(|err| <$error_type>::ModbusError(err))?
                    .map_err(|err| <$error_type>::ModbusException(err))?;
                Ok(result)
            }

            async fn set_device_address(&mut self, address: u8) -> Result<(), Self::Error> {
                self.set_slave_id().await;
                let result = self
                    .context
                    .write_single_register(
                        CommonHoldingRegisters::DeviceAddress as u16,
                        address as u16,
                    )
                    .await
                    .map_err(|err| <$error_type>::ModbusError(err))?
                    .map_err(|err| <$error_type>::ModbusException(err))?;
                Ok(result)
            }

            async fn read_software_version(&mut self) -> Result<u16, Self::Error> {
                self.set_slave_id().await;
                let result = self
                    .context
                    .read_holding_registers(CommonHoldingRegisters::SoftwareVersion as u16, 1)
                    .await
                    .map_err(|err| <$error_type>::ModbusError(err))?
                    .map_err(|err| <$error_type>::ModbusException(err))?;
                Ok(result[0])
            }
        }
    };
}
*/
