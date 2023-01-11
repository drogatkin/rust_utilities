use std::fs;
use std::env;
use std::path::Path;
use std::io::{self, Write, BufRead};
use std::io::{Error, ErrorKind};
use log::Log;
//use regex::Regex;
use std::cell::RefCell;
use std::rc::{Rc, Weak};
use std::time::{SystemTime};
use std::fs::File;

mod help;
mod ver;
mod log;
mod lex;
mod fun;
mod time;
//mod util;

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
     DryRun,
     Quiet,
     TargetHelp
}

fn parse_command<'a>(log: &'a Log, args: &'a Vec<String>) -> (Vec<CmdOption>, Vec<&'a String>, Vec<String>) {
     let (mut options, mut targets, mut run_args) = (Vec::new(), Vec::new(), Vec::new());
     let mut arg_n = 0;
     while arg_n < args.len() {
         let arg = &args[arg_n] ;
         let len = args.len();
         //println!("analizing {}", arg);
          if arg.starts_with("-h") {
              options.push(CmdOption::Help);
          } else if arg == &"-f" || arg.starts_with("-file") || arg.starts_with("-build") {
               arg_n += 1;
               if arg_n < len {
                    options.push(CmdOption::ScriptFile(args[arg_n].to_string()));
               } else {
                    log.error(&format!("No file path specified in -file option"));
               }
          } else if arg.starts_with("-s") || arg.starts_with("-find") {
               arg_n += 1;
               if arg_n < len {
                    if args[arg_n].starts_with("-") {
                         options.push(CmdOption::SearchUp("_".to_string()));
                         arg_n -= 1;
                    } else {
                         options.push(CmdOption::SearchUp(args[arg_n].to_string()));
                    }
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
          } else if arg.starts_with("-D")  {
               let prop_def = &arg[2..];
               let eq_pos = prop_def.find('=');
               if eq_pos.is_some() {
                    let pos = eq_pos.unwrap();
                    let name = &prop_def[0..pos];
                    let val = &prop_def[pos+1..];
                    env::set_var(name, val);
               } else {
                    log.error(&format!("Invalid property definition: {}", &arg));
               }
          } else if arg.starts_with("-xprop") || arg.starts_with("-prop") {
               arg_n += 1;
               if arg_n < len {
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
          } else if arg.starts_with("-q") {
               options.push(CmdOption::Quiet);
          } else if arg.starts_with("-th") || arg.starts_with("-targethelp") {
               options.push(CmdOption::TargetHelp);
          } else if arg == "--" { 
               arg_n += 1;
               if arg_n < len {
                    run_args.extend_from_slice( &args[arg_n..]);
                    
                    break;
               }
          } else if arg.starts_with("-")  {
               log.error(&format!("Not supported option: {}", &arg));
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

fn find_script(dir: &Path, name: &str) -> Option<String> {
     let absolute = fs::canonicalize(&dir.to_path_buf());
     if !absolute.is_ok() {
          return None
     }
     let binding = absolute.unwrap();
     let mut curr_dir = binding.as_path();
     while curr_dir.is_dir() {
          if name == "_" {
               for entry in fs::read_dir(curr_dir).unwrap() {
                    let entry = entry.unwrap();
                    let path = entry.path();
                    if path.is_file() {
                          if let Some(path1) = path.file_name() {
                               if let Some(file_path) = path1.to_str() {
                                    if is_bee_scrpt(&file_path) {
                                        env::set_var("PWD", curr_dir.to_str().unwrap());
                                         return Some(path.to_str().unwrap().to_string());
                                    }
                               }
                          }
                     }
                }
          } else {
               let mut path_buf = curr_dir.to_path_buf();
               path_buf.push(name);
               let script_path = path_buf.as_path();
               //println!{"-> {:?}", script_path};
               if script_path.exists() {
                    env::set_var("PWD", curr_dir.to_str().unwrap());
                    return Some(script_path.to_str().unwrap().to_string());
               }
          }        
          if let Some(dir1) = curr_dir.parent() {
               curr_dir = dir1;
               //println!{"looking in parent {:?}", curr_dir};
          } else {
               //println!{"no parent for {:?}", curr_dir};
               break;
          }
     }
     None
}

fn main() -> io::Result<()> {
     let mut log = Log {debug : false, verbose : false, quiet : false};
     let mut path = "_".to_string();
     let args: Vec<String> = env::args().collect();
     let (options, targets, run_args) = parse_command( &log, &args);

     let lex_tree = fun::GenBlockTup(Rc::new(RefCell::new(fun::GenBlock::new(fun::BlockType::Main))));
     let mut real_targets: Vec<String> = Vec::new();
     for target in targets {
          real_targets.push(target.to_string());
     }
     let _ = &lex_tree.add_var(String::from("~args~"), lex::VarVal::from_vec(&run_args));
     let _ = &lex_tree.add_var(String::from("~os~"),  lex::VarVal::from_string(std::env::consts::OS));
     if std::env::consts::OS == "windows" {
          let _ = &lex_tree.add_var(String::from("~separator~"),  lex::VarVal::from_string("\\"));
          let _ = &lex_tree.add_var(String::from("~path_separator~"),  lex::VarVal::from_string(";"));
     } else {
          let _ = &lex_tree.add_var(String::from("~separator~"),  lex::VarVal::from_string("/"));
          let _ = &lex_tree.add_var(String::from("~path_separator~"),  lex::VarVal::from_string(":"));
     }
     let cwd = Path::new(&".").canonicalize().unwrap().into_os_string().into_string().unwrap();
     lex_tree.add_var(String::from("~cwd~"),  lex::VarVal::from_string(&cwd));
     //println!("additional ars {:?}", lex_tree.search_up(&String::from("~args~")));
     let mut target_help = false;
     for opt in options {
          //println!("{:?}", opt);
          match opt {
               CmdOption::Version => {
                    let (ver, build, date) = ver::version();
                    println!("RB Version: {}, build: {} on {}", ver, build, date);
               },
               CmdOption::Help => { println!("{}", help::get_help()); return Ok(())},
               CmdOption::Verbose => log.verbose = true,
               CmdOption::Diagnostics => log.debug = true,
               CmdOption::Quiet => log.quiet = true,
               CmdOption::ScriptFile(file) => {
                    log.log(&format!("Script: {}", file));
                    
                    path = file.to_string();
               },
               CmdOption::SearchUp(file) => {
                    log.log(&format!("Search: {}", file));
                    let path1 = find_script(&Path::new("."), &file);
                    if path1.is_some() {
                         path = path1.unwrap();
                         let path1 = Path::new(&path);
                         let cwd = path1.parent().unwrap().to_str().unwrap();
                         lex_tree.add_var(String::from("~cwd~"), lex::VarVal::from_string(&cwd));
                    } else {
                         log.error(&format!("Script: {} not found", file));
                         return Err(Error::from_raw_os_error(-2)/*Error::new(ErrorKind::Other, "Script not found")*/);
                    }
               },
               CmdOption::ForceRebuild => {
                    let fb = lex::VarVal{val_type:lex::VarType::Bool, value: String::from("true"), values: Vec::new()};
                    let _ = &lex_tree.add_var(String::from("~force-build-target~"), fb);
               },
               CmdOption::DryRun => {
                    let dr = lex::VarVal{val_type:lex::VarType::Bool, value: String::from("true"), values: Vec::new()};
                    let _ = &lex_tree.add_var(String::from("~dry-run~"), dr);
               },
               CmdOption::PropertyFile(filename) => {
                    let file = File::open(filename)?;
                    let lines = io::BufReader::new(file).lines();
                    for line in lines {
                         if let Ok(prop_def) = line {
                              let eq_pos = prop_def.find('=');
                              if eq_pos.is_some() {
                                   let pos = eq_pos.unwrap();
                                   let name = &prop_def[0..pos];
                                   let val = &prop_def[pos+1..];
                                   env::set_var(name, val);
                              } else {
                                   log.error(&format!("Invalid property definition: {}", &prop_def));
                              }    
                         }
                     }
               },
               CmdOption::TargetHelp => target_help = true
          }
     }
     if !log.quiet {
          println!("RustBee (\x1b[0;36mrb\x1b[0m) v {} (c) Copyright {} D. Rogatkin", ver::version().0, 2023);
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
      if target_help {
          let tree = exec_tree.0.borrow();
         for child_tree in &tree.children {
                let child = child_tree.0.borrow();
               if child .block_type == fun::BlockType::Target {
                    log.message(&format!("Target {:?} as {:?}", child.name, child.flex));
               }
         }
      } else {
          fun::run(&log, exec_tree, &mut real_targets);
      }
     
     match sys_time.elapsed() {
          Ok(elapsed) => {
               log.log(&format!("Finished in {} sec(s)", elapsed.as_secs()));
          },
          _ =>  ()
     }
     io::stdout().flush()?;
     Ok(())
}