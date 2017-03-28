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

use dbus;

dbus_interface!("org.freedesktop.Avahi.Server", interface Server {
    fn entry_group_new() -> dbus::Path;
});

dbus_interface!("org.freedesktop.Avahi.EntryGroup", interface EntryGroup {
    fn add_service(ifindex: i32,
                   protocol: i32,
                   flags: u32,
                   name: &str,
                   service_type: &str,
                   domain: &str,
                   host: &str,
                   port: u16,
                   text: &str);
    fn commit();
});

pub fn publish(port: u16) -> Result<(),dbus::Error> {
    let server = Server::new("org.freedesktop.Avahi", "/", dbus::BusType::System);

    // FIXME: Make this async when it's possible
    let group_path = server.entry_group_new()?;
    println!("group: {}", group_path);

    let group = EntryGroup::new("org.freedesktop.Avahi", group_path, dbus::BusType::System);
    group.add_service(-1, -1, 0, "gps-share", "_nmea-0183._tcp", "", "", port, "")?;
    group.commit()?;

    Ok(())
}
