// Copyright 2018 ARP Network
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

extern crate libc;
extern crate libevdev_sys;

use self::libevdev_sys::{evdev::*, input_event_codes::*, linux_input::input_event};

use std::ffi::CStr;
use std::fs::{read_dir, OpenOptions};
use std::io;
use std::mem::{size_of_val, transmute};
use std::os::unix::fs::FileTypeExt;
use std::os::unix::io::{IntoRawFd, RawFd};
use std::path::Path;
use std::ptr;

type Evdev = libevdev;
type InputEvent = input_event;

struct State {
    contacts: Vec<bool>,
    actived: u32,
    tracking_id: i32,
}

/// Input Device
pub struct Device {
    evdev: *mut Evdev,
    path: String,
    fd: RawFd,
}

/// Multitouch Device
pub struct MTDevice {
    pub dev: Device,
    max_contacts: usize,
    max_tracking_id: i32,
    has_btn_touch: bool,
    has_btn_tool_finger: bool,
    has_pressure: bool,
    state: State,
}

impl Device {
    /// Attempts to open a device in read-write mode.
    pub fn open(path: &AsRef<Path>) -> io::Result<Device> {
        let fd = OpenOptions::new().write(true).open(path)?.into_raw_fd();

        let mut evdev: *mut Evdev = ptr::null_mut();
        let ret = unsafe { libevdev_new_from_fd(fd, &mut evdev) };
        if ret != 0 {
            return Err(io::Error::from_raw_os_error(ret));
        }

        let path = String::from(path.as_ref().to_str().unwrap());
        Ok(Device { evdev, path, fd })
    }

    /// Write a input event into this device.
    pub fn write_event(&self, type_: u16, code: u16, value: i32) {
        let event = InputEvent {
            type_,
            code,
            value,
            ..InputEvent::default()
        };
        unsafe {
            libc::write(self.fd, transmute(&event), size_of_val(&event));
        }
    }

    /// Gets the device's name.
    pub fn name(&self) -> &str {
        let name = unsafe { libevdev_get_name(self.evdev) };
        unsafe { CStr::from_ptr(name) }.to_str().unwrap()
    }

    /// Gets the device's path.
    pub fn path(&self) -> &str {
        self.path.as_str()
    }

    /// Gets the device's max contacts count.
    pub fn max_contacts(&self) -> i32 {
        self.abs_max(ABS_MT_SLOT).unwrap_or(-1) + 1
    }

    /// Gets the device's max value in the x-axis.
    pub fn max_x(&self) -> i32 {
        self.abs_max(ABS_MT_POSITION_X).unwrap_or(0)
    }

    /// Gets the device's max value in the y-axis.
    pub fn max_y(&self) -> i32 {
        self.abs_max(ABS_MT_POSITION_Y).unwrap_or(0)
    }

    /// Gets the device's max pressure value.
    pub fn max_pressure(&self) -> i32 {
        self.abs_max(ABS_MT_PRESSURE).unwrap_or(0)
    }

    /// Gets the device's max touch-major value.
    pub fn max_touch_major(&self) -> i32 {
        self.abs_max(ABS_MT_TOUCH_MAJOR).unwrap_or(0)
    }

    /// Gets the device's max touch-minor value.
    pub fn max_touch_minor(&self) -> i32 {
        self.abs_max(ABS_MT_TOUCH_MINOR).unwrap_or(0)
    }

    /// Returns whether this device is a multitouch device.
    /// Only support `Type B` currently.
    pub fn is_multitouch(&self) -> bool {
        self.has_abs_code(ABS_MT_SLOT)
    }

    fn abs_max(&self, code: u16) -> Option<i32> {
        if self.has_abs_code(code) {
            Some(self.get_abs_maximum(code))
        } else {
            None
        }
    }

    fn get_abs_maximum(&self, code: u16) -> i32 {
        unsafe { libevdev_get_abs_maximum(self.evdev, code as u32) }
    }

    fn has_abs_code(&self, code: u16) -> bool {
        self.has_event_code(EV_ABS, code)
    }

    fn has_event_code(&self, type_: u16, code: u16) -> bool {
        (unsafe { libevdev_has_event_code(self.evdev, type_ as u32, code as u32) }) == 1
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe {
            libevdev_free(self.evdev);
            libc::close(self.fd);
        }
    }
}

impl MTDevice {
    /// Constructs a new `MTDevice`
    pub fn new(dev: Device) -> MTDevice {
        let size = dev.max_contacts() as usize;
        let mut contacts = Vec::with_capacity(size);
        contacts.resize(size, false);
        let max_tracking_id = dev.get_abs_maximum(ABS_MT_TRACKING_ID);
        let has_btn_touch = dev.has_event_code(EV_KEY, BTN_TOUCH);
        let has_btn_tool_finger = dev.has_event_code(EV_KEY, BTN_TOOL_FINGER);
        let has_pressure = dev.has_abs_code(ABS_MT_PRESSURE);

        MTDevice {
            dev,
            max_contacts: size,
            max_tracking_id,
            has_btn_touch,
            has_btn_tool_finger,
            has_pressure,
            state: State {
                contacts,
                actived: 0,
                tracking_id: 0,
            },
        }
    }

    /// Schedules a touch down on contact `<contact>` at `<x>,<y>` with
    /// `<pressure>` pressure for the next commit.
    pub fn touch_down(
        &mut self,
        contact: usize,
        x: i32,
        y: i32,
        pressure: i32,
        touch_major: i32,
        touch_minor: i32,
    ) {
        if contact < self.max_contacts {
            if !self.state.contacts[contact] {
                self.state.contacts[contact] = true;
                self.state.actived += 1;

                self.touch_down_or_move(contact, x, y, pressure, touch_major, touch_minor, true);
            }
        }
    }

    /// Schedules a touch move on contact `<contact>` at `<x>,<y>` with
    /// `<pressure>` pressure for the next commit.
    pub fn touch_move(
        &mut self,
        contact: usize,
        x: i32,
        y: i32,
        pressure: i32,
        touch_major: i32,
        touch_minor: i32,
    ) {
        if contact < self.max_contacts && self.state.contacts[contact] {
            self.touch_down_or_move(contact, x, y, pressure, touch_major, touch_minor, false);
        }
    }

    /// Schedules a touch up on contact `<contact>`.
    pub fn touch_up(&mut self, contact: usize) {
        if contact < self.max_contacts && self.state.contacts[contact] {
            self.state.contacts[contact] = false;
            self.state.actived -= 1;

            self.write_abs_event(ABS_MT_SLOT, contact as i32);
            self.write_abs_event(ABS_MT_TRACKING_ID, -1);

            if self.state.actived == 0 {
                self.write_btn_event(0);
            }
        }
    }

    /// Attemps to reset the current set of touches by creating appropriate
    /// touch up events and then committing them.
    pub fn reset(&mut self) {
        if self.state.actived > 0 {
            for i in 0..self.max_contacts {
                if self.state.contacts[i] {
                    self.touch_up(i);
                }
            }
            self.commit();
        }
    }

    /// Commits the current set of changed touches, causing them to play out on the screen.
    /// Note that nothing visible will happen until you commit.
    pub fn commit(&self) {
        self.dev.write_event(EV_SYN, SYN_REPORT, 0);
    }

    fn touch_down_or_move(
        &mut self,
        contact: usize,
        x: i32,
        y: i32,
        pressure: i32,
        touch_major: i32,
        touch_minor: i32,
        is_down: bool,
    ) {
        self.write_abs_event(ABS_MT_SLOT, contact as i32);
        if is_down {
            let tracking_id = self.next_tracking_id();
            self.write_abs_event(ABS_MT_TRACKING_ID, tracking_id);

            if self.state.actived == 1 {
                self.write_btn_event(1);
            }
        }

        if self.has_pressure {
            self.write_abs_event(ABS_MT_PRESSURE, pressure);
        }
        self.write_abs_event(ABS_MT_TOUCH_MAJOR, touch_major);
        self.write_abs_event(ABS_MT_TOUCH_MINOR, touch_minor);
        self.write_abs_event(ABS_MT_POSITION_X, x);
        self.write_abs_event(ABS_MT_POSITION_Y, y);
    }

    fn write_btn_event(&self, value: i32) {
        if self.has_btn_touch {
            self.write_key_event(BTN_TOUCH, value);
        }
        if self.has_btn_tool_finger {
            self.write_key_event(BTN_TOOL_FINGER, value);
        }
    }

    fn write_abs_event(&self, code: u16, value: i32) {
        self.dev.write_event(EV_ABS, code, value);
    }

    fn write_key_event(&self, code: u16, value: i32) {
        self.dev.write_event(EV_KEY, code, value);
    }

    fn next_tracking_id(&mut self) -> i32 {
        let tracking_id = self.state.tracking_id;
        if tracking_id < self.max_tracking_id {
            self.state.tracking_id += 1;
        } else {
            self.state.tracking_id = 0;
        }
        tracking_id
    }
}

/// Autodetect a multitouch device and return it.
pub fn autodetect() -> Option<MTDevice> {
    let score = |dev: &Device| dev.max_x() * dev.max_y();
    enumerate()
        .into_iter()
        .filter(|dev| dev.is_multitouch())
        .max_by(|a, b| score(a).cmp(&score(b)))
        .and_then(|dev| Some(MTDevice::new(dev)))
}

fn enumerate() -> Vec<Device> {
    let mut res = Vec::new();
    if let Ok(dir) = read_dir("/dev/input") {
        for entry in dir {
            if let Ok(entry) = entry {
                let file_type = entry.file_type().unwrap();
                if file_type.is_block_device() || file_type.is_char_device() {
                    if let Ok(dev) = Device::open(&entry.path()) {
                        res.push(dev)
                    }
                }
            }
        }
    }
    res
}
