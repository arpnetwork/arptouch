// Copyright 2018 ARP Network
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

extern crate arptouch;

use arptouch::{
    command::{Command, Command::*}, device,
};
use std::thread::sleep;
use std::time::Duration;

fn main() {
    if let Some(mut mt) = device::autodetect() {
        {
            let dev = &mt.dev;
            println!(
                "{} {} {} {} {} {} {}",
                dev.max_contacts(),
                dev.max_x(),
                dev.max_y(),
                dev.max_pressure(),
                dev.max_touch_major(),
                dev.max_touch_minor(),
                dev.path()
            );
        }

        let mut buf = String::new();
        while let Ok(_) = std::io::stdin().read_line(&mut buf) {
            if let Ok(cmd) = Command::parse(buf.trim()) {
                match cmd {
                    Commit => mt.commit(),
                    Reset => mt.reset(),
                    Down(contact, x, y, pressure, touch_major, touch_minor) => {
                        mt.touch_down(contact, x, y, pressure, touch_major, touch_minor)
                    }
                    Move(contact, x, y, pressure, touch_major, touch_minor) => {
                        mt.touch_move(contact, x, y, pressure, touch_major, touch_minor)
                    }
                    Up(contact) => mt.touch_up(contact),
                    Wait(ms) => sleep(Duration::from_millis(ms)),
                }
            }

            buf.clear();
        }
    } else {
        eprintln!("Can't find any multitouch device.");
        std::process::exit(-1);
    }
}
