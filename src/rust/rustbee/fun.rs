use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::{Rc, Weak};
use lex::{process_template_value, Lexem, VarVal, VarType};
use std::io::{self, Write};
use log::Log;
use std::path::Path;
use std::time::{ SystemTime};
use std::fs;
use std::fs::File;
use std::process::Command;
//use http::{Request,Response};
use time;

type FunCall = fn(Vec<Lexem>) -> Option<()>;

#[derive(Debug, PartialEq)]
pub enum BlockType {
    Main,
    Target,
    Dependency,
    If,
    Scope,
    Eq,
    Function,
    Neq,
    Then,
    Else,
    Or,
    And,
    Not,
}

#[derive(Debug)]
pub struct GenBlock {
    pub name: Option<String>,
    pub block_type: BlockType,
    pub dir:Option<String>, // working directory
    pub flex: Option<String>,
    pub vars: HashMap<String, VarVal>,
    pub params: Vec<String>, // for a function, perhsps should be tuple as parameter(value,type)
    pub children: Vec<GenBlockTup>,
    pub deps: Vec<GenBlockTup>,
    //pub parent: Option<WeakGenBlock>,
    pub parent: Option<GenBlockTup>,
}

#[derive(Clone, Debug)]
pub struct GenBlockTup(pub Rc<RefCell<GenBlock>>);

pub type WeakGenBlock = Weak<RefCell<GenBlock>>;

impl GenBlock {
    pub fn new (block_type: BlockType) -> GenBlock {
        GenBlock {
            block_type : block_type,
            name : None,
            dir : None,
            flex : None,
            vars : HashMap::new(),
            children : Vec::new(),
            params : Vec::new(),
            deps : Vec::new(),
            parent : None
        }
    }
}

impl GenBlockTup {
    pub fn add(&self, node: GenBlockTup) -> GenBlockTup {
        //(node.0).borrow_mut().parent = Some(Rc::downgrade(&self.0));
        (node.0).borrow_mut().parent = Some(GenBlockTup(Rc::clone(&self.0)));
        let result = GenBlockTup(Rc::clone(&node.0));
        self.0.borrow_mut().children.push(node);
        result
    }

    pub fn add_dep(&self, node: GenBlockTup) -> GenBlockTup {
       // (node.0).borrow_mut().parent = Some(Rc::downgrade(&self.0));
       (node.0).borrow_mut().parent = Some(GenBlockTup(Rc::clone(&self.0)));
        let result = GenBlockTup(Rc::clone(&node.0));
        self.0.borrow_mut().deps.push(node);
        result
    }

    pub fn add_var(&self, name: String, val: VarVal) -> Option<VarVal> {
       // println!("borrow_mut()"        );
        let mut current_bl = self.0.borrow_mut();
        current_bl.vars.insert(name, val)
    }

    pub fn search_up(&self, name: &String) -> Option<VarVal> {
        let  current_bl = self.0.borrow();
       // let mut current_vars = current_bl.vars;
        let var = current_bl.vars.get(name);
        match var {
            None => {
                match &current_bl.parent {
                    None => return None,
                    Some(parent) => {
                        return parent.search_up(name);
                    }
                }
            },
            Some(var) => {
                let mut newvec = Vec::new(); // perhaps overhead, find a better solution
                for  newval in &var.values {
                    newvec.push(newval.clone());
                }
                return Some(VarVal{val_type: var.val_type.clone(), value: var.value.clone(), values: newvec});
            }
        }
    }

    pub fn clone(&self) -> GenBlockTup {
        GenBlockTup(Rc::clone(&self.0))
    }

    pub fn parent(& self) -> Option<GenBlockTup> {
        let bl = self.0.borrow();
        if let Some(parent) = &bl.parent {
            Some(parent.clone())
        } else {
            None
        }
    }

    pub fn eval_dep(&self, log: &Log, prev_res: &Option<String>) -> bool {
        let dep = self.0.borrow();
        if dep.children.len() == 0 {
            
            return true
        } else if dep.children.len() == 1 {
            let dep_task = &dep.children[0];
            let dep_block = dep_task.0.borrow();
            match dep_block.block_type {
                BlockType::Function => {
                    match dep_block.name.as_ref().unwrap().as_str() {
                        "target" => {
                            log.debug(&format!("evaluating target: {}", dep_block.params[0]));
                            let target = self.get_target(dep_block.params[0].to_string());
                            match target {
                                Some(target) => {
                                    let target_bor = target.0.borrow();
                                    return exec_target(&log, &target_bor);
                                },
                                _ => ()
                            }
                        },
                        "anynewer" => {
                            log.debug(&format!("evaluating allnewer: {}", dep_block.params.len()));
                            let log = Log {debug : false, verbose : false};
                            let p1 = process_template_value(&log, &dep_block.params[0], self, prev_res);
                            let p2 = process_template_value(&log, &dep_block.params[1], self, prev_res);
                            log.debug(&format!("parameter: {}, {}", p1, p2));
                            return exec_anynewer(self, &p1, &p2);
                        },
                        _ => todo!("function: {:?}", dep_block.name)
                    } 
                },
                BlockType::Eq => {
                    let len = dep_block.children.len();
                    if len  > 0 {
                        let p1 = &dep_block.children[0];
                        let p1_block = p1.0.borrow();
                        let r1 : Option<String> =
                         match p1_block.block_type {
                             BlockType::Function => {
                                p1.exec_fun(&log, &p1_block, prev_res)
                             },
                             _ => { todo!("block: {:?}", dep_block.block_type);
                            }
                        };
                        let r2 : Option<String> =
                            if len == 2 {
                                  match p1_block.block_type {
                                    BlockType::Function => {
                                        p1.exec_fun(&log, &p1_block, prev_res)
                                    },
                                    _ => { todo!("block: {:?}", dep_block.block_type);
                                    }
                                }
                            } else {
                                None
                            };
                            log.debug(&format!("comparing: {:?} and {:?}", r1, r2));
                        return r1 == r2;
                    } else {
                        return false
                    };
                },
                _ => todo!()
            }
        }
        false
    }

    pub fn get_top_block(& self) -> GenBlockTup {
        let mut curr =self.clone();
        loop {
            let parent = curr.parent();
            match parent {
                None => return curr.clone(),
                Some(parent) => {
                    curr = parent;
                }
            }
        }
    }

    pub fn get_target(&self, name: String) -> Option<GenBlockTup> {
        let top_block = &self.get_top_block();
        let naked_block = top_block.0.borrow();
        for ch in &naked_block.children {
            let ch_block = ch.0.borrow();
            if ch_block.block_type == BlockType::Target {
                if let Some(name1) = &ch_block.name {
                    if *name1 == name {
                       // tar_name = ch_block.name.as_ref().unwrap().to_string();
                        return  Some(ch.clone());
                     }
                }
            }
        }
        None
    }

    pub fn exec(&self, log: &Log, prev_res: &Option<String>) -> Option<String> {
        let naked_block = self.0.borrow();
        log.debug(&format!("exec {:?} name: {:?} prev: {:?}", naked_block.block_type, naked_block.name, prev_res));
        match naked_block.block_type {
            BlockType::Scope | BlockType::Then | BlockType::Else => {
                let mut res = prev_res.clone();
                for child in &naked_block.children {
                    res = child.exec(&log, &res);
                }  
            },
            BlockType::If => {
                let children = &naked_block.children;
                let res = children[0].exec(&log, prev_res);
                log.debug(&format!("neq {:?}", res));
                if res.unwrap_or("false".to_string()) == "true" {
                    children[1].exec(&log, prev_res);
                } else if children.len() == 3 {
                    children[2].exec(&log, prev_res);
                } else if children.len() > 3 {
                    log.error(&format!("unexpected block(s) {}", children.len()));
                }
            },
            BlockType::Function => {
               /* println!("function; {:?}", naked_block.name);
                for param in &naked_block.params {
                    println!("parameter; {}", param);
                }  */
                let res = self.exec_fun(&log, &naked_block, prev_res);
                return res;
                
                     
            },
            BlockType::Or => {
                let children = &naked_block.children;
                for child in children {
                    let res = child.exec(&log, prev_res).unwrap_or("false".to_string());
                    if res == "true" {
                        return Some("true".to_string());
                   }
                }
                return Some("false".to_string());
            },
            BlockType::And => {
                let children = &naked_block.children;
                for child in children {
                    let res = child.exec(&log, prev_res).unwrap_or("false".to_string());
                    if res == "false" {
                        return Some("false".to_string());
                   }
                }
                return Some("true".to_string());
            },
            BlockType::Not => {
                let children = &naked_block.children;
                if children.len() > 1 {
                    log.error(&format!("unexpected block(s) {}", children.len()));
                }
                let res = children[0].exec(&log, prev_res).unwrap_or("false".to_string());
                if res == "false" {
                    return Some("true".to_string());
               }
               return Some("false".to_string());
            },
            _ => todo!("block: {:?}, {:?}", naked_block.block_type, naked_block.name)
        }
        None
    }

    pub fn exec_fun(&self, log: &Log, fun_block: &GenBlock, res_prev: &Option<String>) -> Option<String> {
        match fun_block.name.as_ref().unwrap().as_str() {
            "display" => {
                println!("{}", self.parameter(&log, 0, fun_block, res_prev));
            },
            "now" => {
              
              return Some(format_system_time(SystemTime::now()));
            },
            "write" => {
                let fname = self.parameter(&log, 0, fun_block, res_prev);
                let mut f =  File::create(*fname) .expect("Error encountered while creating file!");
                let mut i = 1;
                let len = fun_block.params.len();
                while  i < len {
                    write!(f, "{}", self.parameter(&log, i, fun_block, res_prev)).expect("Error in writing file!");
                   i += 1;
                }
            },
            "neq" => {
                log.debug(&format!("comparing {:?} and {:?}", self.parameter(&log, 0, fun_block, res_prev), self.parameter(&log, 1, fun_block, res_prev)));

                return if self.parameter(&log, 0, fun_block, res_prev) == self.parameter(&log, 1, fun_block, res_prev) {
                    Some("false".to_string())
                } else {
                    Some("true".to_string())
                };
            },
            "eq" => {
                // TODO reuse common code with neq
                log.debug(&format!("comparing {:?} and {:?}", self.parameter(&log, 0, fun_block, res_prev), self.parameter(&log, 1, fun_block, res_prev)));

                return if self.parameter(&log, 0, fun_block, res_prev) == self.parameter(&log, 1, fun_block, res_prev) {
                    Some("true".to_string())
                } else {
                    Some("false".to_string())
                };
            },
            "exec" => {
                let mut exec : String  = fun_block.flex.as_ref().unwrap().to_string();
                // look for var first
                match self.search_up(&exec) {
                    Some(exec1) => { exec = *process_template_value(&log, &exec1.value, self, res_prev);},
                    None => ()
                }
                let mut params: Vec<String> = Vec::new();
                for i in 0..fun_block.params.len() {
                    let param = &fun_block.params[i];
                    let val = self.search_up(&param);
                    // TODO add resolving using last result ~~
                    log.debug(&format!("search: {:?} {:?}", fun_block.params, val));
                    if let Some(param) = val {
                        if param.values.len() > 0 {
                            for param in param.values {
                                params.push(param); 
                            }
                        } else {
                            params.push(param.value);
                        }
                    } else {
                        params.push(*self.parameter(&log, i, fun_block, res_prev));
                    } 
                }
                let dry_run = self.search_up(&"~dry-run~".to_string());
                if let Some(dry_run) = dry_run {
                   // println!("command: {:?} {:?}", exec, params);
                   log.log(&format!("command: {:?} {:?}", exec, params));
                   return Some("0".to_string());
                } else {
                    let status = Command::new(&exec)
                    .args(&params)
                    .status().expect(&format!("{} command with {:?} failed to start", exec, params));
                    match status.code() {
                        Some(code) => {
                            return Some(code.to_string());},
                            //self.parent().unwrap().add_var("~~".to_string(), VarVal{val_type: VarType::Number, value: code.to_string(), values: Vec::new()});},
                        None       => log.error(&format!("Process terminated by signal"))
                    }
               }
            },
            "or" => {
                for i in 0..fun_block.params.len() {
                    let param = *self.parameter(&log, i, fun_block, res_prev);
                    if param == "true" {
                        return Some("true".to_string());
                    }
                }
                return Some("false".to_string());
            },
            "and" => {
                for i in 0..fun_block.params.len() {
                    let param = *self.parameter(&log, i, fun_block, res_prev);
                    if param == "false" {
                        return Some("false".to_string());
                    }
                }
                return Some("true".to_string());
            },
            "cropname" => {
                let param = *self.parameter(&log, 0, fun_block, res_prev); 
            },
            "filename" => {
                let param = *self.parameter(&log, 0, fun_block, res_prev);
                let dot_pos = param.rfind('.');
                let slash_pos = param.rfind('/');
                match slash_pos {
                    None => {
                        match dot_pos {
                            Some(dot_pos) => {
                                return Some(param[0..dot_pos].to_string());
                            },
                            None => return Some(param),
                        }
                    },
                    Some(slash_pos) => {
                        match dot_pos {
                            Some(dot_pos) => {
                                return Some(param[slash_pos..dot_pos].to_string());
                            },
                            None => return Some(param[slash_pos..].to_string())
                        }
                    }
                }
            },
            "ask" => {
                let len = fun_block.params.len();
                if len > 0 {
                    print!("{} ", *self.parameter(&log, 0, fun_block, res_prev));
                    io::stdout().flush().unwrap();
                }
                let mut user_input = String::new();
                let stdin = io::stdin();
                stdin.read_line(&mut user_input);
                user_input = user_input.trim().to_string();
                if user_input.len() == 0 && len > 1 {
                    user_input = *self.parameter(&log, 1, fun_block, res_prev);
                }
                println!("");
                return Some(user_input);
            },
            "timestamp" => {
                return timestamp(&self.parameter(&log, 0, fun_block, res_prev));
            },
            "read" => {
                let fname = self.parameter(&log, 0, fun_block, res_prev);
                return fs::read_to_string(*fname)
                .ok();
            },
            "as_url" => {
               let param = self.search_up(&fun_block.params[0]);
               log.debug(&format!{"param: {:?}", param});
               if let Some(param) = param {
                   match param.val_type {
                    VarType::RepositoryRust => {
                        if let Some(pos) = param.value.find('@') {
                            return Some(format!("https://crates.io/api/v1/crates/{}/{}/download", &param.value[0..pos], &param.value[pos+1..]));
                        }
                    },
                    _ => ()
                   }
               }
            },
            "panic" => {
                panic!("{}", self.parameter(&log, 0, fun_block, res_prev));
            },
            _ => todo!("unimplemented func: {:?}", fun_block.name)
        }
        None
    }

    pub fn parameter(&self, log: &Log, i: usize, fun_block: &GenBlock, res_prev: &Option<String>) -> Box<String> {

        process_template_value(&log, &fun_block.params[i], self, res_prev)
    }
}

pub fn run(log: &Log, block: GenBlockTup, targets: &mut Vec<String>) -> io::Result<()> {
    let naked_block = block.0.borrow();
   
    if targets.len() == 0 { 
        let mut tar_name : Option<String> = None;
        for ch in &naked_block.children {
            let ch_block = ch.0.borrow();
            if ch_block.block_type == BlockType::Target {
                tar_name = Some(ch_block.name.as_ref().unwrap().to_string());
            }
        }
        if tar_name == None {
            panic!("No target found in the build script");
        } else {
            targets.push(tar_name.unwrap());
        }
    }
    log.log(&format!("targets: {:?}", targets));
    'targets: for target in targets {
        log.log(&format!("processing for {} in {}", target, naked_block.children.len()));
        for bl in &naked_block.children {
           // let clone_bl = bl.clone();
            let ch_block = bl.0.borrow();
            if ch_block.block_type == BlockType::Target && ch_block.name.as_ref().unwrap() == target { 
                log.log(&format!("target: {}", exec_target(&log, &ch_block)));
                continue 'targets;
            }
        }
        log.error(&format!("No target {} found", target));
    }
    
    Ok(())
}

pub fn exec_target(log: &Log, target: &GenBlock /*, res_prev: &Option<String>*/) -> bool {
    // dependencies
    let mut need_exec = false;
    for dep in &target.deps {
        need_exec |= dep.eval_dep(&log, &None);
    }
    let force_build = &target.parent.as_ref().unwrap().search_up(&"~force-build-target~".to_string());
    if let Some(_force_build) = force_build {
        need_exec = true;
    }
    if need_exec {
        for child in &target.children {
            child.exec(&log, &None);
        }
    } else {
        log.debug(&format!("no need to run: {:?}", &target.name));
    }
    need_exec
} 

pub fn timestamp(p: &str) -> Option<String> {
    let metadata  = fs::metadata(p);
    if let Ok(metadata) = metadata {
        if let Ok(time) = metadata.modified() {
            Some(format_system_time(time))
        } else {
            None
        }
    } else {
        None
    }
}

pub fn format_system_time(time: SystemTime) -> String {
    let dur = time.duration_since(SystemTime::UNIX_EPOCH).unwrap();
    let (y,m,d,h,min,s) = time:: get_datetime(1970, dur.as_secs());
              
    format!("{:0>2}{:0>2}{:0>2}T{:0>2}{:0>2}{:0>2}Z", y,m,d,h,min,s) 
}

pub fn exec_anynewer(block:&GenBlockTup, p1: &String, p2: &String) -> bool {
    let t1 = newest(p1);
    let t2 = newest(p2);
    t1 > t2
}

pub fn newest(mask : &str) -> Option<SystemTime> {
    let path1 = Path::new(mask);
    let parent1 = path1.parent().unwrap(); // can be empty, check
    let name1 = path1.file_name().unwrap();
    let str_name1 = name1.to_str().unwrap();
    let pos1 = str_name1.find('*'); // TODO add checking for more *
    return
    if let Some(pos) = pos1 {
        let mut last: Option<SystemTime> = None;
        for entry in fs::read_dir(parent1).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_file() {
                  if let Some(path1) = path.file_name() {
                       if let Some(file_path) = path1.to_str() {
                          if str_name1.len() == 1 || 
                             (pos == 0 && file_path.ends_with(&str_name1[1..])) ||
                             (pos == str_name1.len()-1 && file_path.starts_with(&str_name1[0..pos])) ||
                             (file_path.starts_with(&str_name1[0..pos]) && file_path.ends_with(&str_name1[pos+1..]) && file_path.len() >= str_name1.len()) {
                                let current_last = last_modified(&path.into_os_string().into_string().unwrap());
                                match last {
                                    None => last = current_last,
                                    Some(time) => {
                                        if let Some(time2) = current_last {
                                            if time2 > time {
                                                last = current_last;
                                            }
                                        }
                                    }
                                }
                             }    
                       }
                  }
             }
        }
        last
    } else {
        last_modified(path1.to_str().unwrap())
    };
}

pub fn last_modified(file: &str) -> Option<SystemTime> {
    let metadata = fs::metadata(file).expect("metadata call failed");

    if let Ok(time) = metadata.modified() {
        Some(time)
    } else {
        None
    }
}