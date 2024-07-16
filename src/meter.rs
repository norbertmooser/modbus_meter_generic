use tokio_modbus::prelude::*;
use tokio_modbus::client::tcp;
use tokio_modbus::client::Context;
use std::net::SocketAddr;
use std::error::Error;
use config_meter_generic::config::{ConfigRegister, ConfigWriteRegister};


pub struct MeterGeneric {
    context: Option<Context>,
    read_registers: Vec<ConfigRegister>,
    write_registers: Vec<ConfigWriteRegister>,
}

impl MeterGeneric {
    pub fn new(
        read_registers: Vec<ConfigRegister>,
        write_registers: Vec<ConfigWriteRegister>,
    ) -> Self {
        MeterGeneric { context: None, read_registers, write_registers }
    }

    pub async fn connect(&mut self, ip: &str, port: u16) -> Result<(), Box<dyn Error>> {
        let socket_addr: SocketAddr = format!("{}:{}", ip, port).parse()?;
        let context = tcp::connect(socket_addr).await?;
        self.context = Some(context);
        Ok(())
    }

    pub async fn write(&mut self) -> Result<(), Box<dyn Error>> {
        if let Some(ref mut context) = self.context {
            for register in &self.write_registers {
                let value_bits = register.value.to_bits();
                let high = (value_bits >> 16) as u16;
                let low = value_bits as u16;
                let values = vec![high, low];

                match context.write_multiple_registers(register.address, &values).await {
                    Ok(_) => println!("Wrote value: {} to register: {}", register.value, register.name),
                    Err(e) => println!("Failed to write value: {} to register: {}: {}", register.value, register.name, e),
                }
            }
            Ok(())
        } else {
            Err("Not connected".into())
        }
    }

    pub async fn read(&mut self) -> Result<(), Box<dyn Error>> {
        if let Some(ref mut context) = self.context {
            if self.read_registers.is_empty() {
                return Err("No registers configured".into());
            }

            let start_address = self.read_registers.iter().map(|r| r.address).min().unwrap();
            let end_address = self.read_registers.iter().map(|r| r.address).max().unwrap();
            let quantity = (end_address - start_address + 2) as u16;

            let response: Result<Vec<u16>, Exception> = context.read_holding_registers(start_address, quantity).await?;
            let response_vec: Vec<u16> = response?;
            for register in &self.read_registers {
                let offset: usize = (register.address - start_address) as usize;
                if offset + 1 >= response_vec.len() {
                    return Err(format!("Index out of bounds for register: {}", register.name).into());
                }
                let high = response_vec[offset] as u32;
                let low = response_vec[offset + 1] as u32;
                let value = f32::from_bits((high << 16) | low);
                println!("Name: {}, Address: {}, Value: {}", register.name, register.address, value);
            }

            Ok(())
        } else {
            Err("Not connected".into())
        }
    }
}
