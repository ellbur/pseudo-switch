
use std::{path::PathBuf, str::FromStr, sync::mpsc::RecvTimeoutError, time::Duration};
use inotify::{Inotify, WatchMask};
use std::sync::mpsc;
use crate::synthetic_switch::SyntheticTabletSwitch;

fn watch_for_changes<Cb: FnMut(bool)>(device: &str, mut cb: Cb) {
  let path = PathBuf::from_str(device).unwrap();
  let parent = path.parent().unwrap().to_owned();

  let name = path.file_name().unwrap().to_owned();

  let mut inotify = Inotify::init().expect("Error initializing");
  inotify.watches().add(parent, WatchMask::CREATE | WatchMask::ATTRIB | WatchMask::DELETE | WatchMask::MOVED_TO)
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

pub fn run(device: String, hysteresis: Option<f64>) {
  let mut sw = SyntheticTabletSwitch::new().unwrap();

  let (send, receive) = mpsc::channel();

  std::thread::spawn(move || {
    watch_for_changes(&device, |new_state| {
      send.send(new_state).unwrap();
    });
  });

  let timeout = hysteresis.map(|s| Duration::from_secs_f64(s));

  let mut last_sent: Option<bool> = None;
  let mut pending_new_exists: Option<bool> = None;

  loop {
    let new_exists = match timeout {
      Some(timeout) => receive.recv_timeout(timeout),
      None => receive.recv().map_err(|_| RecvTimeoutError::Disconnected)
    };

    match new_exists {
      Ok(new_exists) => {
        if timeout.is_none() {
          if Some(new_exists) != last_sent {
            println!("Setting switch to {}", if new_exists { "ON" } else { "OFF" });
            if let Err(write_err) = sw.write(new_exists) {
              println!("Error writing switch state: {}", write_err);
            }
            last_sent = Some(new_exists);
          }
        }
        else {
          pending_new_exists = Some(new_exists);
        }
      },
      Err(RecvTimeoutError::Timeout) => {
        if let Some(new_exists) = pending_new_exists {
          if Some(new_exists) != last_sent {
            println!("Setting switch to {}", if new_exists { "ON" } else { "OFF" });
            if let Err(write_err) = sw.write(new_exists) {
              println!("Error writing switch state: {}", write_err);
            }
            last_sent = Some(new_exists);
          }
          pending_new_exists = None;
        }
      },
      Err(RecvTimeoutError::Disconnected) => {
        break;
      }
    }
  }
}
