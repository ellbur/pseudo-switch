
// vim: shiftwidth=2

use std::fs::{File, read_to_string};
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};

struct ExtractedProcBusInputDevice {
  sysfs_path: String,
  name: String
}

pub struct ExtractedInputDevice {
  pub dev_path: PathBuf,
  pub name: String
}

fn extract_input_devices_from_proc_bus_input_devices(proc_bus_input_devices: &str) -> Vec<ExtractedProcBusInputDevice> {
  let mut res = Vec::new();
  let lines = proc_bus_input_devices.split('\n');
  
  let mut working_sysfs_path = Box::new(None);
  let mut working_name = Box::new(None);
  let mut working_ev_mask = Box::new(None);
  
  for line in lines {
    if line.starts_with("I:") {
      *working_sysfs_path = None;
      *working_name = None;
      *working_ev_mask = None;
    }
    else if line.starts_with("S: Sysfs=") {
      let new_sysfs_path = line[9..].to_string();
      *working_sysfs_path = Some(new_sysfs_path);
    }
    else if line.starts_with("N: Name=\"") {
      let mut name = line[9..].to_string();
      name = name.trim_end().to_string();
      if name.ends_with('"') {
        name = name[..name.len()-1].to_string();
      }
      *working_name = Some(name);
    }
    else if line.starts_with("B: EV=") {
      *working_ev_mask = Some(line[6..].to_string());
    }
    else if line.trim().is_empty() {
      let name = match &*working_name {
        None => "".to_string(),
        Some(name) => name.clone()
      };
      
      match &*working_sysfs_path {
        None => (),
        Some(p) => {
          res.push(ExtractedProcBusInputDevice {
            sysfs_path: p.to_string(),
            name
          });
        }
      }
    }
  }
  
  res
}

pub fn list_input_devices() -> io::Result<Vec<ExtractedInputDevice>> {
  let mut res = Vec::new();
  
  let proc_bus_input_devices = read_to_string("/proc/bus/input/devices")?;
  let extracted = extract_input_devices_from_proc_bus_input_devices(&proc_bus_input_devices);
  
  for dev in extracted {
    let p = dev.sysfs_path;
    if !p.starts_with("/devices/virtual/input/") {
      match dev_path_for_sysfs_name(&p)? {
        None => (),
        Some(dev_path) => {
          res.push(ExtractedInputDevice {
            dev_path,
            name: dev.name
          });
        }
      }
    }
  }
  
  Ok(res)
}

fn dev_path_for_sysfs_name(sysfs_name: &String) -> io::Result<Option<PathBuf>> {
  let mut sysfs_path = "/sys".to_string();
  sysfs_path.push_str(sysfs_name);

  for _entry in Path::new(&sysfs_path).read_dir()? {
    let entry = _entry?;
    let path = entry.path();
    match path.file_name() {
      None => (),
      Some(_name) => {
        let name = _name.to_string_lossy();
        if name.starts_with("event") {
          let mut uevent_path = path.clone();
          uevent_path.push("uevent");
          for _line in io::BufReader::new(File::open(uevent_path)?).lines() {
            let line = _line?;
            if line.starts_with("DEVNAME=") {
              let dev_name = line[8..].to_string();
              let mut dev_path = PathBuf::new();
              dev_path.push("/dev");
              dev_path.push(dev_name);
              return Ok(Some(dev_path));
            }
          }
        }
      }
    }
  }
  
  Ok(None)
}

