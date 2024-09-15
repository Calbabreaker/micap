use std::{borrow::Cow, io::Write, sync::Arc, time::Duration};

use byteorder::ReadBytesExt;
use futures_util::FutureExt;
use serialport::SerialPort;
use tokio::sync::RwLock;

#[cfg(unix)]
type NativePort = serialport::TTYPort;
#[cfg(windows)]
type NativePort = serialport::COMPort;

#[derive(Default)]
pub struct SerialPortManager {
    port: Arc<RwLock<Option<NativePort>>>,
    buffer: Vec<u8>,
}

impl SerialPortManager {
    pub fn start_scan_loop(&self) {
        let port_rw = self.port.clone();
        tokio::spawn(async move {
            check_ports(&port_rw).await;
            tokio::time::sleep(Duration::from_secs(2)).await;
        });
    }

    pub fn write(&mut self, data: &[u8]) -> anyhow::Result<()> {
        if let Some(mut port_lock) = self.port.write().now_or_never() {
            let port = port_lock
                .as_mut()
                .ok_or_else(|| anyhow::anyhow!("Port does not exist"))?;
            port.write_all(data)?;
        }
        Ok(())
    }

    pub fn read_line(&mut self) -> Option<&str> {
        let mut port_lock = self.port.write().now_or_never()?;
        let port = port_lock.as_mut()?;

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

    pub fn port_name(&self) -> Option<String> {
        self.port.read().now_or_never()?.as_ref()?.name()
    }
}

async fn check_ports(port_rw: &Arc<RwLock<Option<NativePort>>>) {
    let mut port_opt = port_rw.write().await;
    if let Some(port) = port_opt.as_ref() {
        // Disconnect port when can't read
        if port.bytes_to_read().is_err() {
            port_opt.take();
            log::info!("Serial port disconnected");
        }

        return;
    }

    *port_opt = tokio::task::block_in_place(find_usb_port);
}

fn find_usb_port() -> Option<NativePort> {
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
        .open_native()
        .ok()
}
