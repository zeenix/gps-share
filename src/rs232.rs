/* vim: set et ts=4 sw=4: */
/* rs232.rs
 *
 * Copyright (C) 2017 Pelagicore AB.
 * Copyright (C) 2017 Zeeshan Ali.
 *
 * GPSShare is free software; you can redistribute it and/or modify it under
 * the terms of the GNU General Public License as published by the Free
 * Software Foundation; either version 2 of the License, or (at your option)
 * any later version.
 *
 * GPSShare is distributed in the hope that it will be useful, but WITHOUT ANY
 * WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
 * FOR A PARTICULAR PURPOSE.  See the GNU General Public License for more
 * details.
 *
 * You should have received a copy of the GNU General Public License along
 * with GPSShare; if not, write to the Free Software Foundation, Inc.,
 * 51 Franklin St, Fifth Floor, Boston, MA  02110-1301  USA
 *
 * Author: Zeeshan Ali <zeeshanak@gnome.org>
 */

use gps::GPS;
use config::Config;
use serial;
use libudev;
use std::time::Duration;
use std::io;
use std::io::BufReader;
use std::io::BufRead;
use std::rc::Rc;
use std::path::Path;

pub struct RS232 {
    reader: BufReader<serial::SystemPort>,
}

impl RS232 {
    pub fn new(config: Rc<Config>) -> io::Result<Self> {
        match config.dev_path {
            Some(ref path) => RS232::new_for_path(path.as_path(), &config),
            None => RS232::new_detect(&config),
        }
    }

    fn new_for_path(path: &Path, config: &Config) -> io::Result<Self> {
        let mut port = serial::open(path.as_os_str())?;
        RS232::configure(&mut port as &mut serial::SerialPort, config)?;

        Ok(RS232 { reader: BufReader::new(port) })
    }

    fn configure(port: &mut serial::SerialPort, config: &Config) -> serial::Result<()> {
        let baudrate = config.get_baudrate();
        let settings = serial::PortSettings {
            baud_rate: baudrate,
            char_size: serial::Bits8,
            parity: serial::ParityNone,
            stop_bits: serial::Stop1,
            flow_control: serial::FlowNone,
        };

        port.configure(&settings)?;

        port.set_timeout(Duration::from_millis(3_000))?;

        Ok(())
    }

    fn new_detect(config: &Config) -> io::Result<Self> {
        println!("Attempting to autodetect GPS device...");
        let context = libudev::Context::new()?;
        let mut enumerator = libudev::Enumerator::new(&context)?;
        enumerator.match_subsystem("tty")?;
        enumerator.match_property("ID_BUS", "usb")?;
        let devices = enumerator.scan_devices()?;
        for d in devices {
            let path = d.devnode().unwrap_or(Path::new("UNKNOWN")).to_str().unwrap();
            if let Some(driver) = d.parent().as_ref().and_then(|p| p.driver()) {
                if driver != "pl2303" && driver != "cdc_acm" {
                    continue;
                }
            }

            println!("{} seems interesting", path);
            if let Some(p) = d.devnode().and_then(|devnode| devnode.to_str()) {
                let path = Path::new(p);

                match RS232::new_for_path(&path, config) {
                    Ok(mut gps) => {
                        println!("Needs verification");
                        if gps.verify() {
                            println!("Detected {} as a GPS device", p);

                            return Ok(gps);
                        } else {
                            println!("Not verified");
                        }
                    }

                    Err(e) => println!("Error openning {}: {}", p, e),
                }
            }
        }

        Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Failed to autodetect GPS device",
        ))
    }

    fn verify(&mut self) -> bool {
        let mut buffer = String::new();

        for _ in 1..3 {
            println!("Reading from port..");
            if let Ok(_) = self.read_line(&mut buffer) {
                println!("Read from port: {}", buffer);
                if buffer.len() >= 15 && buffer.starts_with("$G") &&
                    buffer.chars().nth(6) == Some(',')
                {
                    return true;
                } else {
                    println!("Read from port: {}", buffer);
                }

                buffer.clear();
            } else {
                println!("Failed to read from serial port");
            }
        }

        false
    }
}

impl GPS for RS232 {
    fn read_line(&mut self, buffer: &mut String) -> io::Result<usize> {
        self.reader.read_line(buffer)
    }
}
