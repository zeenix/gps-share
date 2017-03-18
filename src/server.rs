/* vim: set et ts=4 sw=4: */
/* gps.rs
 *
 * Copyright (C) 2017 Pelagicore AB.
 * Copyright (C) 2017 Zeeshan Ali.
 *
 * Geoclue is free software; you can redistribute it and/or modify it under
 * the terms of the GNU General Public License as published by the Free
 * Software Foundation; either version 2 of the License, or (at your option)
 * any later version.
 *
 * Geoclue is distributed in the hope that it will be useful, but WITHOUT ANY
 * WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
 * FOR A PARTICULAR PURPOSE.  See the GNU General Public License for more
 * details.
 *
 * You should have received a copy of the GNU General Public License along
 * with Geoclue; if not, write to the Free Software Foundation, Inc.,
 * 51 Franklin St, Fifth Floor, Boston, MA  02110-1301  USA
 *
 * Author: Zeeshan Ali <zeeshanak@gnome.org>
 */

use gps;
use std::io;
use std::net::{TcpListener, TcpStream};
use std::io::Write;

pub struct Server {
    gps: gps::GPS,
    listener: TcpListener,
}

impl Server {
    pub fn new(gps: gps::GPS) -> io::Result<Self> {
        let listener = TcpListener::bind(("0.0.0.0", 0))?;
        let addr = listener.local_addr()?;
        println!("TCP server bound to port {} on all interfaces", addr.port());

        Ok(Server { gps:      gps,
                    listener: listener })
    }

    pub fn run(& mut self) {
        loop {
            match self.listener.accept() {
                Ok((mut stream, addr)) => {
                    println!("Connection from {}", addr.ip());
                    self.handle_client(& mut stream);
                },

                Err(e) => {
                    println!("Connect from client failed: {}", e);
                }
            }
        }
    }

    fn handle_client(& mut self, stream: & mut TcpStream) {
        let mut buffer = String::new();

        loop {
            self.gps.read_line(& mut buffer).unwrap();

            match stream.write(buffer.as_bytes()) {
                Ok(0) => break,

                Ok(_) => {},

                Err(e) => {
                    println!("Error writing to client: {}", e);

                    break;
                }
            }
        }
    }
}
