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

use avahi;
use client_handler::{ClientHandler, Stream};
use config::Config;
use gps;
use std::io;
use std::net::{TcpListener};
use std::os::unix::net::{UnixListener};
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::thread;

pub struct Server {
    gps: Arc<Mutex<dyn gps::GPS>>,
    tcp_listener: TcpListener,
    unix_listener: Option<Arc<Mutex<UnixListener>>>,
    avahi: Option<avahi::Avahi>,
    config: Rc<Config>,
}

impl Server {
    pub fn new<T: gps::GPS>(gps: T, config: Rc<Config>) -> io::Result<Self> {
        let ip = config.get_ip();
        let tcp_listener = TcpListener::bind((ip.as_str(), config.port))?;

        let path = &config.socket_path;
        let unix_listener = match path {
            Some(p) => Some(Arc::new(Mutex::new(UnixListener::bind(p)?))),
            None => None,
        };

        let avahi = if config.announce_on_net {
            match avahi::Avahi::new() {
                Ok(avahi) => Some(avahi),

                Err(e) => {
                    println!("Failed to connect to Avahi: {}", e);

                    None
                }
            }
        } else {
            None
        };

        Ok(Server {
            gps: Arc::new(Mutex::new(gps)),
            tcp_listener: tcp_listener,
            unix_listener: unix_listener,
            avahi: avahi,
            config: config,
        })
    }

    pub fn run(&mut self) -> io::Result<()> {
        let addr = self.tcp_listener.local_addr()?;
        let port = addr.port();
        let config = &self.config;
        match config.net_iface {
            Some(ref i) => println!("TCP server bound on {} interface", i),
            None => println!("TCP server bound on all interfaces"),
        };
        println!("Port: {}", port);

        if let Some(ref avahi) = self.avahi {
            let iface = config.net_iface.as_ref().map(|i| i.as_str());

            if let Err(e) = avahi.publish(iface, port) {
                println!("Failed to publish service on Avahi: {}", e);
            };
        };

        let streams: Vec<Stream> = vec![];
        let streams_arc = Arc::new(Mutex::new(streams));

        if let Some(listener) = &self.unix_listener {
            let listener = listener.clone();
            let streams_arc = streams_arc.clone();
            let gps = self.gps.clone();
            thread::spawn(move || {
                let listener = listener.lock().unwrap();
                loop {
                    match listener.accept() {
                        Ok((stream, _addr)) => {
                            let launch_handler;
                            {
                                // unwrap cause we don't want a poisoned lock:
                                // https://doc.rust-lang.org/std/sync/struct.Mutex.html#poisoning
                                let mut streams = streams_arc.lock().unwrap();
                                streams.push(Stream::Unix(stream));
                                launch_handler = streams.len() == 1;
                            }

                            if launch_handler {
                                let handler = ClientHandler::new(gps.clone(), streams_arc.clone());

                                thread::spawn(move || {
                                    handler.handle();
                                });
                            }
                        },
                        Err(e) => {
                            eprintln!("Local socket failed to accept connection: {}", e);
                        },
                    }
                }
            });
        }

        loop {
            match self.tcp_listener.accept() {
                Ok((stream, addr)) => {
                    println!("Connection from {}", addr.ip());

                    let launch_handler;
                    {
                        // unwrap cause we don't want a poisoned lock:
                        // https://doc.rust-lang.org/std/sync/struct.Mutex.html#poisoning
                        let mut streams = streams_arc.lock().unwrap();
                        streams.push(Stream::Tcp(stream));
                        launch_handler = streams.len() == 1;
                    }

                    if launch_handler {
                        let handler = ClientHandler::new(self.gps.clone(), streams_arc.clone());

                        thread::spawn(move || {
                            handler.handle();
                        });
                    }
                }

                Err(e) => {
                    println!("Connect from client failed: {}", e);
                }
            }
        }
    }
}
