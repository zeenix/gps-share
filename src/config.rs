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

use libc;
use std::ptr;
use std::mem;
use std::ffi::{CStr, CString};
use serial;

pub enum DeviceID {
    Path(::std::path::PathBuf),
    MAC(String),
}

pub struct Config {
    pub dev_id: Option<DeviceID>,
    pub announce_on_net: bool,
    pub port: u16,
    pub net_iface: Option<String>,
    pub baudrate: usize,
}

impl Config {
    pub fn get_ip(& self) -> String {
        match self.net_iface {
            Some(ref iface) => {
                unsafe {
                    Config::get_ip_for_iface(iface)
                }
            },

            None => "0.0.0.0".to_string(),
        }
    }

    pub fn get_baudrate(& self) -> serial::BaudRate {
        match self.baudrate {
            110    => serial::Baud110,
            300    => serial::Baud300,
            600    => serial::Baud600,
            1200   => serial::Baud1200,
            2400   => serial::Baud2400,
            4800   => serial::Baud4800,
            9600   => serial::Baud9600,
            19200  => serial::Baud19200,
            38400  => serial::Baud38400,
            57600  => serial::Baud57600,
            115200 => serial::Baud115200,
            b      => serial::BaudOther(b),
        }
    }

    unsafe fn get_ip_for_iface(iface: & String) -> String {
        let mut addr_ptr = ptr::null_mut();

        let ret = libc::getifaddrs(& mut addr_ptr);
        if ret != 0 || addr_ptr.is_null() {
            return "0.0.0.0".to_string();
        }

        while !addr_ptr.is_null() {
            let addr = *addr_ptr;
            addr_ptr = addr.ifa_next;

            let name;
            match CStr::from_ptr(addr.ifa_name).to_str() {
                Ok(n) => name = n,
                Err(e) => {
                    println!("{}", e);

                    continue;
                },
            };

            if name != iface.as_str() || addr.ifa_addr.is_null() {
                continue;
            }

            let mut host = CString::from_vec_unchecked(vec![0u8; libc::NI_MAXHOST as usize]);
            let size;
            match i32::from((*addr.ifa_addr).sa_family) {
                libc::AF_INET  => size = mem::size_of::<libc::sockaddr_in>() as u32,
                libc::AF_INET6 => size = mem::size_of::<libc::sockaddr_in6>() as u32,
                _ => continue,
            };
            let host_ptr = host.into_raw() as * mut i8;
            let ret = libc::getnameinfo(addr.ifa_addr, size,
                                        host_ptr, libc::NI_MAXHOST,
                                        ptr::null_mut(), 0,
                                        libc::NI_NUMERICHOST);
            host = CString::from_raw(host_ptr);
            if ret != 0 {
                return "0.0.0.0".to_string();
            }

            match host.into_string() {
                Ok(ip) => return ip,
                Err(e) => {
                    println!("{}", e);

                    continue;
                },

            }
        }

        "0.0.0.0".to_string()
    }
}
