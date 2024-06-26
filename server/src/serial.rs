pub fn write_serial(data: &[u8]) -> anyhow::Result<()> {
    let ports = serialport::available_ports()?;
    let port_info = ports
        .iter()
        .find(|port| matches!(port.port_type, serialport::SerialPortType::UsbPort(_)))
        .ok_or_else(|| anyhow::anyhow!("USB device not found"))?;

    log::info!("Writing to USB serial port: {}", port_info.port_name);
    let mut port = serialport::new(&port_info.port_name, 9600)
        .timeout(std::time::Duration::from_millis(10))
        .open()?;
    port.write_all(data)?;
    Ok(())
}
