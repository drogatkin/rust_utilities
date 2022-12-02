use std::collections::HashMap;
use lex::{Lexem, VarVal};

type FunCall = fn(Vec<Lexem>) -> Option<()>;

pub enum BlockType {
    Main,
    Target,
    Dependency,
    If,
    Expr,
    Scope,
}

pub struct GenBlock <'a>{
    name: Option<String>,
    block_type: BlockType,
    dir:Option<String>, // working directory
    flex: Option<String>,
    pub vars: HashMap<String, VarVal>,
    children: Vec<&'a GenBlock<'a>>,
    deps: Vec<&'a GenBlock<'a>>,
    parent: Option<Box<GenBlock<'a>>>,
}

impl GenBlock<'static> {
    pub fn new (block_type: BlockType) -> GenBlock<'static> {
        GenBlock {
            block_type : block_type,
            name : None,
            dir : None,
            flex : None,
            vars : HashMap::new(),
            children : Vec::new(),
            deps : Vec::new(),
            parent : None
        }

    }

}