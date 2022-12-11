use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::{Rc, Weak};
use lex::{process_template_value, Lexem, VarVal, VarType};
use std::io::{self, Write};
use log::Log;
use std::path::Path;
use std::ffi::OsStr;
use std::time::{Duration, SystemTime};
use std::fs;
use fs::Metadata;

type FunCall = fn(Vec<Lexem>) -> Option<()>;

#[derive(Debug, PartialEq)]
pub enum BlockType {
    Main,
    Target,
    Dependency,
    If,
    Expr,
    Scope,
    Eq,
    Function,
    Neq,
    Then,

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
        let mut current_bl = self.0.borrow_mut();
        current_bl.vars.insert(name, val)
    }

    pub fn search_up(&self, name: &String) -> Option<VarVal> {
        let current_bl = self.0.borrow();
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
                return Some(VarVal{val_type: var.val_type.clone(), value: var.value.clone(), values: Vec::new()});
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

    pub fn eval_dep(&self) -> bool {
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
                            println!("evaluating target: {}", dep_block.params[0]);
                            let mut target = self.get_target(dep_block.params[0].to_string());
                            match target {
                                Some(target) => {
                                    let target_bor = target.0.borrow();
                                    return exec_target(&target_bor);
                                    
                                },
                                _ => ()
                            }
                        },
                        "anynewer" => {
                            println!("evaluating allnewer: {}", dep_block.params.len());
                            let log = Log {debug : false, verbose : false};
                            let p1 = process_template_value(&log, &dep_block.params[0], self);
                            let p2 = process_template_value(&log, &dep_block.params[1], self);
                            println!("parameter: {}, {}", p1, p2);
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
                                  eval_fun(&p1_block)
                             },
                             _ => { todo!("block: {:?}", dep_block.block_type);
                                None
                            }
                        };
                            let r2 =
                            if len == 2 {
                                None
                            } else {
                                None
                            };
                        r1 == r2
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
            let mut parent = curr.parent();
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
}

pub fn run(log: &Log, block: GenBlockTup, targets: &Vec<String>) -> io::Result<()> {
    let naked_block = block.0.borrow();
    let target = if targets.len() == 0 {
        //naked_block.children.reverse();
        let mut tar_name = String::from("");
        for ch in &naked_block.children {
            let ch_block = ch.0.borrow();
            if ch_block.block_type == BlockType::Target {
                tar_name = ch_block.name.as_ref().unwrap().to_string();
                //break ;
            }
        }
        tar_name
    } else {
        targets[0].to_string()
    };
    println!("processing for {} in {}", target, naked_block.children.len());
    for bl in &naked_block.children {
        let clone_bl = bl.clone();
        let ch_block = bl.0.borrow();
        if ch_block.block_type == BlockType::Target && ch_block.name.as_ref().unwrap().to_string() == target { 
            println!("target: {}", exec_target(&ch_block));
        }
    }
    Ok(())
}

pub fn exec_target(target: &GenBlock) -> bool {
    // dependencies
    let mut need_exec = false;
    for dep in &target.deps {
        need_exec |= dep.eval_dep();
    }
    need_exec
}

pub fn eval_fun(fun: &GenBlock) -> Option<String> {
    if fun.block_type == BlockType::Function {
        match fun.name.as_ref().unwrap().as_str() {
            "timestamp" => {
                return timestamp(&fun.params[0]);
            },
            _ => todo!("unreleased function: {:?}", fun.name)
        }
    }
    None
} 

pub fn timestamp(p: &str) -> Option<String> {
    let metadata  = fs::metadata(p);
    if let Ok(metadata) = metadata {
        if let Ok(time) = metadata.modified() {
            Some(format!("{time:?}"))
        } else {
            None
        }
    } else {
        None
    }
    
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