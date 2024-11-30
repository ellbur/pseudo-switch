
use crate::synthetic_switch::SyntheticTabletSwitch;
use std::thread::sleep;
use std::time::Duration;

pub fn run(interval: f64) {
  let mut dev = SyntheticTabletSwitch::new().unwrap();

  let mut state = false;

  loop {
    sleep(Duration::from_secs_f64(interval));
    println!("Sending state: {}", state);
    dev.write(state).unwrap();
    state = !state;
  }
}

