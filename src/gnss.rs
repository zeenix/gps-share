/* vim: set et ts=4 sw=4: */
/* gnss.rs
 *
 * Copyright (C) 2017 Pelagicore AB.
 * Copyright (C) 2017 Zeeshan Ali.
 * Copyright (C) 2020 Purism SPC.
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

use config::Config;
use gps::GPS;
use libudev;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;
use std::rc::Rc;
use std::fs::File;
use std::fs;
use std::io;

pub struct GNSS {
    reader: BufReader<fs::File>,
}

impl GNSS {
    pub fn new(config: Rc<Config>) -> io::Result<Self> {
        match config.dev_path {
            Some(ref path) => GNSS::new_for_path(path.as_path()),
            None => GNSS::new_detect(),
        }
    }

    fn new_for_path(path: &Path) -> io::Result<Self> {
        let port = File::open(path.as_os_str())?;

        Ok(GNSS {
            reader: BufReader::new(port),
        })
    }

    fn new_detect() -> io::Result<Self> {
        println!("Attempting to autodetect GNSS device...");
        let context = libudev::Context::new()?;
        let mut enumerator = libudev::Enumerator::new(&context)?;
        enumerator.match_subsystem("gnss")?;
        let devices = enumerator.scan_devices()?;
        for d in devices {
            if let Some(p) = d.devnode().and_then(|devnode| devnode.to_str()) {
                let path = Path::new(p);

                match GNSS::new_for_path(&path) {
                    Ok(mut gps) => {
                        if gps.verify() {
                            println!("Detected {} as a GPS device", p);

                            return Ok(gps);
                        }
                    }

                    Err(e) => println!("Error openning {}: {}", p, e),
                }
            }
        }

        Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Failed to autodetect GNSS device",
        ))
    }

    fn verify(&mut self) -> bool {
        let mut buffer = String::new();

        for _ in 1..3 {
            if let Ok(_) = self.read_line(&mut buffer) {
                if buffer.len() >= 15
                    && buffer.chars().nth(0) == Some('$')
                    && buffer.chars().nth(6) == Some(',')
                {
                    return true;
                }

                buffer.clear();
            } else {
                println!("Failed to read from serial port");
            }
        }

        false
    }
}

impl GPS for GNSS {
    fn read_line(&mut self, buffer: &mut String) -> io::Result<usize> {
        self.reader.read_line(buffer)
    }
}
