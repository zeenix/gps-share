/* vim: set et ts=4 sw=4: */
/* stdin_gps.rs
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

use crate::gps::GPS;
use std::io;

pub struct StdinGPS {
    stdin: io::Stdin,
}

impl StdinGPS {
    pub fn new() -> Self {
        StdinGPS { stdin: io::stdin() }
    }
}

impl GPS for StdinGPS {
    fn read_line(&mut self, buffer: &mut String) -> io::Result<usize> {
        self.stdin.read_line(buffer)
    }
}
