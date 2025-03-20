use crate::{
    common::{Baudrates, Channel, CommonHoldingRegisters, Parity, WaveshareModbus},
    ThreadSafeContext,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AnalogOutputError {
    #[error("Modbus Exception Error: `{0}`")]
    ModbusException(tokio_modbus::ExceptionCode),
    #[error("Modbus Error: `{0}`")]
    ModbusError(tokio_modbus::Error),
    #[error("Invalid Control Mode")]
    InvalidControlMode,
}

#[derive(Debug)]
pub struct AnalogOutput {
    pub unit_id: u8,
    pub context: ThreadSafeContext,
}

#[derive(Debug, Copy, Clone)]
#[repr(u16)]
pub enum HoldingRegisterBases {
    // These are specific to the underlying hardwares jumper configuration
    AnalogValue = 0x0000,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u16)]
pub enum ControlMode {
    V0V10 = 0x0000, // 0~10V, output range: 0~5000mV;
    V2V10 = 0x0001, // 2~10V, output range: 1000~5000mV;
    C0C20 = 0x0002, // 0~20mA, output range: 0~20000uA;
    C4C20 = 0x0003, // 4~20mA, output range: 4000~20000uA;
    RAW = 0x0004, // directly output the value code, output range: 0~4096, the linear transformation is required to obtain the actual measured voltage and current.
}

impl ControlMode {
    pub fn from_u16(value: u16) -> Result<ControlMode, AnalogOutputError> {
        match value {
            0x0000 => Ok(ControlMode::V0V10),
            0x0001 => Ok(ControlMode::V2V10),
            0x0002 => Ok(ControlMode::C0C20),
            0x0003 => Ok(ControlMode::C4C20),
            0x0004 => Ok(ControlMode::RAW),
            _ => Err(AnalogOutputError::InvalidControlMode),
        }
    }
}

impl AnalogOutput {
    pub fn new(unit_id: u8, context: ThreadSafeContext) -> Self {
        AnalogOutput { unit_id, context }
    }

    pub async fn set_slave_id(&mut self) {
        self.context
            .set_slave(tokio_modbus::Slave(self.unit_id))
            .await;
    }

    pub async fn read_output_channel_value(
        &mut self,
        channel: Channel,
    ) -> Result<u16, AnalogOutputError> {
        self.set_slave_id().await;
        let result = self
            .context
            .read_holding_registers(channel as u16 + HoldingRegisterBases::AnalogValue as u16, 1)
            .await
            .map_err(|err| AnalogOutputError::ModbusError(err))?
            .map_err(|err| AnalogOutputError::ModbusException(err))?;
        Ok(result[0])
    }

    pub async fn write_output_channel_value(
        &mut self,
        channel: Channel,
        value: u16,
    ) -> Result<(), AnalogOutputError> {
        self.set_slave_id().await;
        self.context
            .write_single_register(
                channel as u16 + HoldingRegisterBases::AnalogValue as u16,
                value,
            )
            .await
            .map_err(|err| AnalogOutputError::ModbusError(err))?
            .map_err(|err| AnalogOutputError::ModbusException(err))?;
        Ok(())
    }
}

impl WaveshareModbus for AnalogOutput {
    type Error = AnalogOutputError;

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
            .map_err(|err| AnalogOutputError::ModbusError(err))?
            .map_err(|err| AnalogOutputError::ModbusException(err))?;
        Ok(result)
    }

    async fn set_device_address(&mut self, address: u8) -> Result<(), Self::Error> {
        self.set_slave_id().await;
        let result = self
            .context
            .write_single_register(CommonHoldingRegisters::DeviceAddress as u16, address as u16)
            .await
            .map_err(|err| AnalogOutputError::ModbusError(err))?
            .map_err(|err| AnalogOutputError::ModbusException(err))?;
        Ok(result)
    }

    async fn read_software_version(&mut self) -> Result<u16, Self::Error> {
        self.set_slave_id().await;
        let result = self
            .context
            .read_holding_registers(CommonHoldingRegisters::SoftwareVersion as u16, 1)
            .await
            .map_err(|err| AnalogOutputError::ModbusError(err))?
            .map_err(|err| AnalogOutputError::ModbusException(err))?;
        Ok(result[0])
    }
}
