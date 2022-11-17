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

enum CmdOption {
     HELP,
     ScriptFile(String),
     VERSION,

}

fn parse_command(args: &Vec<String>) -> (Vec<CmdOption>, Vec<&String>, Vec<&String>) {
     let (mut options, mut targets, mut run_args) = (Vec::new(), Vec::new(), Vec::new());
     let mut arg_n = 0;
     let mut mode_self = true;
     while arg_n < args.len() {
         let arg = &args[arg_n] ;
         //println!("analizing {}", arg);
         if mode_self {
            if arg.starts_with("-h") {
               options.push(CmdOption::HELP);
            } else if arg.starts_with("-f") {

            } else if arg.starts_with("-v") {
               options.push(CmdOption::VERSION);
            } else if arg == "--" {
               mode_self = false;
            }
            
         } else {
          run_args.push(arg);
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
         
          match opt {
               VERSION => {
                    let (ver, build, date) = ver::version();
                    println!("RB version: {}, build: {} on {}", ver, build, date);
               },
               HELP => println!("{}", help::get_help()),
               
               _ => {}
          }
     }
     io::stdout().flush()?;
     Ok(())
}