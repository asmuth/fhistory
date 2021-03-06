/**
 * integritycheck - https://github.com/asmuth/integritycheck
 * Copyright (c) 2018, Paul Asmuth <paul@asmuth.com>
 *
 * This file is part of the "integritycheck" project. integritycheck is free software
 * licensed under the Apache License, Version 2.0 (the "License"); you may not
 * use this file except in compliance with the License.
 */
use std;
use std::io::Write;
use colored;
use colored::*;
use libc;
use time;

#[allow(non_upper_case_globals)]
static mut enable_progress : bool = false;

#[allow(non_upper_case_globals)]
static mut enable_debug : bool = false;

pub fn set_progress(opt: bool) {
  unsafe {
    enable_progress = opt;
  }
}

pub fn set_debug(opt: bool) {
  unsafe {
    enable_debug = opt;
  }
}

pub fn set_colours(opt: bool) {
  colored::control::set_override(opt);
}

pub fn print_progress_step(step: u32, steps_total: u32, msg: &str) {
  unsafe {
    if !enable_progress {
      return;
    }
  }

  let res = writeln!(
      &mut std::io::stderr(),
      "{} {}",
      format!("[{}/{}]", step, steps_total).white().dimmed(),
      msg);

  res.expect("cannot write to stderr");
}

pub fn print_progress_complete() {
  unsafe {
    if !enable_progress {
      return;
    }
  }

  writeln!(&mut std::io::stderr(), "").expect("cannot write to stderr");
}

pub fn print_debug(msg: &str) {
  unsafe {
    if !enable_debug {
      return;
    }
  }

  let res = writeln!(
      &mut std::io::stderr(),
      "{} {}",
      "DEBUG".white().dimmed(),
      msg);

  res.expect("cannot write to stderr");
}

pub fn print_success(msg: &str) {
  println!("{}", msg.green());
}

pub fn print_repository_path(path: &str) {
  if let Ok(path) = std::fs::canonicalize(std::path::Path::new(&path)) {
    println!("Repository: {}", path.to_str().unwrap_or("ERROR"));
  } else {
    println!("Repository: {}", path);
  }
}

pub fn print_repository_size(snap: &::IndexSnapshot) {
  writeln!(
      &mut std::io::stderr(),
      "Total Size: {} ({} files)",
      format_bytecount(snap.total_size_bytes()),
      snap.total_file_count());
}

pub fn print_repository_status(status: bool) {
  println!(
      "Status: {}",
      if status { "CLEAN".green() } else { "DIRTY".red() });
}

pub fn print_snapshot_time(timestamp_us: i64) {
  let time = time::at(time::Timespec::new(timestamp_us / 1_000_000, 0));
  println!("Last Snapshot: {}", time.rfc822z());
}

pub fn print_diff(diff: &::index_diff::IndexDiffList) {
  let mut diff = diff.to_owned();
  if diff.len() == 0 {
    return;
  }

  let sort_name = |d: &::index_diff::IndexDiff| match d {
    &::index_diff::IndexDiff::Deleted{ref file} => file.to_owned(),
    &::index_diff::IndexDiff::Modified{ref file} => file.to_owned(),
    &::index_diff::IndexDiff::MetadataModified{ref file} => file.to_owned(),
    &::index_diff::IndexDiff::Renamed{ref from, ..} => from.to_owned(),
    &::index_diff::IndexDiff::Created{ref file} => file.to_owned(),
  };

  diff.sort_by(|a, b| sort_name(&a).cmp(&sort_name(&b)));

  let sort_rank = |d: &::index_diff::IndexDiff| match d {
    &::index_diff::IndexDiff::Deleted{..} => 1,
    &::index_diff::IndexDiff::Modified{..} => 2,
    &::index_diff::IndexDiff::MetadataModified{..} => 2,
    &::index_diff::IndexDiff::Renamed{..} => 3,
    &::index_diff::IndexDiff::Created{..} => 4,
  };

  diff.sort_by(|a, b| sort_rank(&a).cmp(&sort_rank(&b)));

  print!("\n");

  for d in diff {
    let msg = match d {
      ::index_diff::IndexDiff::Created{ref file} =>
       format!("    created  {:?}", file).green(),
      ::index_diff::IndexDiff::Deleted{ref file} =>
       format!("    deleted  {:?}", file).red(),
      ::index_diff::IndexDiff::Modified{ref file} =>
       format!("    modified {:?}", file).yellow(),
      ::index_diff::IndexDiff::MetadataModified{ref file} =>
       format!("    modified {:?} (metadata modifications only)", file).yellow(),
      ::index_diff::IndexDiff::Renamed{ref from, ref to} =>
        format!("    renamed  {:?} -> {:?}", from, to).yellow()
    };

    println!("{}", msg);
  }

  print!("\n");
}

pub fn confirm_diffs(diff: &::index_diff::IndexDiffList) -> bool {
  println!("Acknowledging {} changes:", diff.len());
  print_diff(diff);
  print!("Apply changes? (y/n): ");
  std::io::stdout().flush().ok().expect("Could not flush stdout");

  let resp : i32;
  unsafe {
    resp = libc::getchar();
  };

  return match resp {
    121 => true,
    _ => false,
  };
}

pub fn print_confirmed_diffs(diff: &::index_diff::IndexDiffList) {
  println!("Changes ({})", diff.len());
  print_diff(diff);
}

pub fn print_snapshot_table(index: &::IndexDirectory) -> Result<(), ::Error> {
  for snap_ref in index.list() {
    println!("{}", format!("snapshot {}", snap_ref.checksum).yellow());

    let snap = index.load(snap_ref)?;
    let snap_time = time::at(time::Timespec::new(snap_ref.timestamp_us / 1_000_000, 0));

    println!("Timestamp: {}", snap_time.rfc822z());
    println!(
        "Size: {} ({} files)",
        format_bytecount(snap.total_size_bytes()),
        snap.total_file_count());

    println!("\n    {}\n", snap.message.unwrap_or("<no message>".into()));
  }

  return Ok(());
}

fn format_bytecount(val: u64) -> String {
  if val < u64::pow(2, 10) {
    return format!("{}B", val);
  } else if val < u64::pow(2, 20) {
    return format!("{:.3}KiB", val as f64 / u64::pow(2, 10) as f64)
  } else if val < u64::pow(2, 30) {
    return format!("{:.3}MiB", val as f64 / u64::pow(2, 20) as f64)
  } else if val < u64::pow(2, 40) {
    return format!("{:.3}GiB", val as f64 / u64::pow(2, 30) as f64)
  } else {
    return format!("{:.3}TiB", val as f64 / u64::pow(2, 40) as f64)
  }
}

pub fn print_scanprogress(
    files_scanned: u64,
    bytes_scanned: u64,
    files_total: u64,
    bytes_total: u64) {
  unsafe {
    if !enable_progress || enable_debug {
      return;
    }
  }

  let statusline = if files_total == 0 || bytes_total == 0 {
    format!(
        "\x1B\r[2K> {} files, {}",
        files_scanned,
        format_bytecount(bytes_scanned))
  } else {
    format!(
        "\x1B\r[2K> {:.1}%, {} / {} files, {} / {}",
        (bytes_scanned as f64 / bytes_total as f64) * 100.0,
        files_scanned,
        files_total,
        format_bytecount(bytes_scanned),
        format_bytecount(bytes_total))
  };

  write!(&mut std::io::stderr(), "{}", statusline)
      .expect("Could not write to stderr");

  std::io::stderr()
      .flush()
      .ok()
      .expect("Could not flush stdout");
}

pub fn print_scanprogress_complete() {
  unsafe {
    if !enable_progress || enable_debug {
      return;
    }
  }

  write!(&mut std::io::stderr(), "\x1B\r[2K")
      .expect("Could not write to stderr");

  std::io::stderr()
      .flush()
      .ok()
      .expect("Could not flush stdout");
}

