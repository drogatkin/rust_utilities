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
use std::cell::RefCell;
use std::rc::{Rc, Weak};
use std::time::{SystemTime};

mod help;
mod ver;
mod log;
mod lex;
mod fun;
mod time;

#[derive(Debug)]
enum CmdOption {
     Help,
     ScriptFile(String),
     Version,
     Verbose,
     SearchUp(String),
     PropertyFile(String),
     Diagnostics,
     ForceRebuild,
     DryRun
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
          } else if arg.starts_with("-dry")  {
               options.push(CmdOption::DryRun);
          } else if arg.starts_with("-d") || arg.starts_with("-diagnostic") {
               options.push(CmdOption::Diagnostics);
               env::set_var("RUST_BACKTRACE", "1");
          } else if arg.starts_with("-r")  {
               options.push(CmdOption::ForceRebuild);
          } else if arg.starts_with("-xprop") || arg.starts_with("-prop") {
               arg_n += 1;
               if arg_n < args.len() {
                    if args[arg_n].starts_with("-") {
                         log.error(&"No property file specified");
                         arg_n -= 1;
                         continue;
                    }
                    options.push(CmdOption::PropertyFile(args[arg_n].to_string()));
               } else {
                    log.error(&"Property file isn't specified".to_string());
                    break;
               }
          } else if arg == "--" { 
               arg_n += 1;
               if arg_n < args.len() {
                    run_args.extend_from_slice( &args[arg_n..]);
                    
                    break;
               }
          } else if arg_n > 0 {
               targets.push(arg);
          }
         
         arg_n += 1;
     }
     (options, targets, run_args)
}

fn is_bee_scrpt(file_path: &str) -> bool {
     file_path.starts_with("bee") && file_path.ends_with(".rb") 
}

fn find_script(dir: &Path) -> Option<String> {
     let mut curr_dir = dir;
     while curr_dir.is_dir() {
          for entry in fs::read_dir(curr_dir).unwrap() {
              let entry = entry.unwrap();
              let path = entry.path();
              if path.is_file() {
                    if let Some(path1) = path.file_name() {
                         if let Some(file_path) = path1.to_str() {
                              if is_bee_scrpt(&file_path) {
                                   return Some(file_path.to_string());
                              }
                         }
                    }
               }
          }
          
          if let Some(dir1) = curr_dir.parent() {
               curr_dir = dir1;
          } else {
               break;
          }
     }
     None
}

fn main() -> io::Result<()> {
     println!("RustBee (rb) v 1.0.0 (c) Copyright {} D. Rogatkin", 2022);
     let mut log = Log {debug : false, verbose : false};
     let mut path = "_".to_string();
     let args: Vec<String> = env::args().collect();
     let (options, targets, run_args) = parse_command( &log, &args);

     let lex_tree = fun::GenBlockTup(Rc::new(RefCell::new(fun::GenBlock::new(fun::BlockType::Main))));
     // add command arguments
     let args = lex::VarVal{val_type:lex::VarType::Array, value: String::from(""), values: run_args};
     let mut real_targets: Vec<String> = Vec::new();
     for target in targets {
          real_targets.push(target.to_string());
     }
     &lex_tree.add_var(String::from("~args~"), args);
     //println!("additional ars {:?}", lex_tree.search_up(&String::from("~args~")));
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
               CmdOption::ForceRebuild => {
                    let fb = lex::VarVal{val_type:lex::VarType::Bool, value: String::from("true"), values: Vec::new()};
                    &lex_tree.add_var(String::from("~force-build-target~"), fb);
               },
               CmdOption::DryRun => {
                    let dr = lex::VarVal{val_type:lex::VarType::Bool, value: String::from("true"), values: Vec::new()};
                    &lex_tree.add_var(String::from("~dry-run~"), dr);
               },
               _ => ()
          }
     }
     if path == "_" {
          let paths = fs::read_dir(&"./").unwrap();
          //let re = Regex::new(r"bee.*\.rb").unwrap(); if re.is_match(file_path)
          for (_i, path1) in paths.enumerate() {
               let path2 = path1.unwrap().path() ;
               if path2.is_file() {
                    if let Some(path3) = path2.file_name() {
                         if let Some(file_path) = path3.to_str() {
                              if is_bee_scrpt(&file_path) {
                                   path = file_path.to_string();
                                   break;
                              }
                         }
                    }
               }
          }
          if path == "_" {
               //println!("No script file not found in ./");
             return Err(Error::new(ErrorKind::Other,"No script file found in ./"));
           }
     }
     if !Path::new(&path).exists() {
          //println!("File {} not found", path);
          return Err(Error::new(ErrorKind::Other, format!("File {} not found", path)));
     }
     
     let sys_time = SystemTime::now();
     
     let exec_tree = lex_tree.clone();
     lex::process(&log, &path, lex_tree)?;
      
     fun::run(&log, exec_tree, &mut real_targets);
     match sys_time.elapsed() {
          Ok(elapsed) => {
               log.log(&format!("Finished in {} sec(s)", elapsed.as_secs()));
          },
          _ =>  ()
     }
     io::stdout().flush()?;
     Ok(())
}