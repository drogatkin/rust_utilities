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
use std::fs::OpenOptions;
use std::process::Command;
//use http::{Request,Response};
use time;

type FunCall = fn(Vec<Lexem>) -> Option<()>;

#[derive(Debug, PartialEq,Clone)]
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
    For,
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

pub type WeakGenBlock = Weak<RefCell<GenBlock>>; // use Rc::new_cyclic

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
    pub fn search_up(&self, name: &String) -> Option<VarVal> {
        let var = self.vars.get(name);
        match var {
            None => {
                match &self.parent {
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
                return Some(var.clone());
            }
        }
    }

    pub fn prev_or_search_up(&self, name: &String, prev: &Option<VarVal>) -> Option<VarVal> {
        if "~~" == name {
            prev.clone()
        } else {
            self.search_up(name)
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
                return Some(var.clone());
            }
        }
    }

    pub fn prev_or_search_up(&self, name: &String, prev: &Option<VarVal>) -> Option<VarVal> {
        if "~~" == name {
            prev.clone()
        } else {
            self.search_up(name)
        }
    }

    pub fn parent(& self) -> Option<GenBlockTup> {
        let bl = self.0.borrow();
        if let Some(parent) = &bl.parent {
            Some(parent.clone())
        } else {
            None
        }
    }

    pub fn eval_dep(&self, log: &Log, prev_res: &Option<VarVal>) -> bool {
        let dep = self.0.borrow();
        let len = dep.children.len();
        if len == 0 {
            
            return true
        } else if len == 1 {
            let dep_task = &dep.children[0];
            let dep_block = dep_task.0.borrow();
            match dep_block.block_type {
                BlockType::Function => {
                    match dep_block.name.as_ref().unwrap().as_str() {
                        "target" => {
                            log.debug(&format!("evaluating target: {}", dep_block.params[0]));
                            let target = self.get_target(&dep_block.params[0]);
                            match target {
                                Some(target) => {
                                    //let target_bor = target.0.borrow_mut();
                                    return exec_target(&log, & target);
                                },
                                _ => ()
                            }
                        },
                        "anynewer" => {
                            log.debug(&format!("evaluating anynewer: {}", dep_block.params.len()));
                            let p1 = process_template_value(&log, &dep_block.params[0], &dep, prev_res);
                            let p2 = process_template_value(&log, &dep_block.params[1], &dep, prev_res);
                            log.debug(&format!("anynewer parameters: {}, {}", p1, p2));
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
                        let r1 : Option<VarVal> =
                         match p1_block.block_type {
                             BlockType::Function => {
                                p1.exec_fun(&log, &p1_block, prev_res)
                             },
                             _ => { todo!("block: {:?}", dep_block.block_type);
                            }
                        };
                        let r2 : Option<VarVal> =
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
                        match r1 {
                            None => {
                                match r2 {
                                    None => return true,
                                    _ => return false
                                }
                            },
                            Some(r1) => {
                                match r2 {
                                    None => return false,
                                    Some(r2) => return r1.value == r2.value
                                }
                            }
                        }
                       // return r1 == r2;
                    } else {
                        return false
                    };
                },
                BlockType::Or => {
                    //let len = dep_block.children.len();
                    for child in &dep_block.children {
                        let res = child.exec(&log, prev_res).unwrap_or(VarVal::from_bool(false)).is_true();
                        if res {
                            return true;
                       }
                    }
                    return false;
                },
                _ => todo!("the operation {:?} isn't supported yet", dep_block.block_type)
            }
        } else {
            log.error(&format!("{} children not supported in a dependency", len));
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

    pub fn get_target(&self, name: &String) -> Option<GenBlockTup> {
        let top_block = &self.get_top_block();
        let naked_block = top_block.0.borrow();
        for ch in &naked_block.children {
            let ch_block = ch.0.borrow();
            if ch_block.block_type == BlockType::Target {
                if let Some(name1) = &ch_block.name {
                    if name1 == name {
                       // tar_name = ch_block.name.as_ref().unwrap().to_string();
                        return  Some(ch.clone());
                     }
                }
            }
        }
        None
    }

    pub fn is_assign(&self) -> bool {
        let naked_block = self.0.borrow();
        naked_block.block_type == BlockType::Function && naked_block.name.is_some() && naked_block.name.as_ref().unwrap() == "assign"
    }

    pub fn exec(&self, log: &Log, prev_res: &Option<VarVal>) -> Option<VarVal> {
        //let naked_block = self.0.borrow();
        //log.debug(&format!("exec {:?} name: {:?} prev: {:?}", naked_block.block_type, naked_block.name, prev_res));
        let block_type = self.0.borrow().block_type.clone();
        log.debug(&format!("processing block {:?}", block_type));
        match  block_type {
            BlockType::Scope | BlockType::Then | BlockType::Else => {
                let mut res = match prev_res {
                    None => None,
                    Some(var) => Some(var.clone())
                };
                let children = &self.0.borrow().children.clone();
                for child in children {
                    if child.is_assign() {
                        let child_block = child.0.borrow();
                        let mut naked_block = self.0.borrow_mut();
                        res = child.exec_assign(&log, &child_block, &mut naked_block, &res);
                    } else {
                        res = child.exec(&log, &res);
                    }
                } 
                return res 
            },
            BlockType::If => {
                let naked_block = self.0.borrow();
                let children = &naked_block.children;
                let mut res = children[0].exec(&log, prev_res);
                log.debug(&format!("neq {:?}", res));
                if res.as_ref().unwrap_or(&VarVal::from_bool(false)).is_true() {
                    res = children[1].exec(&log, prev_res);
                } else if children.len() == 3 {
                    res = children[2].exec(&log, prev_res);
                } else if children.len() > 3 {
                    log.error(&format!("unexpected block(s) {}", children.len()));
                }
                return res
            },
            BlockType::Function => {
                let naked_block = self.0.borrow();
                log.debug(&format!("function; {:?}", naked_block.name));
                for param in &naked_block.params {
                    log.debug(&format!("parameter; {}", param));
                } 
                let res = self.exec_fun(&log, &naked_block, prev_res);
                return res;
            },
            BlockType::For => {
                let mut res = match prev_res {
                    None => None,
                    Some(var) => Some(var.clone())
                };
                let mut range = Vec::new();
                //range.push("test".to_string());
                let name_as_opt = &self.0.borrow().name.clone();
                if name_as_opt.is_none() {
                    log.error(&format!("For variable isn't specified"));
                    return None;
                }
                let name = name_as_opt.as_ref().unwrap();
                // dir as range
                let range_as_opt = &self.0.borrow().dir.clone();
                if range_as_opt.is_none() {
                    log.error(&format!("For range isn't specified"));
                    return None;
                }
                let range_as_var = self.search_up(&range_as_opt.as_ref().unwrap());
                
                if range_as_var.is_some() {
                    let range_as_val = range_as_var.unwrap();
                    if range_as_val.val_type == VarType::Array {
                        for var_el in range_as_val.values {
                            range.push(var_el.clone());
                        }
                    } else {
                        let sep_can = &self.0.borrow().flex.clone();
                        if sep_can.is_none() {
                            log.error(&format!("For values separator isn't specified"));
                            return None;
                        }
                        let sep_var = self.search_up(&sep_can.as_ref().unwrap());
                        let sep_val = match sep_var {
                            None => sep_can.as_ref().unwrap().clone(),
                            Some(val) => val.value.clone(),
                        };
                        let values = range_as_val.value.split(&sep_val);
                        for var_el in values {
                            range.push(var_el.to_string());
                        }
                    }
                } else {
                    let sep_can = &self.0.borrow().flex.clone();
                        if sep_can.is_none() {
                            log.error(&format!("For values separator isn't specified"));
                            return None;
                        }
                        let sep_var = self.search_up(&sep_can.as_ref().unwrap());
                        let sep_val = match sep_var {
                            None => sep_can.as_ref().unwrap().clone(),
                            Some(val) => val.value.clone(),
                        };
                    let values = range_as_opt.as_ref().unwrap().split(&sep_val);
                    for var_el in values {
                        range.push(var_el.to_string());
                    }
                }
                let children = &self.0.borrow().children.clone();
                //let mut naked_block = self.0.borrow_mut();
                for (index, element) in range.iter().enumerate() {
                    let var_element = VarVal{val_type: VarType::Generic, value: element.clone(), values: Vec::new()};
                    let var_index = VarVal{val_type: VarType::Number, value: format!("{}", index), values: Vec::new()};
                    //self.add_var(name.to_string(), var_element); element.clone()
                    self.0.borrow_mut().vars.insert(name.to_string(), var_element);
                    self.0.borrow_mut().vars.insert("~index~".to_string(), var_index);
                    //naked_block.vars.insert(name.to_string(), var_element);
                    for child in children {
                        if child.is_assign() {
                            let child_block = child.0.borrow();
                            let mut naked_block = self.0.borrow_mut();
                            res = child.exec_assign(&log, &child_block, &mut naked_block, &res);
                        } else {
                            res = child.exec(&log, &res);
                        } 
                    } 
                }     
                return res    
            },
            BlockType::Or => {
                let naked_block = self.0.borrow();
                let children = &naked_block.children;
                for child in children {
                    let res = child.exec(&log, prev_res).unwrap_or_default().is_true();
                    if res {
                        return Some(VarVal::from_bool(res));
                   }
                }
                return Some(VarVal::from_bool(false));
            },
            BlockType::And => {
                let naked_block = self.0.borrow();
                let children = &naked_block.children;
                for child in children {
                    let res = child.exec(&log, prev_res).unwrap_or_default().is_true();
                    if !res {
                        return Some(VarVal::from_bool(false));
                   }
                }
                return Some(VarVal::from_bool(true));
            },
            BlockType::Not => {
                let naked_block = self.0.borrow();
                let children = &naked_block.children;
                if children.len() > 1 {
                    log.error(&format!("unexpected block(s) {}", children.len()));
                }
                let res = children[0].exec(&log, prev_res).unwrap_or_default().is_true();
                if !res {
                    return Some(VarVal::from_bool(true));
               }
               return Some(VarVal::from_bool(false));
            },
            _ => todo!("block: {:?}, {:?}", self.0.borrow().block_type, self.0.borrow().name)
        }
      //  None
    }

    pub fn exec_fun(&self, log: &Log, fun_block: & GenBlock, res_prev: &Option<VarVal>) -> Option<VarVal> {
        match fun_block.name.as_ref().unwrap().as_str() {
            "display" => {
                println!("{}", self.parameter(&log, 0, fun_block, res_prev));
            },
            "now" => {
              return Some(VarVal::from_string(&format_system_time(SystemTime::now())));
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
            "writea" => {
                let fname = self.parameter(&log, 0, fun_block, res_prev);
                let mut f = OpenOptions::new()
                    .read(true)
                    .append(true) 
                    .create(true)
                    .open(*fname).expect("Error encountered while opening file!");
                let mut i = 1;
                let len = fun_block.params.len();
                while  i < len {
                    //log.log(&format!("->{}",self.parameter(&log, i, fun_block, res_prev)));
                    write!(f, "{}", self.parameter(&log, i, fun_block, res_prev)).expect("Error in writing file!");
                   i += 1;
                }
            },
            "neq" => {
                log.debug(&format!("comparing {:?} and {:?}", self.parameter(&log, 0, fun_block, res_prev), self.parameter(&log, 1, fun_block, res_prev)));

                return if self.parameter(&log, 0, fun_block, res_prev) == self.parameter(&log, 1, fun_block, res_prev) {
                    Some(VarVal::from_bool(false))
                } else {
                    Some(VarVal::from_bool(true))
                };
            },
            "eq" => {
                // TODO reuse common code with neq
                log.debug(&format!("comparing {:?} and {:?}", self.parameter(&log, 0, fun_block, res_prev), self.parameter(&log, 1, fun_block, res_prev)));

                return if self.parameter(&log, 0, fun_block, res_prev) == self.parameter(&log, 1, fun_block, res_prev) {
                    Some(VarVal::from_bool(true))
                } else {
                    Some(VarVal::from_bool(false))
                };
            },
            "exec" => {
                let mut exec : String  = fun_block.flex.as_ref().unwrap().to_string();
                // look for var first
                match fun_block.search_up(&exec) {
                    Some(exec1) => { exec = *process_template_value(&log, &exec1.value, &fun_block, res_prev);},
                    None => ()
                }
                let mut params: Vec<_> = Vec::new();
                for i in 0..fun_block.params.len() {
                    let param = &fun_block.params[i];
                    let val = self.prev_or_search_up(&param, res_prev);
                    // TODO add resolving using last result ~~
                    log.debug(&format!("exec params: {:?} for {:?}", fun_block.params, val));
                    if let Some(param) = val {
                        if param.values.len() > 0 { // array
                            for param in param.values {
                                params.push(*process_template_value(&log, &param, &fun_block, res_prev)); 
                            }
                        } else if param.val_type != VarType::Array {
                            params.push(*process_template_value(&log, &param.value, &fun_block, res_prev));
                        }
                    } else {
                        params.push(*self.parameter(&log, i, fun_block, res_prev));
                    } 
                }
                let dry_run = self.search_up(&"~dry-run~".to_string());
                let mut cwd = String::new();
                if fun_block.dir.is_some() {
                    let work_dir_val = fun_block.dir.as_ref().unwrap().to_string();
                    if !work_dir_val.is_empty() {
                        let work_dir =
                        match fun_block.search_up(&work_dir_val) {
                            Some(work_dir_val1) => { *process_template_value(&log, &work_dir_val1.value, &fun_block, res_prev)},
                            None => *process_template_value(&log, &work_dir_val, fun_block, res_prev)
                        };
                        //let work_dir = *process_template_value(&log, &work_dir_val, fun_block, res_prev);
                        let path =  Path::new(&work_dir);
                        if path.exists() {
                            cwd = path.canonicalize().unwrap().into_os_string().into_string().unwrap();
                        }
                    }
                }
                
                if let Some(_dry_run) = dry_run {
                   log.log(&format!("command: {:?} {:?} in {}", exec, params, cwd));
                   return Some(VarVal::from_i32(0));
                } else {
                    let status = if cwd.is_empty() { Command::new(&exec)
                    .args(&params)
                    .status().expect(&format!("{} command with {:?} failed to start", exec, params)) } else {
                        Command::new(&exec).current_dir(&cwd).args(&params)
                        .status().expect(&format!("{} command with {:?} in {} failed to start", exec, params, cwd)) 
                    };
                    match status.code() {
                        Some(code) => {
                            return Some(VarVal::from_i32(code));},
                            //self.parent().unwrap().add_var("~~".to_string(), VarVal{val_type: VarType::Number, value: code.to_string(), values: Vec::new()});},
                        None       => log.error(&format!("Process terminated by signal"))
                    }
               }
            },
            "or" => {
                for i in 0..fun_block.params.len() {
                    let param = *self.parameter(&log, i, fun_block, res_prev);
                    if param == "true" {
                        return Some(VarVal::from_bool(true));
                    }
                }
                return Some(VarVal::from_bool(false));
            },
            "and" => {
                for i in 0..fun_block.params.len() {
                    let param = *self.parameter(&log, i, fun_block, res_prev);
                    if param == "false" {
                        return Some(VarVal::from_bool(false));
                    }
                }
                return Some(VarVal::from_bool(true));
            },
            "scalar" => { // vector var, separator
                let sep = if fun_block.params.len() > 1 {
                    *self.parameter(&log, 1, fun_block, res_prev)
                } else {" ".to_string()};
                let vec_param = match fun_block.search_up(&fun_block.params[0]) {
                    Some(vec_param) => { 
                        if vec_param.val_type == VarType::Array {
                            let mut collect_str = vec_param.values[0].to_owned();
                            let mut next_el = 1;
                            while next_el < vec_param.values.len() {
                                collect_str.push_str(&sep);
                                collect_str.push_str(&vec_param.values[next_el]);
                                next_el += 1;
                            }
                            Some(VarVal::from_string(&collect_str))
                        } else {
                            Some(VarVal::from_string(&vec_param.value))
                        }
                     },
                    None => None
                };
                 
                return vec_param
            },
            "filename" => {
                let param = *self.parameter(&log, 0, fun_block, res_prev);
                let dot_pos = param.rfind('.');
                let slash_pos = param.rfind('/');
                match slash_pos {
                    None => {
                        match dot_pos {
                            Some(dot_pos) => {
                                return Some(VarVal::from_string(&param[0..dot_pos]));
                            },
                            None => return Some(VarVal::from_string(&param)),
                        }
                    },
                    Some(slash_pos) => {
                        match dot_pos {
                            Some(dot_pos) => {
                                return Some(VarVal::from_string(&param[slash_pos+1..dot_pos]));
                            },
                            None => return Some(VarVal::from_string(&param[slash_pos+1..]))
                        }
                    }
                }
            },
            "ask" => {
                let len = fun_block.params.len();
                // consider using trair write - write!{writer, "..."}
                print!("{} ", *self.parameter(&log, 0, fun_block, res_prev));
                io::stdout().flush().unwrap();
                let mut user_input = String::new();
                let stdin = io::stdin();
                let res = stdin.read_line(&mut user_input);
                if res.is_err() {
                    log.error(&format!{"An error in getting use input, default is used"});
                }
                user_input = user_input.trim().to_string();
                if user_input.is_empty() && len > 1 {
                    user_input = *self.parameter(&log, 1, fun_block, res_prev);
                }
                println!("");
                return Some(VarVal::from_string(&user_input));
            },
            "timestamp" => {
                if no_parameters(&fun_block) {
                    log.error(&format!{"no argument for timestamp"});
                } else {
                    let ts = timestamp(&self.parameter(&log, 0, fun_block, res_prev));
                    match ts {
                        Some(timestamp) => return Some(VarVal::from_string(&timestamp)),
                        None => return None
                    }
                }   
            },
            "read" => {
                let fname = self.parameter(&log, 0, fun_block, res_prev);
                return Some(VarVal::from_string(&fs::read_to_string(*fname)
                .ok().unwrap()));
            },
            "newerthan" => {
                // compare modification date of files specified by 1st parameter
                // in a form path/.ext1 with files specified by second parameter in
                // the same form and return an array of files from first parameters with 
                // newer modification time. File names are prependent with directory names 
                // relative to the parameter path
                // check if 1 or 2 parameters only
                let len = fun_block.params.len();
                let (dir1, ext1) = dir_ext_param(&self.parameter(&log, 0, fun_block, res_prev));
                if dir1.is_none() || ext1.is_none() {
                    log.error(&format!("Parameter {} doesn't have path/ext pattern", &self.parameter(&log, 0, fun_block, res_prev)));
                    return None
                }
                let (dir2, ext2) =
                    if len > 1 { 
                        dir_ext_param(&self.parameter(&log, 1, fun_block, res_prev))
                    } else {
                        (None,None)
                    };
                log.debug(&format!{"newerthen: {:?}/{:?} then {:?}/{:?}", &dir1, &ext1, &dir2, &ext2});
                return Some(VarVal::from_vec(&find_newer(&dir1.unwrap(), &ext1.unwrap(), &dir2, &ext2)));

            },
            "as_url" => {
               let param = self.search_up(&fun_block.params[0]);
               log.debug(&format!{"param: {:?}", param});
               if let Some(param) = param {
                   match param.val_type {
                    VarType::RepositoryRust => {
                        if let Some(pos) = param.value.find('@') {
                            return Some(VarVal::from_string(&format!("https://crates.io/api/v1/crates/{}/{}/download", &param.value[0..pos], &param.value[pos+1..])));
                        }
                    },
                    VarType::RepositoryMaven => {
                        let parts = param.value.split(':');
                        let mav_parts: Vec<_> = parts.collect();
                        //https://repo1.maven.org/maven2/com/baomidou/mybatis-plus-boot-starter/3.5.3.1/mybatis-plus-boot-starter-3.5.3.1.jar
                        return Some(VarVal::from_string(&format!("https://repo1.maven.org/maven2/{}/{}/{}/{}-{}.jar", &mav_parts[0].replace(".", "/"), &mav_parts[1], &mav_parts[2], &mav_parts[1], &mav_parts[2])));
                    },
                    _ => ()
                   }
               }
            },
            "assign" => {
                log.error(&format!("assign function has to be called as exec_assign"));
            },
            "array" => {
                let mut res : Vec<_> = Vec::new();
                for i in 0..fun_block.params.len() {
                    // TODO make the approach as a util method
                    // TODO although there is a note about string interpolation, perhaps do it only for final
                    // destinations as evaluate parameters in a function or a block
                    match fun_block.search_up(&fun_block.params[i]) {
                        Some(param1) => { 
                            if param1.val_type == VarType::Array {
                                res.extend_from_slice(&param1.values); // consider to massage a value  *process_template_value(&log, param1.values[k], &fun_block, res_prev);
                            } else {
                                res.push(*self.parameter(&log, i, fun_block, res_prev));
                            }
                        ;},
                        None => { let param = *self.parameter(&log, i, fun_block, res_prev);
                            res.push(param);
                        }
                    }
                   
                }
                return Some(VarVal::from_vec(&res));
            },
            "file_filter" => { // remove from an array parameter all matching parameters 1..n
                let param = self.prev_or_search_up(&fun_block.params[0], res_prev);
                if param.is_some() && param.as_ref().unwrap().val_type == VarType::Array {
                    let filter_vals: _ = fun_block.params[1..].iter().map(|filter| process_template_value(&log, filter, &fun_block, res_prev)).collect::<Vec<_>>();
                    let files = param.unwrap().values;
                    let vec = files.into_iter().filter(|file| {
                        let p = Path::new(file);
                        if !p.exists() {return false} ;
                        let name = p.file_name().unwrap().to_str().unwrap();
                        for filter in &filter_vals {
                            if matches(&name, &filter) {
                                return false
                            }
                        }
                        return true
                    }).collect();
                    return Some(VarVal::from_vec(&vec));
                } else {
                    log.error(&format!{"Variable {} not found or not an array", fun_block.params[0]});
                }
            },
            "panic" => {
                panic!("{}", self.parameter(&log, 0, fun_block, res_prev));
            },
            "element" => { // the function allow to extract or set an element of an array
                match fun_block.search_up(&fun_block.params[0]) {
                    Some(mut array) => { 
                        if array.val_type == VarType::Array {
                            let index_param = match self.prev_or_search_up(&fun_block.params[1], res_prev) {
                                None => fun_block.params[1].to_owned(),
                                Some(val) => val.value.clone()
                            };
                            let index: usize = index_param.parse().unwrap_or_default();
                            if fun_block.params.len() == 3 { // set
                                array.values[index] = fun_block.params[2].to_owned();
                            } else if fun_block.params.len() == 2 { // get
                                return Some(VarVal::from_string(&array.values[index]))
                            }
                        } else {
                            log.error(&format!{"Specified argument isn't an array"});
                        }
                    },
                    None => log.error(&format!{"Specified argument wasn't found"})
                }
            },
            _ => todo!("unimplemented func: {:?}", fun_block.name)
        }
        None
    }

    pub fn exec_assign(&self, log: &Log, fun_block: &GenBlock, parent_block: &mut GenBlock, res_prev: &Option<VarVal>) -> Option<VarVal> {
        let name = fun_block.params[0].to_owned();
       // let parent = parent_block.parent.as_ref().unwrap();
        let val = parent_block.prev_or_search_up(&fun_block.params[1], res_prev);
        log.debug(&format!("assign arguments resolved as {} {:?}/{}", name, val, &fun_block.params[1]));

        match val {
            None => parent_block.vars.insert(name, VarVal::from_string(*process_template_value(&log, &fun_block.params[1], &parent_block, res_prev))),
            Some(val) => parent_block.vars.insert(name, val.clone())
        }
    }

    pub fn parameter(&self, log: &Log, i: usize, fun_block: &GenBlock, res_prev: &Option<VarVal>) -> Box<String> {
        log.debug(&format!("looking for {:?} of {:?}", &fun_block.params[i], &fun_block.block_type));  
        let param = fun_block.prev_or_search_up(&fun_block.params[i], res_prev);
        match param {
            None => process_template_value(&log, &fun_block.params[i], &fun_block, res_prev),
            Some(val) => process_template_value(&log, &val.value, &fun_block, res_prev)
        }
        //process_template_value(&log, &fun_block.params[i], &fun_block, res_prev)
    }

}

pub fn run(log: &Log, block: GenBlockTup, targets: &mut Vec<String>) -> io::Result<()> {
    let naked_block = block.0.borrow();
   
    if targets.is_empty() { 
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
                log.log(&format!("target: {}", exec_target(&log, & bl)));
                continue 'targets;
            }
        }
        log.error(&format!("No target {} found", target));
    }
    
    Ok(())
}

pub fn exec_target(log: &Log, target_bl: & GenBlockTup) -> bool {
    // dependencies
    let mut need_exec = false;
    {
        let target = target_bl.0.borrow();
        log.debug(&format!("processing: {} deps of {:?}", &target.deps.len(), &target.name));
        for dep in &target.deps {
            need_exec |= dep.eval_dep(&log, &None);
        }
        let force_build = &target.parent.as_ref().unwrap().search_up(&"~force-build-target~".to_string());
        if let Some(_force_build) = force_build {
            need_exec = true;
        }
    }
    
    if need_exec {
        if target_bl.0.borrow().dir.is_some() {
            let dir_val = target_bl.0.borrow().dir.as_ref().unwrap().to_string();
            if !dir_val.is_empty() {
                let dir = *process_template_value(&log, &dir_val, &target_bl.0.borrow(), &None);
               let path =  Path::new(&dir);
               if path.exists() {
                    let cwd = path.canonicalize().unwrap().into_os_string().into_string().unwrap();
                    target_bl.0.borrow_mut().vars.insert(String::from("~cwd~"),  VarVal::from_string(&cwd));
               }
            }
        }
        {
            let target = target_bl.0.borrow();
            for child in &target.children {
                child.exec(&log, &None);
            }
        }
        
    } else {
        log.debug(&format!("no need to run: {:?}", &target_bl.0.borrow().name));
    }
    need_exec
} 

fn no_parameters(fun: &GenBlock) -> bool {
    fun.block_type == BlockType::Function && fun.params.len() < 2 && (fun.params.len() == 0 || fun.params[0].is_empty())
}

 fn val_to_string(val: Option<VarVal>) -> Option<String> {
    if val.is_none() {
        return None
    }
    let var_val = val.unwrap();
    if var_val.val_type == VarType::Array {
        let mut collect_str = var_val.values[0].to_owned();
        let mut next_el = 1;
        while next_el < var_val.values.len() {
            collect_str.push_str(&var_val.values[next_el]);
            next_el += 1;
        }
        Some(collect_str)
    } else {
        Some(var_val.value)
    }
}

pub fn vec_to_str(arr: &Vec<String>) -> String {
    let mut res = String::new();
    for el in arr {
        res.push_str(&el);
        res.push('\t');
    }
    res.to_string()
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

const DAYS_OF_WEEK: &[&str] = &["Thursday", "Friday", "Saturday",  "Sunday","Monday","Tuesday","Wednesday"]; 

pub fn format_system_time(time: SystemTime) -> String {
    let dur = time.duration_since(SystemTime::UNIX_EPOCH).unwrap();
    let (y,m,d,h,min,s,_w) = time:: get_datetime(1970, dur.as_secs());
    //println!{"week {} - {}", w, DAYS_OF_WEEK[w as usize]} ;
    format!("{:0>2}{:0>2}{:0>2}T{:0>2}{:0>2}{:0>2}Z", y,m,d,h,min,s) // see ISO 8601
}

pub fn exec_anynewer(_block:&GenBlockTup, p1: &String, p2: &String) -> bool {
    let t1 = newest(p1);
    let t2 = newest(p2);
  //  println!{"modified {:?} and {:?}", t1, t2};
    t1 > t2
}

fn dir_ext_param(parameter: &String) -> (Option<String>,Option<String>) {
    let path_end = parameter.rfind('/');
    if path_end.is_none() {
        return (None,None)
    }
    let pos = path_end.unwrap();
    let path = &parameter[0..pos];
    if pos == parameter.len(){
        return (Some(path.to_string()),None)
    }   
    let ext = &parameter[pos+1..];
    (Some(path.to_string()),Some(ext.to_string()))
}

// TODO implement as pushing in passing through vector
fn find_newer(dir1: &str, ext1: &str, dir2 : &Option<String>, ext2 : &Option<String>) -> Vec<String> {
    let mut result = Vec::new();
    let paths = fs::read_dir(&dir1);
    if paths.is_err() {
        return result
    }
    //println!{"find newerthen: {:?}/{:?} then {:?}/{:?}", &dir1, &ext1, &dir2, &ext2};
    let dir = paths.unwrap();
    for file1 in dir {
        let file1_path = &file1.as_ref().unwrap().path().into_os_string().into_string().unwrap();
        let file1_name = &file1.as_ref().unwrap().file_name().into_string().unwrap();
        if file1.unwrap().file_type().unwrap().is_dir() {
            let file2_str = match dir2 {
                Some(file2) => {
                    Some(format!{"{}/{}", file2, file1_name})
                },
                None => None
            };
            result = [result, find_newer(&file1_path, &ext1, &file2_str, &ext2)].concat();
        } else {  
            if file1_name.ends_with(ext1) {
                match dir2 {
                    Some(dir2) => {
                        let file2 = format!{"{}/{}{}", &dir2, &file1_name[0..file1_name.len()-ext1.len()], &ext2.as_ref().unwrap()};
                        
                        let t1 = last_modified(&file1_path);
                        let t2 = last_modified(&file2);
                        //println!{"comparing: {:?}:{:?}<>{:?}:{:?}", &file1_path, &t1, &file2, &t2};
                        if t2.is_none() || t1.unwrap() > t2.unwrap() {
                            //println!{"none or newer: {:?}>{:?}", t1.unwrap() ,t2.unwrap_or(std::time::UNIX_EPOCH) };
                            result.push(file1_path.to_string());
                        }
                    },
                    None => result.push(file1_path.to_string())
                }
            }
        }
    }
    result
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
        let dir = fs::read_dir(parent1).ok();
        if dir.is_none() {
            return None
        }
        for entry in dir.unwrap() {
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
             } else {
                let dir_entry_path = entry.path().into_os_string().into_string().unwrap();
                let last_dir = newest(&format!{"{}/*", dir_entry_path}) ;
                match last {
                    None => last = last_dir,
                    Some(time) => {
                        if let Some(time2) = last_dir {
                            if time2 > time {
                                last = last_dir;
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
    let result = fs::metadata(file);
    if result.is_err() {
        return None
    }
    let metadata = result.unwrap();
    if let Ok(time) = metadata.modified() {
        Some(time)
    } else {
        None
    }
}

fn matches(name: &str, filter: &str) -> bool {
    let star_pos = filter.find('*');
    match star_pos {
        None=> {
            return name == filter
        },
        Some (pos)=> {
            let len = name.len();
            match pos {
                0 => {
                    return name.ends_with(&filter[1..])
                },
                last if last == len - 1 => {
                    return name.starts_with(&filter[0..last])
                },
               _  => {
                    let start = &filter[0..pos];
                    let end = &filter[pos+1..];
                    return name.starts_with(&start) && name.ends_with(&end)
               }
            }
        } 
    }
}