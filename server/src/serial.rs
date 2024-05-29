pub fn write_serial(data: &[u8]) -> Result<(), String> {
    let ports = serialport::available_ports().map_err(|e| e.to_string())?;
    let port_info = ports
        .iter()
        .find(|port| matches!(port.port_type, serialport::SerialPortType::UsbPort(_)))
        .ok_or_else(|| "No USB serial device found".to_string())?;

    log::info!("Writing to USB serial port: {}", port_info.port_name);
    let mut port = serialport::new(&port_info.port_name, 9600)
        .timeout(std::time::Duration::from_millis(10))
        .open()
        .map_err(|e| e.to_string())?;
    port.write_all(data).map_err(|e| e.to_string())?;
    Ok(())
}
