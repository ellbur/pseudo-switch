
mod device_listing;

use clap::{Parser, Args, Subcommand, ValueEnum};
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
      println!("TODO!");
    }
  }
}

