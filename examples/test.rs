// use tokio::io::{AsyncReadExt, AsyncWriteExt};
use anyhow::Result;
use tokio_modbus::{self};
use tokio_serial::SerialPortBuilderExt;
use waveshare::analog_in::AnalogInput;
use waveshare::digital::DigitalIO;
use waveshare::ThreadSafeContext;

pub struct Controller {
    pub context: ThreadSafeContext,
    pub analog: AnalogInput,
    pub digital: DigitalIO,
}

#[tokio::main]
async fn main() -> Result<()> {
    let serial = tokio_serial::new("/dev/ttyUSB1", 256000)
        .timeout(std::time::Duration::from_millis(100))
        .open_native_async()?;

    let ctx = tokio_modbus::client::rtu::attach_slave(serial, tokio_modbus::Slave(0x01));
    let ts_ctx = ThreadSafeContext::new(ctx);

    let analog = AnalogInput::new(1, ts_ctx.clone());
    let digital = DigitalIO::new(1, ts_ctx.clone());
    let mut controller = Controller {
        context: ts_ctx,
        analog,
        digital,
    };
    tokio::spawn(async move {
        let results = controller.analog.read_input_channels().await;
        println!("{:?}", results);
    });
    Ok(())
}
