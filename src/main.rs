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

mod rs232;
mod gps;
mod server;
mod avahi;
mod client_handler;
mod stdin_gps;
mod config;
mod cmdline_config;

extern crate serial;
extern crate dbus;
#[macro_use]
extern crate dbus_macros;
extern crate core;
#[macro_use]
extern crate chan;
extern crate chan_signal;
extern crate clap;

use gps::GPS;
use rs232::RS232;
use stdin_gps::StdinGPS;
use server::Server;
use config::Config;
use std::thread;

use chan_signal::Signal;

fn main() {
    let config = cmdline_config::config_from_cmdline();

    let signal = chan_signal::notify(&[Signal::INT, Signal::TERM]);
    let (sdone, rdone) = chan::sync(0);

    thread::spawn(move || run(sdone, config));

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

fn run(_sdone: chan::Sender<()>, config: Config) {
    match config.dev_path.as_ref() {
        "-" => {
            let stdin_gps = StdinGPS::new();

            run_server(stdin_gps, config);
        },
        _   => {
            let rs232 = RS232::new(config.dev_path.as_str()).unwrap();

            run_server(rs232, config);
        },
    };
}

fn run_server<G: GPS>(gps: G, config: Config) {
    let mut server = Server::new(gps, config).unwrap();

    server.run().unwrap();
}
