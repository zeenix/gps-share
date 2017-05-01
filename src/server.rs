/* vim: set et ts=4 sw=4: */
/* server.rs
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
use avahi;
use client_handler::ClientHandler;
use std::io;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

pub struct Server<G> {
    gps: Arc<Mutex<G>>,
    listener: TcpListener,
    avahi: Option<avahi::Avahi>,
}

impl<G: gps::GPS> Server<G> {
    pub fn new(gps: G, announce: bool) -> io::Result<Self> {
        let listener = TcpListener::bind(("0.0.0.0", 0))?;

        let avahi = if announce {
            Some(avahi::Avahi::new())
        } else {
            None
        };

        Ok(Server { gps:      Arc::new(Mutex::new(gps)),
                    listener: listener,
                    avahi:    avahi })
    }

    pub fn run(& mut self) -> io::Result<()> {
        let addr = self.listener.local_addr()?;
        let port = addr.port();
        println!("TCP server bound on all interfaces");
        println!("Port: {}", port);

        if let Some(ref avahi) = self.avahi {
            if let Err(e) = avahi.publish(port) {
                println!("Failed to publish service on Avahi: {}", e);
            };
        };

        let streams: Vec<TcpStream> = vec!();
        let streams_arc = Arc::new(Mutex::new(streams));

        loop {
            match self.listener.accept() {
                Ok((stream, addr)) => {
                    println!("Connection from {}", addr.ip());

                    let launch_handler;
                    {
                        // unwrap cause this shouldn't fail, should it?
                        let mut streams = streams_arc.lock().unwrap();
                        streams.push(stream);
                        launch_handler = streams.len() == 1;
                    }

                    if launch_handler {
                        let handler = ClientHandler::new(self.gps.clone(), streams_arc.clone());

                        thread::spawn(move || {
                            handler.handle();
                        });
                    }
                },

                Err(e) => {
                    println!("Connect from client failed: {}", e);
                }
            }
        }
    }
}
