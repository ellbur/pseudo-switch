
mod device_listing;
mod device_monitoring;
mod identify_routine;
mod run_routine;
mod synthetic_switch;
mod simulate_switch_routine;
mod struct_ser;
mod simulate_once_routine;
mod systemd_utils;

use std::{path::Path, str::FromStr};

use clap::{Parser, Subcommand};
use tabled::{Tabled, Table, settings::Style};

#[derive(Tabled)]
struct ListedDevice {
  name: String,
  path: String,
  by_path_path: String,
}

#[derive(Parser)]
struct Cli {
  #[command(subcommand)]
  command: Command,
}

#[derive(Subcommand)]
enum Command {
  ListInputDevices,
  IdentifyDetachableDevices,
  SimulatePeriodically {
    #[arg(long)]
    interval: Option<f64>
  },
  SimulateOnce {
    #[arg(value_enum, value_name = "on|off")]
    state: State
  },
  Run {
    device: String,
    #[arg(long)]
    hysteresis: Option<f64>
  },
  AddSystemdService {
    device: String,
    #[arg(long)]
    hysteresis: Option<f64>
  },
}

#[derive(Clone, Debug)]
enum State {
  On,
  Off
}

impl FromStr for State {
  type Err = String;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let lc = s.to_lowercase();
    if lc == "off" || lc == "false" || lc == "0" || lc == "no" {
      Ok(State::Off)
    }
    else if lc == "on" || lc == "true" || lc == "1" || lc == "yes" {
      Ok(State::On)
    }
    else {
      Err("Should be 'on' or 'off'".to_owned())
    }
  }
}

fn main() {
  let cli = Cli::parse();

  match cli.command {
    Command::ListInputDevices => {
      let listed_devices: Vec<ListedDevice> = device_listing::list_input_devices().unwrap().into_iter().map(|d| {
        ListedDevice {
          name: d.name,
          path: d.dev_path.to_string_lossy().to_string(),
          by_path_path: d.by_path_path.map(|p| p.to_string_lossy().to_string()).unwrap_or("".to_string())
        }
      }).collect();
      println!("{}", Table::new(listed_devices).with(Style::blank()).to_string());
    },
    Command::IdentifyDetachableDevices => {
      identify_routine::run();
    },
    Command::Run { device, hysteresis } => {
      run_routine::run(device, hysteresis);
    },
    Command::SimulatePeriodically { interval } => {
      simulate_switch_routine::run(interval.unwrap_or(5.0));
    },
    Command::SimulateOnce { state } => {
      simulate_once_routine::run(match state { State::On => true, State::Off => false });
    },
    Command::AddSystemdService { device, hysteresis } => {
      systemd_utils::add_and_start_systemd_service(Path::new(&device), hysteresis).unwrap();
    }
  }
}

