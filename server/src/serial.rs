use std::borrow::Cow;

use byteorder::ReadBytesExt;

#[derive(Default)]
pub struct SerialPortManager {
    port: Option<Box<dyn serialport::SerialPort>>,
    buffer: Vec<u8>,
}

impl SerialPortManager {
    pub fn scan_ports(&mut self) -> anyhow::Result<()> {
        if self.port.is_some() {
            return Ok(());
        }

        // Find a USB port
        let ports = serialport::available_ports()?;
        let port_info = ports
            .iter()
            .find(|port| matches!(port.port_type, serialport::SerialPortType::UsbPort(_)))
            .ok_or_else(|| anyhow::anyhow!("USB device not found"))?;

        log::info!("Found serial port: {}", port_info.port_name);
        self.port = Some(
            serialport::new(&port_info.port_name, 14400)
                .timeout(std::time::Duration::from_millis(10))
                .open()?,
        );

        Ok(())
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

        self.buffer.clear();
        let mut ignore = false;

        // Read until new line
        // Using BufReader is unreliable for some reason
        while let Ok(byte) = port.read_u8() {
            if byte == b'\n' {
                break;
            } else if byte == b'[' {
                // Ingnore rest of bytes if log message (beginning with [)
                ignore = true;
            }

            if !ignore {
                self.buffer.push(byte);
            }
        }

        if self.buffer.is_empty() {
            return None;
        }

        // Can only be a borrow string
        let str = match String::from_utf8_lossy(&self.buffer) {
            Cow::Owned(_) => return None,
            Cow::Borrowed(b) => b,
        };

        Some(str)
    }

    pub fn read_status(&mut self) -> Option<&'static str> {
        let line = self.read_line()?;
        Some(match line {
            "WifiConnecting" => "Connecting to WiFi network",
            "WifiConnectOk" => "Connected to WiFi network",
            "WifiConnectTimeout" => "Failed to connect to WiFi network",
            "Connecting" => "Connecting to server",
            "Connected" => "Connected to server",
            _ => return None,
        })
    }
}
