use std::{borrow::Cow, io::Write, time::Duration};

use byteorder::ReadBytesExt;
use serialport::SerialPort;
use std::sync::mpsc::Receiver;

#[cfg(unix)]
type NativePort = serialport::TTYPort;
#[cfg(windows)]
type NativePort = serialport::COMPort;

pub struct SerialPortManager {
    /// Empty string means not connected
    port: Option<NativePort>,
    port_rx: Receiver<NativePort>,
    buffer: Vec<u8>,
}

impl Default for SerialPortManager {
    fn default() -> Self {
        let (port_tx, port_rx) = std::sync::mpsc::sync_channel(1);

        // Scanning ports blocks a bit so put it in a seperate task
        tokio::spawn(async move {
            loop {
                if let Some(port) = tokio::task::block_in_place(find_usb_port) {
                    port_tx.try_send(port).unwrap();
                }

                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        });

        Self {
            port_rx,
            port: None,
            buffer: Vec::new(),
        }
    }
}

impl SerialPortManager {
    pub fn write(&mut self, data: &[u8]) -> anyhow::Result<()> {
        let port = self
            .port
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("Serial port does not exist"))?;
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
                let str = match String::from_utf8_lossy(&self.buffer) {
                    Cow::Owned(_) => return None,
                    Cow::Borrowed(b) => b,
                };

                // Remove the new line character
                return Some(&str[0..str.len() - 1]);
            }
        }

        None
    }

    /// Returns true if the port state changed
    pub async fn check_port(&mut self) -> bool {
        if let Some(port) = self.port.as_ref() {
            // Disconnect port when can't read
            if port.bytes_to_read().is_err() {
                self.port.take();
                log::info!("Serial port disconnected");
                return true;
            }
        } else if let Ok(port) = self.port_rx.try_recv() {
            self.port = Some(port);
            return true;
        }

        false
    }

    pub fn port_name(&self) -> Option<Box<str>> {
        Some(Box::from(self.port.as_ref()?.name()?))
    }
}

fn find_usb_port() -> Option<NativePort> {
    // Find a USB port
    let ports = serialport::available_ports().ok()?;
    let port_info = ports
        .iter()
        .find(|port| matches!(port.port_type, serialport::SerialPortType::UsbPort(_)))?;

    let port = serialport::new(&port_info.port_name, 14400)
        .timeout(std::time::Duration::from_millis(5))
        .open_native();

    if port.is_ok() {
        log::info!(
            "Found serial port: {}\n{:?}",
            port_info.port_name,
            port_info.port_type
        );
    }

    port.ok()
}
