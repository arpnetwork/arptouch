// Copyright 2018 ARP Network
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::error::Error;
use std::io;

/// Multitouch Command
pub enum Command {
    Commit,
    Reset,
    Down(usize, i32, i32, i32),
    Move(usize, i32, i32, i32),
    Up(usize),
    Wait(u64),
}

impl Command {
    /// Try parses string into a multitouch command.
    pub fn parse(s: &str) -> Result<Command, Box<Error>> {
        let invalid_input = || Box::new(io::Error::from(io::ErrorKind::InvalidInput));

        let items: Vec<_> = s.split(" ").collect();
        if !items.is_empty() {
            let len = items.len();
            match items[0] {
                "c" if len == 1 => Ok(Command::Commit),
                "r" if len == 1 => Ok(Command::Reset),
                "d" if len == 5 => Ok(Command::Down(
                    items[1].parse()?,
                    items[2].parse()?,
                    items[3].parse()?,
                    items[4].parse()?,
                )),
                "m" if len == 5 => Ok(Command::Move(
                    items[1].parse()?,
                    items[2].parse()?,
                    items[3].parse()?,
                    items[4].parse()?,
                )),
                "u" if len == 2 => Ok(Command::Up(items[1].parse()?)),
                "w" if len == 2 => Ok(Command::Wait(items[1].parse()?)),
                _ => Err(invalid_input()),
            }
        } else {
            Err(invalid_input())
        }
    }
}
