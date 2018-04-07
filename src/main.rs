extern crate crypto;
extern crate getopts;
extern crate regex;
extern crate walkdir;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

mod checksum;
mod op;
mod op_acknowledge;
mod op_diff;
mod op_fsck;
mod op_history;
mod op_init;
mod index;
mod index_diff;
mod index_scan;

use std::env;
use std::io::Write;
use op::*;
use index::*;

type Error = String;

const VERSION : &'static str = env!("CARGO_PKG_VERSION");
const DEFAULT_DATA_DIR : &'static str = ".";
const DEFAULT_INDEX_DIR : &'static str = ".fhistory";
const USAGE : &'static str = "\
usage: fhistory <command> [options]
Another file integrity monitoring tool.

global options:
  -d,--data_dir=PATH     Set the path of the repository/data directory
  -x,--index_dir=PATH    Set the path of the history/index directory
  --help=PATH            Print the help message for one of the commands and exit

commands:
  diff      Compare the current state of the repository to a snapshot (quick)
  ack       Acknowledge changes to files in the repository and create a new snapshot
  log       Display a historical log of snapshots and changes to the repository
  fsck      Perform a full check of the repository's integrity
  version   Print the version of this program and exit
  help      Print the help message for one of the commands and exit
";

#[derive(Debug)]
enum Command {
  PrintUsage{ topic: Option<Operation> },
  PrintVersion,
  Operation{ op: Operation, args: Vec<String> }
}

fn perform_op(op: Operation, args: &Vec<String>) -> Result<bool, Error> {
  return match op {
    Operation::Acknowledge => op_acknowledge::perform(args),
    Operation::Diff => op_diff::perform(args),
    Operation::Fsck => op_fsck::perform(args),
    Operation::History => op_history::perform(args),
    Operation::Initialize => op_init::perform(args),
  };
}

fn print_usage(op: Option<Operation>) -> Result<bool, Error> {
  let usage_msg = match op {
    Some(Operation::Acknowledge) => op_acknowledge::USAGE,
    Some(Operation::Diff) => op_diff::USAGE,
    Some(Operation::Fsck) => op_fsck::USAGE,
    Some(Operation::History) => op_history::USAGE,
    Some(Operation::Initialize) => op_init::USAGE,
    None => USAGE,
  };

  match std::io::stdout().write(usage_msg.as_bytes()) {
    Err(e) => Err(e.to_string()),
    Ok(_) => Ok(true)
  }
}

fn print_version() -> Result<bool, Error> {
  println!("fhistory v{}", VERSION);
  println!("Copyright (c) 2018 Paul Asmuth");
  println!("Licensed under the 3-clause BSD license");
  println!("https://github.com/asmuth/fhistory");
  return Ok(true);
}

fn main() {
  let args : Vec<String> = env::args().collect();
  let argsr : Vec<&str> = args.iter().map(|s| s.as_ref()).collect();

  let command = match argsr.get(1) {
    Some(&"version") =>
      Command::PrintVersion{},
    Some(&"help") =>
      match argsr.get(2) {
        Some(topic) =>
          Command::PrintUsage{ topic: Operation::from_str(topic) },
        None =>
          Command::PrintUsage{ topic: None },
      }
    Some(cmd) =>
      match Operation::from_str(cmd) {
        Some(op) =>
          if argsr[2..].iter().any(|x| *x == "--help") {
            Command::PrintUsage{ topic: Some(op) }
          } else {
            Command::Operation{ op: op, args: args[2..].to_vec() }
          },
        _ =>
          Command::PrintUsage{ topic: None },
      },
    _ =>
      Command::PrintUsage{ topic: None },
  };

  let result = match command {
    Command::PrintUsage{topic} => print_usage(topic),
    Command::PrintVersion => print_version(),
    Command::Operation{op, args} => perform_op(op, &args),
  };

  match result {
    Ok(true) => return,
    Ok(false) => std::process::exit(1),
    Err(e) => {
      writeln!(&mut std::io::stderr(), "ERROR: {}", e).expect("ERROR");
      std::process::exit(1);
    }
  }
}
