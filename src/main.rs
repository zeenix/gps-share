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

mod gps;
mod server;
mod avahi;

extern crate serial;
extern crate dbus;
#[macro_use]
extern crate dbus_macros;

use gps::GPS;
use server::Server;

fn main() {
    let mut args = std::env::args();
    if args.len() == 1 {
        let arg0 = match args.nth(0) {
            Some(s) => s,
            None => String::from("gps-share"),
        };
        println!("Usage: {} DEVICE_PATH", arg0);

        return;
    }

    let dev_path = args.nth(1).unwrap();
    let gps = GPS::new(dev_path.as_str()).unwrap();
    let mut server = Server::new(gps).unwrap();

    server.run().unwrap();
}
