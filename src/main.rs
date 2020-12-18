/* vim: set et ts=4 sw=4: */
/* main.rs
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

mod avahi;
mod client_handler;
mod cmdline_config;
mod config;
mod gps;
mod rs232;
mod gnss;
mod server;
mod stdin_gps;

extern crate dbus;
extern crate serial;
#[macro_use]
extern crate dbus_macros;
extern crate core;
#[macro_use]
extern crate chan;
extern crate chan_signal;
extern crate clap;
extern crate libc;
extern crate libudev;

use config::Config;
use gps::GPS;
use rs232::RS232;
use gnss::GNSS;
use server::Server;
use std::thread;
use stdin_gps::StdinGPS;

use chan_signal::Signal;
use std::rc::Rc;

fn main() {
    let config = cmdline_config::config_from_cmdline();

    let signal = chan_signal::notify(&[Signal::INT, Signal::TERM]);
    let (sdone, rdone) = chan::sync(0);

    thread::spawn(move || run(sdone, Rc::new(config)));

    chan_select! {
        signal.recv() -> signal => {
            match signal {
                Some(Signal::INT) => {
                    println!("Interrupt from keyboard. Exitting..");
                },

                Some(Signal::TERM) => {
                    println!("Kill signal received. Exitting..");
                },

                _ => (),
            }
        },

        rdone.recv() => {
            println!("Program completed normally.");
        }
    }
}

fn run(_sdone: chan::Sender<()>, config: Rc<Config>) {
    let gps = get_gps(config.clone());

    run_server_handle_err(gps, config.clone());
}

fn get_gps(config: Rc<Config>) -> Box<GPS> {
    if let Some(ref path) = config.dev_path {
        if path.to_str() == Some("-") {
            return Box::new(StdinGPS::new());
        }
    }

    // FIXME: the discovery part should be separated from the RS232 module so that adding
    //  more devices doesn't get even more convoluted.
    match RS232::new(config.clone()) {
        Ok(rs232) => return Box::new(rs232),

        Err(e) => {
            match e.kind() {
                ::std::io::ErrorKind::NotFound => match GNSS::new(config.clone()) {
                    Ok(gnss) => return Box::new(gnss),

                    Err(e) => {
                        match e.kind() {
                            ::std::io::ErrorKind::NotFound => println!("{}", e),

                            _ => println!("Failed to open GNSS device: {}", e),
                        }

                        std::process::exit(1);
                    }
                },

                _ => {println!("Failed to open serial device: {}", e); std::process::exit(1)},
            }
        }
    }
}

fn run_server_handle_err(gps: Box<GPS>, config: Rc<Config>) {
    if let Err(e) = run_server(gps, config) {
        println!("Failed to start TCP service: {}", e);

        std::process::exit(2);
    }
}

fn run_server(gps: Box<GPS>, config: Rc<Config>) -> ::std::io::Result<()> {
    let mut server = Server::new(gps, config)?;

    server.run()
}
