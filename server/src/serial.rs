use std::borrow::Cow;

use byteorder::ReadBytesExt;

#[derive(Default)]
pub struct SerialPortManager {
    port: Option<Box<dyn serialport::SerialPort>>,
    buffer: Vec<u8>,
}

impl SerialPortManager {
    // Returns if the port was connected or disconnected this call
    pub fn scan_ports(&mut self) -> bool {
        if let Some(port) = self.port.as_ref() {
            if port.bytes_to_read().is_err() {
                // Disconnect port when can't read
                self.port.take();
                log::info!("Serial port disconnected");
                return true;
            }

            return false;
        }

        self.port = find_usb_port();
        self.port.is_some()
    }

    pub fn write(&mut self, data: &[u8]) -> anyhow::Result<()> {
        let port = self
            .port
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("Port does not exist"))?;
        port.write_all(data)?;

        Ok(())
    }

    pub fn read_line(&mut self) -> Option<&str> {
        let port = self.port.as_mut()?;

        if port.bytes_to_read().ok()? == 0 {
            return None;
        }

        if self.buffer.last() == Some(&b'\n') {
            self.buffer.clear();
        }

        // Only return string when new line is reached to prevent cut off messages
        while let Ok(byte) = port.read_u8() {
            self.buffer.push(byte);

            if byte == b'\n' {
                // Can only be a borrowed string
                return Some(match String::from_utf8_lossy(&self.buffer) {
                    Cow::Owned(_) => return None,
                    Cow::Borrowed(b) => b,
                });
            }
        }

        None
    }

    pub fn port_name(&self) -> Option<String> {
        self.port.as_ref()?.name()
    }
}

pub fn find_usb_port() -> Option<Box<dyn serialport::SerialPort>> {
    // Find a USB port
    let ports = serialport::available_ports().ok()?;
    let port_info = ports
        .iter()
        .find(|port| matches!(port.port_type, serialport::SerialPortType::UsbPort(_)))?;

    log::info!(
        "Found serial port: {}\n{:?}",
        port_info.port_name,
        port_info.port_type
    );
    serialport::new(&port_info.port_name, 14400)
        .timeout(std::time::Duration::from_millis(5))
        .open()
        .ok()
}
