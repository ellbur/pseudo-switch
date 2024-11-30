
mod device_listing;
mod device_monitoring;
mod identify_routine;
mod run_routine;
mod synthetic_switch;
mod simulate_switch_routine;
mod struct_ser;

use clap::{Parser, Subcommand};
use tabled::{Tabled, Table, settings::Style};

#[derive(Tabled)]
struct ListedDevice {
  name: String,
  path: String
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
  SimulateSwitch {
    #[arg(long)]
    interval: Option<f64>
  },
  Run {
    device: String,
    #[arg(long)]
    hysteresis: Option<f64>
  },
}

fn main() {
  let cli = Cli::parse();

  match cli.command {
    Command::ListInputDevices => {
      let listed_devices: Vec<ListedDevice> = device_listing::list_input_devices().unwrap().into_iter().map(|d| {
        ListedDevice {
          name: d.name,
          path: d.dev_path.to_string_lossy().to_string()
        }
      }).collect();
      println!("{}", Table::new(listed_devices).with(Style::blank()).to_string());
    },
    Command::IdentifyDetachableDevices => {
      identify_routine::run();
    },
    Command::Run { device, hysteresis } => {
      run_routine::run(&device, hysteresis);
    },
    Command::SimulateSwitch { interval } => {
      simulate_switch_routine::run(interval.unwrap_or(5.0));
    }
  }
}

