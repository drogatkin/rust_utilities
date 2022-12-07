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

    pub fn search_up(&self, name: &String) -> Option<VarVal> {
        let current_bl = self.0.borrow_mut();
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

    pub fn parent(&self) -> Option<GenBlockTup> {
        if let Some(parent) = &self.0.borrow_mut().parent {
            Some(parent.clone())
        } else {
            None
        }
    }

}

pub fn run(block: GenBlockTup, targets: &Vec<String>, arguments: &Vec<String>) -> io::Result<()> {
    let mut naked_block = block.0.borrow_mut();
    let target = if targets.len() == 0 {
        naked_block.children.reverse();
        let mut tar_name = String::from("");
        for ch in &naked_block.children {
            let ch_block = ch.0.borrow();
            if ch_block.block_type == BlockType::Target {
                tar_name = ch_block.name.as_ref().unwrap().to_string();
                break ;
            }
        }
        tar_name
    } else {
        targets[0].to_string()
    };
    println!("processing for {} in {}", target, naked_block.children.len());
    Ok(())
}