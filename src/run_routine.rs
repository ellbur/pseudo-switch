
use std::{path::PathBuf, str::FromStr};
use inotify::{Inotify, WatchMask};
use std::sync::mpsc;

fn watch_for_changes<Cb: FnMut(bool)>(device: &str, mut cb: Cb) {
  let path = PathBuf::from_str(device).unwrap();
  let parent = path.parent().unwrap().to_owned();
  let name = path.file_name().unwrap().to_owned();

  let mut inotify = Inotify::init().expect("Error initializing");
  inotify.watches().add(parent, WatchMask::CREATE | WatchMask::ATTRIB | WatchMask::DELETE)
    .expect("Failed to add watch");

  let mut exists = None;

  loop {
    let new_exists = path.exists();
    if Some(new_exists) != exists {
      exists = Some(new_exists);
      cb(new_exists);
    }

    'checking: loop {
      let mut buffer = [0; 1024];
      let events = inotify.read_events_blocking(&mut buffer).expect("Error reading events");

      for event in events {
        if let Some(event_name) = event.name {
          if event_name == name {
            break 'checking;
          }
        }
      }
    }
  }
}

pub fn run(device: &str, hysteresis: Option<f64>) {
  watch_for_changes(device, |new_state| {
    println!("{}", new_state);
  });
}
