use std::fs;
use std::env;
use std::path::Path;
use std::path::PathBuf;
use std::io::{self, Write, prelude::*, SeekFrom};
use std::fs::File;
use std::str;
use std::io::{Error, ErrorKind};

mod help;
mod ver;

#[derive(Debug)]
enum CmdOption {
     HELP,
     ScriptFile(String),
     VERSION,

}

fn parse_command(args: &Vec<String>) -> (Vec<CmdOption>, Vec<&String>, Vec<String>) {
     let (mut options, mut targets, mut run_args) = (Vec::new(), Vec::new(), Vec::new());
     let mut arg_n = 0;
     while arg_n < args.len() {
         let arg = &args[arg_n] ;
         //println!("analizing {}", arg);
          if arg.starts_with("-h") {
              options.push(CmdOption::HELP);
          } else if arg.starts_with("-f") {
               arg_n += 1;
               if arg_n < args.len() {
                    options.push(CmdOption::ScriptFile(args[arg_n].to_string()));
               } else {
                    println!("No file path specified in -f option");
               }
          } else if arg.starts_with("-v") {
            options.push(CmdOption::VERSION);
          } else if arg == "--" {
               arg_n += 1;
               if arg_n < args.len() {
                    run_args.extend_from_slice( &args[arg_n..]);
               }
          }
         
         arg_n += 1;
     }
     (options, targets, run_args)
}

fn main() -> io::Result<()> {
     println!("RustBee (rb) v 1.0 D. Rogatkin (c) Copyright {}", 2022);
     
     let args: Vec<String> = env::args().collect();
     let (options, targets, run_args) = parse_command(&args);
     for opt in options {
          //println!("{:?}", opt);
          match opt {
               CmdOption::VERSION => {
                    let (ver, build, date) = ver::version();
                    println!("RB version: {}, build: {} on {}", ver, build, date);
               },
               CmdOption::HELP => println!("{}", help::get_help()),
               CmdOption::ScriptFile(file) => {
                    println!("Script {}", file);
               },
               _ => {}
          }
     }
     io::stdout().flush()?;
     Ok(())
}