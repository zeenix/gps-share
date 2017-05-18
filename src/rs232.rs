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
use serial::prelude::*;
use std::time::Duration;
use std::io;
use std::io::BufReader;
use std::io::BufRead;
use std::rc::Rc;

pub struct RS232 {
    reader: BufReader<serial::SystemPort>,
}

impl RS232 {
    pub fn new(config: Rc<Config>) -> Result<Self, serial::Error> {
        let mut port = serial::open(config.dev_path.as_str())?;
        port.reconfigure(& RS232::reconfigure)?;
        port.set_timeout(Duration::from_millis(1000))?;

        Ok(RS232 { reader: BufReader::new(port) })
    }

    fn reconfigure(settings: & mut serial::SerialPortSettings) -> serial::Result<()> {
        settings.set_baud_rate(serial::Baud38400)?; // FIXME: Need to be configurable
        settings.set_char_size(serial::Bits8);
        settings.set_parity(serial::ParityNone);
        settings.set_stop_bits(serial::Stop1);
        settings.set_flow_control(serial::FlowNone);

        Ok(())
    }
}

impl GPS for RS232 {
    fn read_line(& mut self, buffer: & mut String) -> io::Result<usize> {
        self.reader.read_line(buffer)
    }
}
