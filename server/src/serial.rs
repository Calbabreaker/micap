use std::io::{BufRead, BufReader};

#[derive(Default)]
pub struct SerialPortManager {
    port: Option<Box<dyn serialport::SerialPort>>,
}

impl SerialPortManager {
    pub fn scan_ports(&mut self) -> anyhow::Result<()> {
        let ports = serialport::available_ports()?;
        let port_info = ports
            .iter()
            .find(|port| matches!(port.port_type, serialport::SerialPortType::UsbPort(_)))
            .ok_or_else(|| anyhow::anyhow!("USB device not found"))?;

        log::info!("Found serial port: {}", port_info.port_name);
        self.port = Some(
            serialport::new(&port_info.port_name, 9600)
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

    pub fn read_line(&mut self) -> Option<String> {
        if let Some(port) = &mut self.port {
            if port.bytes_to_read().unwrap_or(0) > 0 {
                let mut reader = BufReader::new(port);
                let mut buf = String::new();
                reader.read_line(&mut buf).ok()?;
                return Some(buf);
            }
        }

        None
    }

    pub fn read_status(&mut self) -> Option<&'static str> {
        let line = self.read_line()?;
        Some(match line.as_str() {
            "WifiConnecting" => "Connecting to WiFi network",
            "WifiConnectOk" => "Connected to WiFi network",
            "WifiConnectTimeout" => "Failed to connect to WiFi network",
            "Connecting" => "Connecting to server",
            "Connected" => "Connected to server",
            _ => return None,
        })
    }
}
