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
    Eq,
}

pub struct GenBlock <'a>{
    pub name: Option<String>,
    pub block_type: BlockType,
    pub dir:Option<String>, // working directory
    pub flex: Option<String>,
    pub vars: HashMap<String, VarVal>,
    pub children: Vec<&'a GenBlock<'a>>,
    pub deps: Vec<&'a GenBlock<'a>>,
    pub parent: Option<Box<GenBlock<'a>>>,
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

pub struct GenFun {
    pub name: String,
    pub params: Vec<String>,

}