
// vim: shiftwidth=2

use std::io::ErrorKind;

use inotify::{Inotify, WatchMask};

pub fn monitor_devices<Cb: FnMut()>(mut cb: Cb) -> Result<(), String> {
  let mut inotify = Inotify::init().expect("Error initializing");
  inotify.watches().add("/dev/input", WatchMask::CREATE | WatchMask::ATTRIB | WatchMask::DELETE)
    .expect("Failed to add watch");

  loop {
    cb();

    let mut buffer = [0; 1024];
    inotify.read_events_blocking(&mut buffer).expect("Error reading events");
    loop {
      match inotify.read_events(&mut buffer) {
        Err(err) =>
          if err.kind() == ErrorKind::WouldBlock {
            break;
          }
          else {
            panic!("Error reading events: {}", err);
          },
        Ok(mut events) =>
          if events.next().is_none() {
            break;
          }
      }
    }
  }
}

