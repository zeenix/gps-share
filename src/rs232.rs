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
use serialport;
use serialport::posix::TTYPort;
use serialport::prelude::*;
use std::time::Duration;
use std::io;
use std::io::BufReader;
use std::io::BufRead;
use std::rc::Rc;

pub struct RS232 {
    reader: BufReader<TTYPort>,
}

impl RS232 {
    pub fn new(config: Rc<Config>) -> Result<Self, serialport::Error> {
        let baudrate = config.get_baudrate();
        let settings = serialport::SerialPortSettings { baud_rate: baudrate,
                                                        data_bits: DataBits::Eight,
                                                        parity: Parity::None,
                                                        stop_bits: StopBits::One,
                                                        flow_control: FlowControl::None,
                                                        timeout: Duration::from_millis(1000), };
        let port = TTYPort::open(config.dev_path.as_path(), &settings)?;

        Ok(RS232 { reader: BufReader::new(port) })
    }
}

impl GPS for RS232 {
    fn read_line(& mut self, buffer: & mut String) -> io::Result<usize> {
        self.reader.read_line(buffer)
    }
}
