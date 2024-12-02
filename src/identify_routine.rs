
use std::collections::HashMap;
use std::{collections::HashSet, path::PathBuf};
use std::io::{self, Write};
use crate::device_listing::{ExtractedInputDevice, list_input_devices};
use crossterm::cursor::{RestorePosition, SavePosition};
use crossterm::terminal::Clear;
use crossterm::QueueableCommand;
use tabled::grid::config::Entity;
use tabled::settings::object::{ObjectIterator, Rows};
use tabled::settings::{Color, Modify};
use tabled::{Tabled, Table, settings::Style};

#[derive(Tabled)]
struct ListedDevice {
  present: String,
  name: String,
  path: String,
  by_path_path: String,
  changed: String,
}

pub fn run() {
  let mut stdout = io::stdout();

  let mut working_devices: Vec<ExtractedInputDevice> = vec![];
  let mut all_ever_devices: HashMap<PathBuf, ExtractedInputDevice> = HashMap::new();
  let mut all_ever_changed: HashSet<PathBuf> = HashSet::new();
  let mut is_first = true;

  crate::device_monitoring::monitor_devices(move || {
    let new_listing = list_input_devices().unwrap();

    let new_paths: HashSet<PathBuf> = new_listing.iter().map(|d| d.dev_path.clone()).collect();
    let old_paths: HashSet<PathBuf> = working_devices.iter().map(|d| d.dev_path.clone()).collect();

    let removed_paths: HashSet<PathBuf> = old_paths.difference(&new_paths).map(|p|p.clone()).collect();
    let added_paths: HashSet<PathBuf> = new_paths.difference(&old_paths).map(|p|p.clone()).collect();
    
    for d in new_listing.iter() {
      all_ever_devices.insert(d.dev_path.clone(), d.clone());
    }

    if !is_first {
      for p in added_paths.iter().chain(removed_paths.iter()) {
        all_ever_changed.insert(p.clone());
      }
    }

    let mut rows: Vec<ExtractedInputDevice> = all_ever_devices.values().map(|d| d.clone()).collect();
    rows.sort_by_key(|d| d.dev_path.clone());
    let rows = rows;

    let listing: Vec<ListedDevice> = rows.iter().map(|d| {
      ListedDevice {
        present: if new_paths.contains(&d.dev_path) { "*".to_owned() } else { " ".to_owned() },
        name: d.name.clone(),
        path: d.dev_path.to_string_lossy().to_string(),
        by_path_path: d.by_path_path.clone().map(|p| p.to_string_lossy().to_string()).unwrap_or("".to_string()),
        changed: if all_ever_changed.contains(&d.dev_path) { "*".to_owned() } else { " ".to_owned() },
      }
    }).collect();

    let present_row_set: HashSet<usize> = rows.iter().enumerate()
      .filter(|(_i, d)| new_paths.contains(&d.dev_path)).map(|(i, _d)| i+1).collect();

    stdout.queue(Clear(crossterm::terminal::ClearType::All)).unwrap();
    stdout.queue(SavePosition { }).unwrap();

    println!("{}",
      Table::new(listing)
        .with(Style::blank())
        .with(Modify::new(Rows::new(1 .. rows.len()+1).filter(|entity| match entity {
          Entity::Row(n) => present_row_set.contains(&n),
          _ => false
        })).with(Color::FG_BRIGHT_WHITE | Color::BOLD))
        .to_string()
    );

    stdout.queue(RestorePosition).unwrap();
    stdout.flush().unwrap();

    working_devices = new_listing;
    is_first = false;
  }).unwrap();
}

