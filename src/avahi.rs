/* vim: set et ts=4 sw=4: */
/* avahi.rs
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

use std::rc::Rc;
use zbus;
use zbus::dbus_proxy;
use zvariant::OwnedObjectPath;

#[dbus_proxy(
    interface = "org.freedesktop.Avahi.Server",
    default_service = "org.freedesktop.Avahi",
    default_path = "/",
)]
trait Server {
    fn entry_group_new(&self) -> zbus::Result<OwnedObjectPath>;
    fn get_network_interface_index_by_name(&self, name: &str) -> zbus::Result<i32>;
}

#[dbus_proxy(
    interface = "org.freedesktop.Avahi.EntryGroup",
    default_path = "/",
)]
trait EntryGroup {
    fn add_service(&self,
        ifindex: i32,
        protocol: i32,
        flags: u32,
        name: &str,
        service_type: &str,
        domain: &str,
        host: &str,
        port: u16,
        text: Vec<Vec<u8>>
    ) -> zbus::Result<()>;
    fn commit(&self) -> zbus::Result<()>;
}

pub struct Avahi {
    connection: Rc<zbus::Connection>,
}

impl Avahi {
    pub fn new() -> Result<Self, zbus::Error> {
        let connection = zbus::Connection::new_system()?;
        let connection = Rc::new(connection);

        Ok(Avahi {
            connection: connection,
        })
    }

    pub fn publish(&self, net_iface: Option<&str>, port: u16) -> Result<(), zbus::Error> {
        let server = ServerProxy::new(&self.connection.clone())?;
        
        // FIXME: Make this async when it's possible
        let group_path = server.entry_group_new()?;
        println!("group: {}", group_path.as_str());

        let group = EntryGroupProxy::new_for(&self.connection.clone(), "org.freedesktop.Avahi", &group_path)?;
        let txt = "accuracy=exact".to_string();
        let array: Vec<Vec<u8>> = vec![txt.into_bytes()];

        let iface = match net_iface {
            Some(name) => match server.get_network_interface_index_by_name(name) {
                Ok(i) => i,
                Err(e) => {
                    println!("Failed to get interface index from Avahi: {}", e);

                    -1
                }
            },
            None => -1,
        };
        group.add_service(
            iface,
            -1,
            0,
            "gps-share",
            "_nmea-0183._tcp",
            "",
            "",
            port,
            array,
        )?;
        group.commit()?;

        Ok(())
    }
}
