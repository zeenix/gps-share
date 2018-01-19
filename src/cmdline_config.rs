/* vim: set et ts=4 sw=4: */
/* cmdline_config.rs
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

use clap::{Arg, App};
use config::Config;

pub fn config_from_cmdline() -> Config {
    let matches = App::new("GPS Share")
        .version("0.1")
        .author("Zeeshan Ali <zeeshanak@gnome.org>")
        .about("Utility to share your GPS device on local network.")
        .arg(Arg::with_name("device").help("GPS device node").required(
            false,
        ))
        .arg(
            Arg::with_name("disable-announce")
                .short("a")
                .long("--disable-announce")
                .help("Disable announcing through Avahi"),
        )
        .arg(
            Arg::with_name("port")
                .short("p")
                .long("--port")
                .help("Port to run TCP service on (default: 10110)")
                .takes_value(true)
                .value_name("PORT"),
        )
        .arg(
            Arg::with_name("interface")
                .short("n")
                .long("--network-interface")
                .help("Bind specific network interface")
                .takes_value(true)
                .value_name("INTERFACE"),
        )
        .arg(
            Arg::with_name("baudrate")
                .short("b")
                .long("--baudrate")
                .help("Baudrate to use for communication with GPS device")
                .takes_value(true)
                .value_name("BAUDRATE"),
        )
        .get_matches();

    let announce = !matches.is_present("disable-announce");
    let dev_path = matches.value_of("device").and_then(|p| {
        Some(::std::path::PathBuf::from(p))
    });
    let port: u16 = matches.value_of("port").unwrap_or("10110").parse().unwrap_or(0);
    let iface = matches.value_of("interface").map(|s| s.to_string());
    let baudrate = matches
        .value_of("baudrate")
        .unwrap_or("38400")
        .parse()
        .unwrap_or(38400usize);

    Config {
        dev_path: dev_path,
        announce_on_net: announce,
        port: port,
        net_iface: iface,
        baudrate: baudrate,
    }
}
