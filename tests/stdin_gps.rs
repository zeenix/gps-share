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
use std::fs;
use std::io::Read;
use std::io::Write;
use std::net::TcpStream;
use std::os::unix::net::{UnixStream};
use std::process::{Child, Command, Stdio};


enum LocalSocket {
    Only(&'static str),
    Some(&'static str),
    None,
}

#[test]
fn stdin_gps() {
    // Just with default options.
    test_stdin_gps(None, None, LocalSocket::None);

    // With TCP port specified.
    test_stdin_gps(Some(9314), None, LocalSocket::None);
    // With TCP port and interface specified.
    test_stdin_gps(Some(9315), Some("lo"), LocalSocket::None);

    // Local only.
    test_stdin_gps(None, None, LocalSocket::Only("/tmp/sock"));
    // Local with defaults.
    test_stdin_gps(None, None, LocalSocket::Some("/tmp/sock"));
}

fn test_stdin_gps(tcp_port: Option<u16>, net_iface: Option<&str>, local_socket: LocalSocket) {
    let mut cmd = Command::new("target/debug/gps-share");

    cmd.arg("-a")
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped());
    if let Some(port) = tcp_port {
        cmd.args(&["-p", &port.to_string()]);
    }
    if let Some(iface) = net_iface {
        cmd.args(&["-n", iface]);
    }
    match &local_socket {
        LocalSocket::Only(path) => {
            cmd.args(&["--no-tcp", "--socket-path", path]);
        },
        LocalSocket::Some(path) => {
            cmd.args(&["--socket-path", path]);
        },
        LocalSocket::None => {},
    }

    let mut child = cmd.spawn().expect("Failed to start gps-share");

    let nmea_trace = "\
                      $GPVTG,0.0,T,,M,0.0,N,0.0,K,A*0D\n\
                      $GPGLL,5744.4784,N,01201.6130,E,122731.00,A,A*66\n\
                      $GPGSA,A,3,02,12,19,24,,,,,,,,,9.6,6.5,7.1*37\n\
                      $GPRMC,122732.000,A,5744.4784,N,01201.6130,E,0.0,0.0,300417,,,A*63\n\
                      $GPGGA,122732.000,5744.4784,N,01201.6130,E,1,04,6.5,61.7,M,44.5,M,,0000*62\n";

    write_nmea_to_child(&mut child, nmea_trace);

    let port_wanted = match &local_socket {
        LocalSocket::Only(_) => false,
        _ => true,
    };

    if port_wanted {
        let child_port = get_port_from_child(&mut child);
        if let Some(port) = child_port {
            if let Some(requested_port) = tcp_port {
                assert_eq!(port, requested_port);
            }
            let trace = get_nmea_from_port(port, nmea_trace.len());
            assert_eq!(trace, nmea_trace);
        }
    }

    let local_path = match &local_socket {
        LocalSocket::Only(path) => Some(path),
        LocalSocket::Some(path) => Some(path),
        _ => None,
    };
    
    // Read from the local socket
    // if data hasn't already been read from the network.
    if let LocalSocket::Only(path) = local_socket {
        let trace = get_nmea_from_local(path, nmea_trace.len());
        assert_eq!(trace, nmea_trace);
    }

    child.kill().unwrap();
    if let Some(path) = local_path {
        fs::remove_file(path).unwrap();
    }
}

fn write_nmea_to_child(child: &mut Child, nmea_trace: &str) {
    if let Some(ref mut stdin) = child.stdin {
        let len = stdin.write(nmea_trace.as_ref()).unwrap();

        assert_eq!(len, nmea_trace.len());
    };
}

fn get_port_from_child(mut child: &mut Child) -> Option<u16> {
    let mut port = get_port(&mut child);
    if port.is_none() {
        std::thread::sleep(std::time::Duration::from_millis(100));
        port = get_port(&mut child);
    }
    port
}

fn get_port(child: &mut Child) -> Option<u16> {
    let mut port = None;
    let stdout = child.stdout.as_mut().unwrap();
    let mut output = [0u8; 1024];

    stdout.read(&mut output).unwrap();

    let output = String::from_utf8(output.to_vec()).unwrap();

    for line in output.split("\n") {
        if let Some(port_str) = line.split(" ").nth(1) {
            port = u16::from_str_radix(port_str, 10).ok();

            if port.is_some() {
                break;
            }
        }
    }

    port
}

fn get_nmea_from_port(port: u16, trace_len: usize) -> String {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();

    let mut output = vec![0u8; trace_len];

    stream.read_exact(&mut output[..]).unwrap();

    String::from_utf8(output).unwrap()
}

fn get_nmea_from_local(path: &str, trace_len: usize) -> String {
    std::thread::sleep(std::time::Duration::from_millis(200));
    let mut stream = UnixStream::connect(path).unwrap();
    let mut output = vec![0u8; trace_len];

    stream.read_exact(&mut output[..]).unwrap();
    String::from_utf8(output).unwrap()
}
