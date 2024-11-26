
// vim: shiftwidth=2

use inotify::{Inotify, WatchMask};

pub fn monitor_devices<Cb: FnMut()>(mut cb: Cb) -> Result<(), String> {
  let mut inotify = Inotify::init().expect("Error initializing");
  inotify.watches().add("/dev/input", WatchMask::CREATE | WatchMask::ATTRIB | WatchMask::DELETE)
    .expect("Failed to add watch");

  loop {
    cb();

    let mut buffer = [0; 1024];
    inotify.read_events_blocking(&mut buffer).expect("Error reading events");
  }
}

