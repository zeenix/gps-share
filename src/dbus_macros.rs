/*
 * Copyright (c) 2016 Boucher, Antoni <bouanto@zoho.com>
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy of
 * this software and associated documentation files (the "Software"), to deal in
 * the Software without restriction, including without limitation the rights to
 * use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
 * the Software, and to permit persons to whom the Software is furnished to do so,
 * subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
 * FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
 * COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
 * IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
 * CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
 */

/*
 * TODO: add Cargo categories.
 * TODO: Switch to macro 1.1.
 */

pub fn to_camel(term: &str) -> String {
    let underscore_count = term.chars().filter(|c| *c == '_').count();
    let mut result = String::with_capacity(term.len() - underscore_count);
    let mut at_new_word = true;

    for c in term.chars() {
        if c == '_' {
            at_new_word = true;
        } else if at_new_word {
            result.push(c.to_ascii_uppercase());
            at_new_word = false;
        } else {
            result.push(c);
        }
    }

    result
}

//
// Server-side
//

#[macro_export]
macro_rules! dbus_functions {
    ($self_:expr, $factory:expr, $interface:ident,) => {
    };
    ($self_:expr, $factory:expr, $interface:ident, fn $func_name:ident (&$this:ident $(, $arg:ident : $arg_type:ty )* ) -> Result<$return_type:ty,$error:ty> $block:block $($rest:tt)*) => {
        let $this = $self_.clone();
        let $interface = $interface.add_m(
            $factory.method(::dbus_macros::to_camel(stringify!($func_name)), (), move |method| {
                let mut i = method.msg.iter_init();
                $(
                    let $arg: $arg_type = i.get().ok_or(dbus::tree::MethodErr::no_arg())?;
                    i.next();
                )*
                let result: $return_type = $block?;
                Ok(vec!(method.msg.method_return().append1(result)))
            })
                $(
                    .inarg::<$arg_type, _>(stringify!($arg))
                )*
                .outarg::<$return_type, _>("result")
        );
        dbus_functions!($self_, $factory, $interface, $($rest)*);
    };
    ($self_:expr, $factory:expr, $interface:ident, fn $func_name:ident (&$this:ident $(, $arg:ident : $arg_type:ty )* ) -> $return_type:ty $block:block $($rest:tt)*) => {
        let $this = $self_.clone();
        let $interface = $interface.add_m(
            $factory.method(::dbus_macros::to_camel(stringify!($func_name)), (), move |method| {
                let mut i = method.msg.iter_init();
                $(
                    let $arg: $arg_type = i.get().ok_or(dbus::tree::MethodErr::no_arg())?;
                    i.next();
                )*
                let result = $block;
                Ok(vec!(method.msg.method_return().append1(result)))
            })
                $(
                    .inarg::<$arg_type, _>(stringify!($arg))
                )*
                .outarg::<$return_type, _>("result")
        );
        dbus_functions!($self_, $factory, $interface, $($rest)*);
    };
    ($self_:expr, $factory:expr, $interface:ident, fn $func_name:ident (&$this:ident $(, $arg:ident : $arg_type:ty )* ) $block:block $($rest:tt)*) => {
        let $this = $self_.clone();
        let $interface = $interface.add_m(
            $factory.method(::dbus_macros::to_camel(stringify!($func_name)), (), move |method| {
                let mut i = method.msg.iter_init();
                $(
                    let $arg: $arg_type = i.get().ok_or(dbus::tree::MethodErr::no_arg())?;
                    i.next();
                )*
                $block;
                let result = 0;
                Ok(vec!(method.msg.method_return().append1(result)))
            })
                $(
                    .inarg::<$arg_type, _>(stringify!($arg))
                )*
                .outarg::<i32, _>("result")
        );
        dbus_functions!($self_, $factory, $interface, $($rest)*);
    };
}

#[macro_export]
macro_rules! dbus_class {
    ($interface_name:expr, class $class_name:ident { $($functions:tt)* }) => {
        #[derive(Clone)]
        pub struct $class_name {
        }

        impl $class_name {
            pub fn new() -> Self {
                $class_name {
                }
            }

            pub fn run<P>(&self, bus_name: &str, connection: &dbus::Connection, path: P)  where P: Into<dbus::Path<'static>> {
                connection.register_name(bus_name, dbus::NameFlag::ReplaceExisting as u32).unwrap();

                let factory = dbus::tree::Factory::new_fn::<()>();
                let class = factory.tree(()).add(factory.object_path(path, ()).introspectable().add({
                    let interface = factory.interface($interface_name, ());
                    dbus_functions!(self, factory, interface, $($functions)*);
                    interface
                }));
                class.set_registered(&connection, true).unwrap();

                for _ in class.run(&connection, connection.iter(1000)) {
                }
            }
        }
    };
    ($interface_name:expr, class $class_name:ident ($($variables:ident : $variable_types:ty),*) { $($functions:tt)* }) => {
        #[derive(Clone)]
        pub struct $class_name {
            $($variables : $variable_types,)*
        }

        impl $class_name {
            pub fn new($($variables: $variable_types),*) -> Self {
                $class_name {
                    $($variables : $variables,)*
                }
            }

            pub fn run<P>(&self, bus_name: &str, connection: &dbus::Connection, path: P)  where P: Into<dbus::Path<'static>> {
                connection.register_name(bus_name, dbus::NameFlag::ReplaceExisting as u32).unwrap();

                let factory = dbus::tree::Factory::new_fn::<()>();
                let class = factory.tree(()).add(factory.object_path(path, ()).introspectable().add({
                    let interface = factory.interface($interface_name, ());
                    dbus_functions!(self, factory, interface, $($functions)*);
                    interface
                }));
                class.set_registered(&connection, true).unwrap();

                for _ in class.run(&connection, connection.iter(1000)) {
                }
            }
        }
    };
}

//
// Client-side
//

#[macro_export]
macro_rules! dbus_prototypes {
    ($interface_name:expr, $class_name:ident, ) => {
    };
    ($interface_name:expr, $class_name:ident, fn $func_name:ident ( $( $arg:ident : $arg_type:ty ),* ) -> $return_type:ty; $($rest:tt)*) => {
        pub fn $func_name(&self, $( $arg: $arg_type ),* ) -> Result<$return_type, dbus::Error> {
            let message = dbus::Message::new_method_call(&self.bus_name, self.path.clone(), $interface_name, ::dbus_macros::to_camel(stringify!($func_name))).unwrap();
            $(
                let message = message.append1($arg);
            )*
            let response = try!(self.connection.send_with_reply_and_block(message, 2000));
            response.get1().ok_or(dbus::Error::from(dbus::tree::MethodErr::no_arg()))
        }
        dbus_prototypes!($interface_name, $class_name, $($rest)*);
    };
    ($interface_name:expr, $class_name:ident, fn $func_name:ident ( $( $arg:ident : $arg_type:ty ),* ) ; $($rest:tt)*) => {
        pub fn $func_name(&self, $( $arg: $arg_type ),* ) -> Result<(), dbus::Error> {
            let message = dbus::Message::new_method_call(&self.bus_name, self.path.clone(), $interface_name, ::dbus_macros::to_camel(stringify!($func_name))).unwrap();
            $(
                let message = message.append1($arg);
            )*
            self.connection.send(message).ok();
            Ok(())
        }
        dbus_prototypes!($interface_name, $class_name, $($rest)*);
    };
}

#[macro_export]
macro_rules! dbus_interface {
    ($interface_name:expr, interface $class_name:ident { $($prototypes:tt)* }) => {
        pub struct $class_name<'a> {
            bus_name: String,
            path: dbus::Path<'a>,
            connection: Rc<dbus::Connection>,
        }

        impl<'a>  $class_name<'a> {
            pub fn new<P>(dbus_name: &str, path: P, connection: Rc<dbus::Connection>) -> Self where P: Into<dbus::Path<'a>> {
                $class_name {
                    bus_name: dbus_name.to_string(),
                    path: path.into(),
                    connection: connection,
                }
            }

            dbus_prototypes!($interface_name, $class_name, $($prototypes)*);
        }
    };
}
