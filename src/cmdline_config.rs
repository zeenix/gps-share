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

use crate::config::Config;
use clap::{Arg, ArgAction, Command, value_parser};

pub fn config_from_cmdline() -> Config {
    let matches = Command::new("GPS Share")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Zeeshan Ali <zeeshanak@gnome.org>")
        .about("Utility to share your GPS device on local network.")
        .arg(
            Arg::new("device")
                .help("GPS device node")
                .required(false)
                .value_parser(value_parser!(std::path::PathBuf)),
        )
        .arg(
            Arg::new("disable-announce")
                .short('a')
                .long("disable-announce")
                .action(ArgAction::SetTrue)
                .help("Disable announcing through Avahi"),
        )
        .arg(
            Arg::new("port")
                .short('p')
                .long("port")
                .help("Port to run TCP service on")
                .value_name("PORT")
                .default_value("10110")
                .value_parser(value_parser!(u16)),
        )
        .arg(
            Arg::new("interface")
                .short('n')
                .long("network-interface")
                .help("Bind specific network interface (default: all)")
                .value_name("INTERFACE"),
        )
        .arg(
            Arg::new("no-tcp")
                .short('x')
                .long("no-tcp")
                .action(ArgAction::SetTrue)
                .help("Don't share over TCP"),
        )
        .arg(
            Arg::new("socket")
                .short('s')
                .long("socket-path")
                .help("Path to place the socket service (default: don't run)")
                .value_name("SOCKET"),
        )
        .arg(
            Arg::new("baudrate")
                .short('b')
                .long("baudrate")
                .help("Baudrate to use for communication with GPS device")
                .value_name("BAUDRATE")
                .default_value("38400")
                .value_parser(value_parser!(u32)),
        )
        .get_matches();

    let announce = !matches.get_flag("disable-announce");
    let dev_path = matches.get_one::<std::path::PathBuf>("device").cloned();
    let port = *matches.get_one::<u16>("port").expect("has a default");
    let no_tcp = matches.get_flag("no-tcp");
    let iface = matches.get_one::<String>("interface").cloned();
    let socket_path = matches.get_one::<String>("socket").cloned();
    let baudrate = *matches.get_one::<u32>("baudrate").expect("has a default");

    Config {
        dev_path,
        announce_on_net: announce,
        port,
        net_iface: iface,
        no_tcp,
        socket_path,
        baudrate,
    }
}
