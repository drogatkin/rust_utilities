use std::fs;
use std::env;
use std::path::Path;
//use std::path::PathBuf;
use std::io::{self, Write};
// use std::fs::File;
//use std::str;
use std::io::{Error, ErrorKind};
use log::Log;
//use regex::Regex;

mod help;
mod ver;
mod log;
mod lex;

#[derive(Debug)]
enum CmdOption {
     Help,
     ScriptFile(String),
     Version,
     Verbose,
     SearchUp(String),
     PropertyFile(String),
     Diagnostics
}

fn parse_command<'a>(log: &'a Log, args: &'a Vec<String>) -> (Vec<CmdOption>, Vec<&'a String>, Vec<String>) {
     let (mut options, mut targets, mut run_args) = (Vec::new(), Vec::new(), Vec::new());
     let mut arg_n = 0;
     while arg_n < args.len() {
         let arg = &args[arg_n] ;
         //println!("analizing {}", arg);
          if arg.starts_with("-h") {
              options.push(CmdOption::Help);
          } else if arg == &"-f" || arg.starts_with("-file"){
               arg_n += 1;
               if arg_n < args.len() {
                    options.push(CmdOption::ScriptFile(args[arg_n].to_string()));
               } else {
                    println!("No file path specified in -file option");
               }
          } else if arg.starts_with("-s") || arg.starts_with("-find") {
               arg_n += 1;
               if arg_n < args.len() {
                    options.push(CmdOption::SearchUp(args[arg_n].to_string()));
               } else {
                    options.push(CmdOption::SearchUp("_".to_string()));
                    break;
               }
          } else if arg.starts_with("-version") {
            options.push(CmdOption::Version);
          } else if arg.starts_with("-v") || arg.starts_with("-verbose") {
               options.push(CmdOption::Verbose);
          } else if arg.starts_with("-d") || arg.starts_with("-diagnostic") {
               options.push(CmdOption::Diagnostics);
          } else if arg.starts_with("-xprop") || arg.starts_with("-prop") {
               arg_n += 1;
               if arg_n < args.len() {
                    options.push(CmdOption::PropertyFile(args[arg_n].to_string()));
               } else {
                   // log.log(&"Error: Property file isn't specified".to_string());
                    break;
               }
          } else if arg == "--" { 
               arg_n += 1;
               if arg_n < args.len() {
                    run_args.extend_from_slice( &args[arg_n..]);
               }
          } else {
               targets.push(arg);
          }
         
         arg_n += 1;
     }
     (options, targets, run_args)
}

fn is_bee_scrpt(file_path: &str) -> bool {
     file_path.starts_with("bee") && file_path.ends_with(".rb") 
}

fn main() -> io::Result<()> {
     println!("RustBee (rb) v 1.0 D. Rogatkin (c) Copyright {}", 2022);
     let mut log = Log {debug : false, verbose : false};
     let mut path = "_".to_string();
     let args: Vec<String> = env::args().collect();
     let (options, targets, run_args) = parse_command( &log, &args);
     for opt in options {
          //println!("{:?}", opt);
          match opt {
               CmdOption::Version => {
                    let (ver, build, date) = ver::version();
                    println!("RB Version: {}, build: {} on {}", ver, build, date);
               },
               CmdOption::Help => println!("{}", help::get_help()),
               CmdOption::Verbose => log.verbose = true,
               CmdOption::Diagnostics => log.debug = true,
               CmdOption::ScriptFile(file) => {
                    log.log(&format!("Script: {}", file));
                    
                    path = file.to_string();
               },
               CmdOption::SearchUp(file) => {
                    log.log(&format!("Search: {}", file));

               },
               _ => ()
          }
     }
     if path == "_" {
          let paths = fs::read_dir(&"./").unwrap();
          //let re = Regex::new(r"bee.*\.rb").unwrap(); if re.is_match(file_path)
          for (_i, path1) in paths.enumerate() {
               let file_path = path1.unwrap().path().display().to_string();
               if is_bee_scrpt(&file_path) {
                    path = file_path.to_string();
                    break;
               }
          }
          if path == "_" {
               //println!("No script file not found in ./");
             return Err(Error::new(ErrorKind::Other,"No script file not found in ./"));
           }
     }
     if !Path::new(&path).exists() {
          //println!("File {} not found", path);
          return Err(Error::new(ErrorKind::Other, format!("File {} not found", path)));
     }
     lex::process(&log, &path, &run_args)?;
     io::stdout().flush()?;
     Ok(())
}