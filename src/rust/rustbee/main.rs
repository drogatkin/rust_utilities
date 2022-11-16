use std::fs;
use std::env;
use std::path::Path;
use std::path::PathBuf;
use std::io::{self, Write, prelude::*, SeekFrom};
use std::fs::File;
use std::str;
use std::io::{Error, ErrorKind};

mod help;

enum CmdOption {
     HELP,
     ScriptFile(String),
     VERSION,

}

fn parse_command(args: &Vec<String>) -> (Vec<CmdOption>, Vec<String>, Vec<String>) {
     let mut result = (Vec::new(), Vec::new(), Vec::new());
     let mut arg_n = 0;
     while arg_n < args.len() {
         let arg = &args[arg_n] ;
         //println!("analizing {}", arg);
         if arg.starts_with("-h") {
            result.0.push(CmdOption::HELP);
         } else if arg.starts_with("-v") {
            result.0.push(CmdOption::VERSION);
         }
         arg_n += 1;
     }
     result
}

fn main() -> io::Result<()> {
     println!("RustBee (rb) v 1.0 (c) Copyright {} D. Rogatkin", 2022);
     
     let args: Vec<String> = env::args().collect();
     let cmd = parse_command(&args);
     for opt in cmd.0 {
          match opt {
               HELP => println!("{}", help::get_help()),
               _ => {}
          }
     }
     io::stdout().flush()?;
     Ok(())
}