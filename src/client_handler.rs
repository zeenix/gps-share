/* vim: set et ts=4 sw=4: */
/* client_handler.rs
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

use gps;
use std::net::{TcpStream};
use std::io::Write;
use std::sync::{Arc, Mutex};

pub struct ClientHandler {
    gps_arc: Arc<Mutex<gps::GPS>>,
    streams_arc: Arc<Mutex<Vec<TcpStream>>>,
}

impl ClientHandler {
    pub fn new(gps_arc:     Arc<Mutex<gps::GPS>>,
               streams_arc: Arc<Mutex<Vec<TcpStream>>>) -> Self {
        ClientHandler { gps_arc:      gps_arc,
                        streams_arc:  streams_arc }
    }

    pub fn handle(mut self) {
        let mut buffer = String::new();

        loop {
            self.gps_arc.lock().unwrap().read_line(& mut buffer).unwrap();

            let to_delete = self.write_to_clients(& buffer);

            let mut streams = self.streams_arc.lock().unwrap();
            for i in to_delete.iter().rev() {
                streams.remove(*i);
            }

            if streams.len() == 0 {
                break;
            }
        }
    }

    fn write_to_clients(& mut self, buffer: & String) -> Vec<usize> {
        let mut to_delete: Vec<usize> = vec!();

        let streams = self.streams_arc.lock().unwrap();
        for i in 0..streams.len() {
            let mut stream = &streams[i];

            match stream.write(buffer.as_bytes()) {
                Ok(0) => {
                    to_delete.push(i);

                    continue;
                },

                Ok(_) => {},

                Err(e) => {
                    to_delete.push(i);

                    continue;
                }
            }
        }

        to_delete
    }
}
