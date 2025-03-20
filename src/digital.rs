use crate::{
    common::{Baudrates, Channel, CommonHoldingRegisters, Parity, WaveshareModbus},
    ThreadSafeContext,
};
use thiserror::Error;

#[derive(Debug)]
pub struct DigitalIO {
    pub unit_id: u8,
    pub context: ThreadSafeContext,
}

#[derive(Error, Debug)]
pub enum DigitalIOError {
    #[error("Modbus Exception Error: `{0}`")]
    ModbusException(tokio_modbus::ExceptionCode),
    #[error("Modbus Error: `{0}`")]
    ModbusError(tokio_modbus::Error),
    #[error("Invalid Control Mode")]
    InvalidControlMode,
}

#[derive(Debug, Copy, Clone)]
pub struct IoBank {
    pub ch0: bool,
    pub ch1: bool,
    pub ch2: bool,
    pub ch3: bool,
    pub ch4: bool,
    pub ch5: bool,
    pub ch6: bool,
    pub ch7: bool,
}

impl IoBank {
    pub fn as_action_array(&self) -> [Action; 8] {
        [
            if self.ch0 { Action::On } else { Action::Off },
            if self.ch1 { Action::On } else { Action::Off },
            if self.ch2 { Action::On } else { Action::Off },
            if self.ch3 { Action::On } else { Action::Off },
            if self.ch4 { Action::On } else { Action::Off },
            if self.ch5 { Action::On } else { Action::Off },
            if self.ch6 { Action::On } else { Action::Off },
            if self.ch7 { Action::On } else { Action::Off },
        ]
    }
}

impl From<u8> for IoBank {
    fn from(data: u8) -> Self {
        Self {
            ch0: data & 0x01 > 0,
            ch1: data & 0x02 > 0,
            ch2: data & 0x04 > 0,
            ch3: data & 0x08 > 0,
            ch4: data & 0x10 > 0,
            ch5: data & 0x20 > 0,
            ch6: data & 0x40 > 0,
            ch7: data & 0x80 > 0,
        }
    }
}

impl Into<u8> for IoBank {
    fn into(self) -> u8 {
        let mut out = 0u8;
        if self.ch0 {
            out |= 0x01
        };
        if self.ch1 {
            out |= 0x02
        };
        if self.ch2 {
            out |= 0x04
        };
        if self.ch3 {
            out |= 0x08
        };
        if self.ch4 {
            out |= 0x10
        };
        if self.ch5 {
            out |= 0x20
        };
        if self.ch6 {
            out |= 0x40
        };
        if self.ch7 {
            out |= 0x80
        };
        out
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(u16)]
pub enum OutputRegisterBases {
    OutputChannel = 0x0000,
    ControlAllRegisters = 0x00FF,
    OutputChannelFlashOn = 0x0200,
    OutputChannelFlashOff = 0x0400,
}

#[derive(Debug, Copy, Clone)]
#[repr(u16)]
pub enum InputRegisterBases {
    InputChannels = 0x0000,
}

#[derive(Debug, Copy, Clone)]
#[repr(u16)]
pub enum HoldingRegisterBases {
    ControlMode = 0x1000,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u16)]
pub enum Action {
    On = 0xFF00,
    Off = 0x0000,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u16)]
pub enum ControlMode {
    Command = 0x0000,
    Linked = 0x0001,
    Flip = 0x0002,
}

impl DigitalIO {
    pub fn new(unit_id: u8, context: ThreadSafeContext) -> Self {
        DigitalIO { unit_id, context }
    }

    pub async fn set_slave_id(&mut self) {
        self.context
            .set_slave(tokio_modbus::Slave(self.unit_id))
            .await;
    }

    pub async fn write_output_channel(
        &mut self,
        channel: Channel,
        action: Action,
    ) -> Result<(), DigitalIOError> {
        self.set_slave_id().await;
        self.context
            .write_single_coil(
                channel as u16 + OutputRegisterBases::OutputChannel as u16,
                if action == Action::On { true } else { false },
            )
            .await
            .map_err(|err| DigitalIOError::ModbusError(err))?
            .map_err(|err| DigitalIOError::ModbusException(err))?;
        Ok(())
    }

    pub async fn open_all_outputs(&mut self) -> Result<(), DigitalIOError> {
        self.set_slave_id().await;
        self.context
            .write_single_coil(OutputRegisterBases::ControlAllRegisters as u16, true)
            .await
            .map_err(DigitalIOError::ModbusError)?
            .map_err(DigitalIOError::ModbusException)?;
        Ok(())
    }

    pub async fn close_all_outputs(&mut self) -> Result<(), DigitalIOError> {
        self.set_slave_id().await;
        self.context
            .write_single_coil(OutputRegisterBases::ControlAllRegisters as u16, false)
            .await
            .map_err(DigitalIOError::ModbusError)?
            .map_err(DigitalIOError::ModbusException)?;
        Ok(())
    }

    pub async fn write_output_channels(
        &mut self,
        actions: [Action; 8],
    ) -> Result<(), DigitalIOError> {
        let values = actions.map(|x| x == Action::On);
        self.set_slave_id().await;
        self.context
            .write_multiple_coils(OutputRegisterBases::OutputChannel as u16, &values)
            .await
            .map_err(DigitalIOError::ModbusError)?
            .map_err(DigitalIOError::ModbusException)?;
        Ok(())
    }

    pub async fn flash_output_on(
        &mut self,
        channel: Channel,
        interval: u16,
    ) -> Result<(), DigitalIOError> {
        self.set_slave_id().await;
        self.context
            .write_single_register(
                channel as u16 + OutputRegisterBases::OutputChannelFlashOn as u16,
                interval,
            )
            .await
            .map_err(DigitalIOError::ModbusError)?
            .map_err(DigitalIOError::ModbusException)?;
        Ok(())
    }

    pub async fn flash_output_off(
        &mut self,
        channel: Channel,
        interval: u16,
    ) -> Result<(), DigitalIOError> {
        self.set_slave_id().await;
        self.context
            .write_single_register(
                channel as u16 + OutputRegisterBases::OutputChannelFlashOff as u16,
                interval,
            )
            .await
            .map_err(DigitalIOError::ModbusError)?
            .map_err(DigitalIOError::ModbusException)?;
        Ok(())
    }

    pub async fn read_input_channel_status(
        &mut self,
        channel: Channel,
    ) -> Result<bool, DigitalIOError> {
        self.set_slave_id().await;
        let result = self
            .context
            .read_discrete_inputs(channel as u16 + InputRegisterBases::InputChannels as u16, 1)
            .await
            .map_err(DigitalIOError::ModbusError)?
            .map_err(DigitalIOError::ModbusException)?;

        Ok(result.first().copied().unwrap_or(false))
    }

    pub async fn read_input_channels(&mut self) -> Result<Vec<bool>, DigitalIOError> {
        self.set_slave_id().await;
        self.context
            .read_discrete_inputs(InputRegisterBases::InputChannels as u16, 8)
            .await
            .map_err(DigitalIOError::ModbusError)?
            .map_err(DigitalIOError::ModbusException)
    }

    pub async fn set_output_control_mode(
        &mut self,
        channel: Channel,
        mode: ControlMode,
    ) -> Result<(), DigitalIOError> {
        self.set_slave_id().await;
        self.context
            .write_single_register(
                HoldingRegisterBases::ControlMode as u16 + channel as u16,
                mode as u16,
            )
            .await
            .map_err(DigitalIOError::ModbusError)?
            .map_err(DigitalIOError::ModbusException)?;
        Ok(())
    }

    /*
    pub fn write_output_channel(&self, channel: Channel, action: Action) -> RequestAdu {
        self.create_request_adu(Request::WriteSingleCoil(
            channel as u16 + OutputRegisterBases::OutputChannel as u16,
            if action == Action::On { true } else { false },
        ))
    }

    pub fn open_all_outputs(&self) -> RequestAdu {
        self.create_request_adu(Request::WriteSingleCoil(
            OutputRegisterBases::ControlAllRegisters as u16,
            true,
        ))
    }

    pub fn close_all_outputs(&self) -> RequestAdu {
        self.create_request_adu(Request::WriteSingleCoil(
            OutputRegisterBases::ControlAllRegisters as u16,
            false,
        ))
    }

    pub fn write_output_channels<'a>(
        &self,
        actions: [Action; 8],
        coils: &'a mut [u8],
    ) -> RequestAdu<'a> {
        let values = actions.map(|x| x == Action::On);
        let coils = Coils::from_bools(&values, coils).expect("There's no reason this should fail");
        self.create_request_adu(Request::WriteMultipleCoils(
            OutputRegisterBases::OutputChannel as u16,
            coils,
        ))
    }

    pub fn flash_output_on(&self, channel: Channel, interval: u16) -> RequestAdu {
        self.create_request_adu(Request::WriteSingleRegister(
            channel as u16 + OutputRegisterBases::OutputChannelFlashOn as u16,
            interval,
        ))
    }

    pub fn flash_output_off(&self, channel: Channel, interval: u16) -> RequestAdu {
        self.create_request_adu(Request::WriteSingleRegister(
            channel as u16 + OutputRegisterBases::OutputChannelFlashOff as u16,
            interval,
        ))
    }

    pub fn read_input_channel_status(&self, channel: Channel) -> RequestAdu {
        self.create_request_adu(Request::ReadDiscreteInputs(
            channel as u16 + InputRegisterBases::InputChannels as u16,
            1,
        ))
    }

    pub fn read_input_channels(&self) -> RequestAdu {
        self.create_request_adu(Request::ReadDiscreteInputs(
            InputRegisterBases::InputChannels as u16,
            8,
        ))
    }

    pub fn set_output_control_mode(&self, channel: Channel, mode: ControlMode) -> RequestAdu {
        self.create_request_adu(Request::WriteSingleRegister(
            HoldingRegisterBases::ControlMode as u16 + channel as u16,
            mode as u16,
        ))
    }
    */
}

impl WaveshareModbus for DigitalIO {
    type Error = DigitalIOError;

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
            .map_err(|err| DigitalIOError::ModbusError(err))?
            .map_err(|err| DigitalIOError::ModbusException(err))?;
        Ok(result)
    }

    async fn set_device_address(&mut self, address: u8) -> Result<(), Self::Error> {
        self.set_slave_id().await;
        let result = self
            .context
            .write_single_register(CommonHoldingRegisters::DeviceAddress as u16, address as u16)
            .await
            .map_err(|err| DigitalIOError::ModbusError(err))?
            .map_err(|err| DigitalIOError::ModbusException(err))?;
        Ok(result)
    }

    async fn read_software_version(&mut self) -> Result<u16, Self::Error> {
        self.set_slave_id().await;
        let result = self
            .context
            .read_holding_registers(CommonHoldingRegisters::SoftwareVersion as u16, 1)
            .await
            .map_err(|err| DigitalIOError::ModbusError(err))?
            .map_err(|err| DigitalIOError::ModbusException(err))?;
        Ok(result[0])
    }
}
