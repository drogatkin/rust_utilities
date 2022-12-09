use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::{Rc, Weak};
use lex::{Lexem, VarVal, VarType};
use std::io::{self, Write};

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
        println!("parrent --a");
        let bl = self.0.borrow();
        println!("parrent -in {:?}", bl.name);
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
                                    exec_target(&target_bor);
                                },
                                _ => ()
                            }
                        },
                        _ => todo!("function: {:?}", dep_block.name)
                    } 
                },
                _ => todo!()
            }
        }
        false
    }

    

    pub fn get_top_block(& self) -> GenBlockTup {
        let mut curr =self.clone();
        loop {
            let mut parent = parent(curr.clone());
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

pub fn parent(node: GenBlockTup) -> Option<GenBlockTup> {
    let bl = node.0.borrow();
    if let Some(parent) = &bl.parent {
        Some(parent.clone())
    } else {
        None
    }
}

pub fn run(block: GenBlockTup, targets: &Vec<String>) -> io::Result<()> {
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