/* vim: set et ts=4 sw=4: */
/* stdin_gps.rs
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
use std::process::{Command, Stdio, Child};
use std::io::Write;
use std::io::Read;
use std::net::TcpStream;

#[test]
fn test_stdin_gps() {
    let mut child = Command::new("target/debug/gps-share")
                        .arg("-a")
                        .arg("-")
                        .stdin(Stdio::piped())
                        .stdout(Stdio::piped())
                        .spawn()
                        .expect("Failed to start gps-share");

    let nmea_trace = "\
                      $GPVTG,0.0,T,,M,0.0,N,0.0,K,A*0D\n\
                      $GPGLL,5744.4784,N,01201.6130,E,122731.00,A,A*66\n\
                      $GPGSA,A,3,02,12,19,24,,,,,,,,,9.6,6.5,7.1*37\n\
                      $GPRMC,122732.000,A,5744.4784,N,01201.6130,E,0.0,0.0,300417,,,A*63\n\
                      $GPGGA,122732.000,5744.4784,N,01201.6130,E,1,04,6.5,61.7,M,44.5,M,,0000*62\n";

    write_nmea_to_child(& mut child, nmea_trace);

    let port = get_port(& mut child);
    println!("Port is {}", port);

    let trace = get_nmea_from_service(port, nmea_trace.len());
    assert_eq!(trace, nmea_trace);

    child.kill().unwrap();
}

fn write_nmea_to_child(child: & mut Child, nmea_trace: &str) {
    if let Some(ref mut stdin) = child.stdin {
        let len = stdin.write(nmea_trace.as_ref()).unwrap();

        assert_eq!(len, nmea_trace.len());
    };
}

fn get_port(child: & mut Child) -> u16 {
    let port;
    if let Some(ref mut stdout) = child.stdout {
        let mut output = [0u8; 1024];

        let n = stdout.read(& mut output).unwrap();
        assert!(n > 0);

        let output = String::from_utf8(output.to_vec()).unwrap();

        let line2 = output.split("\n").nth(1).unwrap();
        let port_str = line2.split(" ").nth(1).unwrap();
        port = u16::from_str_radix(port_str, 10).unwrap();
    } else {
        panic!();
    }

    port
}

fn get_nmea_from_service(port: u16, trace_len: usize) -> String {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();

    let mut output = vec![0u8; trace_len];

    stream.read_exact(& mut output[..]).unwrap();

    String::from_utf8(output).unwrap()
}
