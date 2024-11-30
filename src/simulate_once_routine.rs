
pub fn run(state: bool) {
  crate::synthetic_switch::SyntheticTabletSwitch::new().unwrap().write(state).unwrap();
}

