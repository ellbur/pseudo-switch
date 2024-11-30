
use nix::fcntl::{open, OFlag};
use nix::sys::stat::Mode;
use uinput_sys::{ui_dev_create, ui_set_evbit, ui_set_swbit, EV_SW, SW_TABLET_MODE};
use nix::unistd::write;
use crate::struct_ser::StructSerializer;
use std::thread::sleep;
use std::time::Duration;

pub fn run(interval: f64) {
  let fdo = open("/dev/uinput", OFlag::O_WRONLY | OFlag::O_NONBLOCK, Mode::empty()).unwrap();
  unsafe { 
    ui_set_evbit(fdo, EV_SW);
    ui_set_swbit(fdo, SW_TABLET_MODE);
  }
  
  {
    let mut user_dev_data = StructSerializer {
      sink: Vec::new()
    };
    
    user_dev_data.add_string_in_buf("Tablet Mode Switch (Synthetic)", 80);
    
    user_dev_data.add_u16(3);
    user_dev_data.add_u16(1);
    user_dev_data.add_u16(1);
    user_dev_data.add_u16(1);
    
    user_dev_data.add_u32(0);
    
    user_dev_data.add_i32_array(&[0; 64]);
    user_dev_data.add_i32_array(&[0; 64]);
    user_dev_data.add_i32_array(&[0; 64]);
    user_dev_data.add_i32_array(&[0; 64]);
    
    write(fdo, &user_dev_data.sink).unwrap();
  }

  unsafe { ui_dev_create(fdo); }

  let mut state = false;

  loop {
    sleep(Duration::from_secs_f64(interval));

    let mut input_event_data = StructSerializer {
      sink: Vec::new()
    };
    
    let mut send_type_code_value = |type_, code, value| {
      input_event_data.add_i64(0);
      input_event_data.add_i64(0);
      input_event_data.add_u16(type_);
      input_event_data.add_u16(code);
      input_event_data.add_i32(value);
    };
    
    send_type_code_value(EV_SW as u16, SW_TABLET_MODE as u16, if state { 1 } else { 0 });
    send_type_code_value(0, 0, 0);
    
    write(fdo, &input_event_data.sink).unwrap();

    state = !state;
  }
}

