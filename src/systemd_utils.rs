
// vim: shiftwidth=2

use std::ffi::CString;
use std::fs::OpenOptions;
use std::io::Write;
use std::os::unix::prelude::MetadataExt;
use std::path::Path;
use std::process::Command;

pub fn add_and_start_systemd_service(device: &Path, hysteresis: Option<f64>) -> Result<(), String> {
  check_usr_bin_pseudo_switch_exists();
  create_input_group_if_necessary()?;
  create_perm_udev_rule()?;
  create_user_if_necessary()?;
  set_permissions_if_necessary()?;
  write_systemd_service(device, hysteresis)?;
  refresh_systemd()?;
  enable_systemd_service()?;
  start_systemd_service()?;
  Ok(())
}

fn find_program(cmd: &str) -> Result<String, String> {
  {
    let p = format!("/bin/{}", cmd);
    if std::fs::metadata(p.clone()).is_ok() {
      return Ok(p);
    }
  }
  
  {
    let p = format!("/sbin/{}", cmd);
    if std::fs::metadata(p.clone()).is_ok() {
      return Ok(p);
    }
  }
  
  {
    let p = format!("/usr/bin/{}", cmd);
    if std::fs::metadata(p.clone()).is_ok() {
      return Ok(p);
    }
  }

  {
    let p = format!("/usr/sbin/{}", cmd);
    if std::fs::metadata(p.clone()).is_ok() {
      return Ok(p);
    }
  }

  Err(format!("Could not find {} in /bin/, /sbin/, /usr/bin/, or /usr/sbin/", cmd))
}

fn check_usr_bin_pseudo_switch_exists() {
  if !Path::new("/usr/bin/pseudo-switch").exists() {
    eprintln!("WARNING: /usr/bin/pseudo-switch does not exist. systemd service will be unable to run until it is installed.");
  }
}

fn create_input_group_if_necessary() -> Result<(), String> {
  let input_group_exists =
    match Command::new("/usr/bin/getent").args(&["group", "input"]).output() {
      Err(e) => Err(format!("Failed to run getent: {}", e)),
      Ok(output) => {
        match output.status.code() {
          None => Err("getent terminated by signal".to_string()),
          Some(0) => Ok(true),
          Some(2) => Ok(false),
          Some(other_code) =>  Err(format!("getent returned unexpected code {}", other_code))
        }
      }
    }?;
  
  if !input_group_exists {
    match Command::new("/usr/sbin/groupadd").args(&["--system", "input"]).output() {
      Err(e) => Err(format!("Failed to run groupadd: {}", e)),
      Ok(output) => {
        match output.status.code() {
          None => Err("groupadd terminated by signal".to_string()),
          Some(0) => Ok(()),
          Some(9) => Ok(()),
          Some(other_code) => Err(format!("groupadd returned unexpected code {}", other_code))
        }
      }
    }?;
  }
  
  Ok(())
}

fn create_perm_udev_rule() -> Result<(), String> {
  if !std::fs::metadata("/etc/udev").is_ok() {
    return Err("Your system does not have /etc/udev. It is likely your system does not use udev. Cannot create needed udev rules.".to_string());
  }
  
  if !std::fs::metadata("/etc/udev/rules.d").is_ok() {
    match std::fs::create_dir("/etc/udev/rules.d") {
      Ok(_) => Ok(()),
      Err(e) => Err(format!("/etc/udev/rules.d does not exist and could not create it: {}", e))
    }?;
  }
  
  let path = "/etc/udev/rules.d/79-input.rules";
  let mut out_file = match OpenOptions::new()
    .truncate(true).read(false).create(true).write(true)
    .open(path)
  {
    Err(err) => {
      match err.kind() {
        std::io::ErrorKind::PermissionDenied => {
          return Err(format!("Permission denied writing to {}. You likely must run this sub-command as root.", path));
        },
        _ => return Err(format!("Error writing to {}: {}", path, err))
      }
    },
    Ok(out_file) => out_file
  };
  
  match out_file.write(
    "KERNEL==\"uinput\", MODE=\"0660\", GROUP=\"input\", OPTIONS+=\"static_node=uinput\"\n\
     SUBSYSTEM==\"misc\", KERNEL==\"uinput\", MODE=\"0660\", GROUP=\"input\"".as_bytes()
  ) {
    Err(err) => return Err(format!("{}", err)),
    Ok(_) => ()
  };
  
  Ok(())
}

fn set_permissions_if_necessary() -> Result<(), String> {
  let stat = match std::fs::metadata("/dev/uinput") {
    Err(e) => Err(format!("Could not stat /dev/uinput: {}", e)),
    Ok(meta) => Ok(meta)
  }?;
  
  let gid = stat.gid();
  
  let input_gid = unsafe {
    let c_str = CString::new("input").unwrap();
    (*libc::getgrnam(c_str.as_ptr())).gr_gid
  };
  
  if gid != input_gid {
    match Command::new("/usr/bin/chown").args(&["root:input", "/dev/uinput"]).output() {
      Err(e) => Err(format!("Failed to run /usr/bin/chown: {}", e)),
      Ok(_) => Ok(())
    }?;
  }
  
  let mode = stat.mode();
  let group_readable = mode & 0o040;
  let group_writable = mode & 0o020;
  
  if (group_readable == 0) || (group_writable == 0) {
    match Command::new("/usr/bin/chmod").args(&["g+rw", "/dev/uinput"]).output() {
      Err(e) => Err(format!("Failed to run /usr/bin/chmod: {}", e)),
      Ok(_) => Ok(())
    }?;
  }
  
  Ok(())
}

fn create_user_if_necessary() -> Result<(), String> {
  let user_exists =
    match Command::new("/usr/bin/id").args(&["-u", "pseudo-switch"]).output() {
      Err(e) => Err(format!("Failed to run /usr/bin/id: {}", e)),
      Ok(output) => {
        match output.status.code() {
          None => Err("id terminated by signal".to_string()),
          Some(0) => Ok(true),
          Some(1) => Ok(false),
          Some(other_code) => Err(format!("/usr/bin/id returned unexpected code {}", other_code))
        }
      }
    }?;

  if !user_exists {
    // On Debian systems, this is needed to correctly create a system user
    if Path::new("/usr/sbin/adduser").exists() {
      match Command::new("/usr/sbin/adduser").args(&["--system", "--no-create-home", "pseudo-switch"]).output() {
        Err(e) => Err(format!("Failed to run /usr/sbin/adduser: {}", e)),
        Ok(output) => {
          match output.status.code() {
            None => Err("adduser terminated by signal".to_string()),
            Some(0) => Ok(()),
            Some(9) => Ok(()),
            Some(other_code) => Err(format!("/usr/sbin/adduser returned unexpected code {}", other_code))
          }
        }
      }?;
    }
    else {
      match Command::new("/usr/sbin/useradd").args(&["--system", "--no-create-home", "pseudo-switch"]).output() {
        Err(e) => Err(format!("Failed to run /usr/sbin/useradd: {}", e)),
        Ok(output) => {
          match output.status.code() {
            None => Err("useradd terminated by signal".to_string()),
            Some(0) => Ok(()),
            Some(9) => Ok(()),
            Some(other_code) => Err(format!("/usr/sbin/useradd returned unexpected code {}", other_code))
          }
        }
      }?;
    }
  }
  
  match Command::new("/usr/sbin/usermod").args(&["-a", "-G", "input", "pseudo-switch"]).output() {
    Err(e) => Err(format!("Failed to run usermod: {}", e)),
    Ok(output) => {
      match output.status.code() {
        None => Err("usermod terminated by signal".to_string()),
        Some(0) => Ok(()),
        Some(other_code) => Err(format!("usermod returned unexpected code {}", other_code))
      }
    }
  }?;
  
  Ok(())
}

fn write_systemd_service(device: &Path, hysteresis: Option<f64>) -> Result<(), String> {
  let path = "/etc/systemd/system/pseudo-switch.service";
  let mut out_file = match OpenOptions::new()
    .truncate(true).read(false).create(true).write(true)
    .open(path)
  {
    Err(err) => {
      match err.kind() {
        std::io::ErrorKind::PermissionDenied => {
          return Err(format!("Permission denied writing to {}. You likely must run this sub-command as root.", path));
        },
        _ => return Err(format!("{}", err))
      }
    },
    Ok(out_file) => out_file
  };
   
  match out_file.write(build_service_text(device, hysteresis)?.as_bytes()) {
    Err(err) => return Err(format!("{}", err)),
    Ok(_) => ()
  };
  
  Ok(())
}

fn build_service_text(device: &Path, hysteresis: Option<f64>) -> Result<String, String> {
  let hysteresis_text = match hysteresis {
    None => "".to_owned(),
    Some(hysteresis) => format!("--hysteresis {}", hysteresis)
  };

  Ok(format!(
    "[Unit]\n\
     Description=pseudo-switch\n\
     \n\
     [Install]\n\
     WantedBy=multi-user.target\n\
     \n\
     [Service]\n\
     Type=simple\n\
     User=pseudo-switch\n\
     Group=input\n\
     ExecStart=/usr/bin/pseudo-switch run {} \"{}\"\n",
    hysteresis_text,
    systemd_arg_escape(device.to_str().ok_or("Device path is not formattable".to_string())?)
  ))
}

fn escape_one_char(c: char) -> String {
  match c {
    '\\' => "\\\\".to_owned(),
    ' ' => "\\s".to_owned(),
    '\x07' => "\\a".to_owned(),
    '\x08' => "\\b".to_owned(),
    '\n' => "\\n".to_owned(),
    '\r' => "\\r".to_owned(),
    '\t' => "\\t".to_owned(),
    '"' => "\\\"".to_owned(),
    '\'' => "'".to_owned(),
    '*' => "\\x2a".to_owned(),
    '?' => "\\x3f".to_owned(),
    _ => {
      if c.is_control() {
        let i = c as u64;
        if i < 128 {
          format!("\\x{:0>2}", i)
        }
        else if i < 0x10000 {
          format!("\\u{:0>4}", i)
        }
        else {
          format!("\\U{:0>8}", i)
        }
      }
      else {
        format!("{}", c)
      }
    }
  }
}

fn systemd_arg_escape(text: &str) -> String {
  let mut res = Vec::new();
  for c in text.chars() {
    res.extend(escape_one_char(c).chars());
  }
  res.iter().collect()
}

fn refresh_systemd() -> Result<(), String> {
  match Command::new(find_program("systemctl")?).args(&["daemon-reload"]).status() {
    Err(e) => Err(format!("Failed to reload systemd: {}", e)),
    Ok(_) => Ok(())
  }?;
  
  Ok(())
}

fn enable_systemd_service() -> Result<(), String> {
  match Command::new(find_program("systemctl")?).args(&["enable", "pseudo-switch.service"]).status() {
    Err(e) => {
      Err(format!("Failed to run systemctl: {}", e))
    },
    Ok(c) => {
      if let Some(code) = c.code() {
        if code == 4 {
          Err("Permission denied running systemctl. Likely you need to run this as root.".to_string())
        }
        else if code == 0 {
          Ok(())
        }
        else {
          Err(format!("systemctl failed with code {}", code))
        }
      }
      else {
        Err("systemctl terminated by signal".to_string())
      }
    }
  }?;
  Ok(())
}

fn start_systemd_service() -> Result<(), String> {
  match Command::new(find_program("systemctl")?).args(&["start", "pseudo-switch.service"]).status() {
    Err(e) => {
      Err(format!("Failed to run systemctl: {}", e))
    },
    Ok(c) => {
      if let Some(code) = c.code() {
        if code == 4 {
          Err("Permission denied running systemctl. Likely you need to run this as root.".to_string())
        }
        else if code == 0 {
          Ok(())
        }
        else {
          Err(format!("systemctl failed with code {}", code))
        }
      }
      else {
        Err("systemctl terminated by signal".to_string())
      }
    }
  }?;
  Ok(())
}

