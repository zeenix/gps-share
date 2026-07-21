/* vim: set et ts=4 sw=4: */
/* config.rs
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

use std::ffi::CStr;
use std::mem;
use std::path::PathBuf;
use std::ptr;

pub struct Config {
    pub dev_path: Option<PathBuf>,
    pub announce_on_net: bool,
    pub port: u16,
    pub net_iface: Option<String>,
    pub no_tcp: bool,
    pub socket_path: Option<String>,
    pub baudrate: u32,
}

impl Config {
    pub fn get_ip(&self) -> String {
        match self.net_iface {
            Some(ref iface) => Config::get_ip_for_iface(iface),

            None => "0.0.0.0".to_string(),
        }
    }

    fn get_ip_for_iface(iface: &str) -> String {
        let mut addr_ptr = ptr::null_mut();

        // SAFETY: `getifaddrs` only writes the list head into `addr_ptr`, which points at a valid
        // local variable.
        let ret = unsafe { libc::getifaddrs(&mut addr_ptr) };
        if ret != 0 || addr_ptr.is_null() {
            return "0.0.0.0".to_string();
        }

        while !addr_ptr.is_null() {
            // SAFETY: `addr_ptr` is non-null and points at an entry of the list `getifaddrs`
            // returned, which stays alive for the whole walk.
            let addr = unsafe { *addr_ptr };
            addr_ptr = addr.ifa_next;

            // SAFETY: `ifa_name` is a NUL-terminated string owned by the entry above.
            let name = match unsafe { CStr::from_ptr(addr.ifa_name) }.to_str() {
                Ok(n) => n,
                Err(e) => {
                    println!("{}", e);

                    continue;
                }
            };

            if name != iface || addr.ifa_addr.is_null() {
                continue;
            }

            // SAFETY: `ifa_addr` was checked for NULL above and points at a `sockaddr`.
            let size = match i32::from(unsafe { (*addr.ifa_addr).sa_family }) {
                libc::AF_INET => mem::size_of::<libc::sockaddr_in>() as u32,
                libc::AF_INET6 => mem::size_of::<libc::sockaddr_in6>() as u32,
                _ => continue,
            };
            let mut host = vec![0u8; libc::NI_MAXHOST as usize];
            // SAFETY: `addr.ifa_addr` describes `size` bytes of address and `host` provides the
            // `NI_MAXHOST` bytes of output space we claim it does.
            let ret = unsafe {
                libc::getnameinfo(
                    addr.ifa_addr,
                    size,
                    host.as_mut_ptr() as *mut libc::c_char,
                    libc::NI_MAXHOST,
                    ptr::null_mut(),
                    0,
                    libc::NI_NUMERICHOST,
                )
            };
            if ret != 0 {
                return "0.0.0.0".to_string();
            }

            // SAFETY: on success `getnameinfo` leaves a NUL-terminated string in `host`.
            let host = unsafe { CStr::from_ptr(host.as_ptr() as *const libc::c_char) };
            match host.to_str() {
                Ok(ip) => return ip.to_string(),
                Err(e) => {
                    println!("{}", e);

                    continue;
                }
            }
        }

        "0.0.0.0".to_string()
    }
}
