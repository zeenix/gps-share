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

use crate::gps;
use std::io;
use std::io::Write;
use std::net::TcpStream;
use std::os::unix::net::UnixStream;
use std::sync::{Arc, Mutex};

pub enum Stream {
    Tcp(TcpStream),
    Unix(UnixStream),
}

impl Stream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            Stream::Tcp(s) => s.write(buf),
            Stream::Unix(s) => s.write(buf),
        }
    }
}

pub struct ClientHandler {
    gps: Arc<Mutex<dyn gps::GPS>>,
    streams: Arc<Mutex<Vec<Stream>>>,
}

impl ClientHandler {
    pub fn new(gps: Arc<Mutex<dyn gps::GPS>>, streams: Arc<Mutex<Vec<Stream>>>) -> Self {
        ClientHandler {
            gps: gps,
            streams: streams,
        }
    }

    pub fn handle(mut self) {
        let mut buffer = String::new();

        loop {
            // unwrap cause we don't want a poisoned lock:
            // https://doc.rust-lang.org/std/sync/struct.Mutex.html#poisoning
            if let Err(e) = self.gps.lock().unwrap().read_line(&mut buffer) {
                println!("Failed to read from serial port: {}", e);

                continue;
            }

            let to_delete = self.write_to_clients(&buffer);
            buffer.clear();

            // unwrap cause we don't want a poisoned lock:
            // https://doc.rust-lang.org/std/sync/struct.Mutex.html#poisoning
            let mut streams = self.streams.lock().unwrap();
            for i in to_delete.iter().rev() {
                streams.remove(*i);
            }

            if streams.len() == 0 {
                break;
            }
        }
    }

    fn write_to_clients(&mut self, buffer: &String) -> Vec<usize> {
        let mut to_delete: Vec<usize> = vec![];

        // unwrap cause we don't want a poisoned lock:
        // https://doc.rust-lang.org/std/sync/struct.Mutex.html#poisoning
        let mut streams = self.streams.lock().unwrap();
        for i in 0..streams.len() {
            let stream = &mut streams[i];

            match stream.write(buffer.as_bytes()) {
                Ok(0) => {
                    to_delete.push(i);

                    continue;
                }

                Ok(_) => {}

                Err(e) => {
                    println!("Failed to write NMEA to client: {}", e);
                    to_delete.push(i);

                    continue;
                }
            }
        }

        to_delete
    }
}
