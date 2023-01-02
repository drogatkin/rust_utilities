// lex analizer
//use std::io::{BufRead, BufReader};
use std::fs::File;
use std::io::{self, Read};
use std::collections::HashMap;

use log::Log;
use std::env;
use fun::{GenBlock, BlockType, GenBlockTup, vec_to_str};
use std::cell::RefCell;
use std::rc::{Rc, Weak};

const BUF_SIZE: usize = 256;

const MAX_LEX_LEN: usize = 4096;

#[derive(Debug, Clone, PartialEq)]
pub enum VarType {
    Generic,
    Property,
    Directory,
    Path,
    Array,
    File,
    Environment,
    Number,
    Date,
    Bool,
    Eval,
    Function,
    Url,
    RepositoryMaven,
    RepositoryRust
}

#[derive(PartialEq, Debug)]
pub enum Lexem {
    Variable(String), 
    Value(String), 
    Comment(String),
    Type(String),
    Range(usize, usize),
    Function(String),
    Parameter(String),
    BlockHdr(String),
    BlockEnd,
    EOF
}

#[derive(PartialEq, Debug, Copy, Clone)]
enum LexState {
    Begin,
    QuotedStart,
    InLex,
    InQtLex,
    InQtValue,
    Escape,
    EscapeValue,
    BlankOrEnd,
    RangeStart,
    Comment,
    InType,
    StartValue,
    InValue,
    RangeEnd,
    InParam,
    InParamBlank,
    InQtParam,
    StartParam,
    EndFunction,
    IgnoredBlankToEnd,
    BlankInValue,
    BlockStart,
    BlockEnd,
    EscapeParam,
    EndQtParam,
    //BlankInParam,
    End
}

#[derive(PartialEq, Debug, Copy, Clone)]
enum TemplateState {
    InVal,
    VarStart,  // $
    LeftBrack,
    RightBrack,
    InVar,
}

#[derive(PartialEq, Debug, Copy, Clone)]
enum HdrState {
    InType,
    NameStart,  // $
    WorkDiv,
    PathDiv,
    InName,
    InPath,
    InWork,
    InNameBlank,
    InWorkBlank,
    InPathBlank,
    InNameQt,
    InPathQt,
    InWorkQt,
}
 
#[derive(Debug)]
pub struct VarVal {
    pub val_type: VarType,
    pub value: String, // TODO make it enum based on type
    pub values: Vec<String>,
}

pub struct Reader {
    buf: [u8;BUF_SIZE],
    pos: usize,
    end: usize,
    line: u32,
    line_offset: u16,
    reader: File,
}

impl VarVal {
    pub fn from_string(str: &str) -> VarVal {
        VarVal{val_type: VarType::Generic, value: str.to_string(), values: Vec::new()}  
    }

    pub fn from_bool(boole: bool) -> VarVal {
        VarVal{val_type: VarType::Bool, value: if boole {"true".to_string()} else {"false".to_string()}, values: Vec::new()}  
    }

    pub fn from_i32(number: i32) -> VarVal {
        VarVal{val_type: VarType::Number, value: format!{"{}", number}, values: Vec::new()}  // 
    }

    pub fn from_vec(vec: &Vec<String>) -> VarVal {
        VarVal{val_type: VarType::Array, value: "".to_string(), values: vec.clone()}  
    }

    pub fn clone1(& self) -> VarVal {
        VarVal{val_type: self.val_type.clone(), value: self.value.clone(), values: self.values.clone()}
    }

    pub fn is_true(& self) -> bool {
        self.value == "true" || self.val_type == VarType::Array && self.values.len() > 0 
        || self.val_type == VarType::Number && self.value.parse::<i32>().is_ok() && self.value.parse::<i32>().unwrap() != 0
    }
}

impl Reader {
    fn next(&mut self) -> Option<char> {
        self.pos += 1;
        if self.pos >= self.end {
            self.end = self.reader.read(&mut self.buf).unwrap();
            
            match self.end {
               0 =>  return None,
               _ => ()
            }
            self.pos = 0;
        }
        self.line_offset += 1;
        // check if it can be UTF8
        let mut byte : u32 = self.buf[self.pos] as u32;
        if (byte & 0b1000_0000) != 0 { // UTF8
            let mut num_byte = 
                if (byte & 0b1111_0000) == 0b1111_0000 {
                    byte &= 0b0000_0111; 3
                } else if (byte & 0b1110_0000) == 0b1110_0000 {
                    byte &= 0b0000_1111; 2
                } else if (byte & 0b1100_0000) == 0b1100_0000 {
                    byte &= 0b0001_1111; 1
                } else {0};

            let mut c32 : u32 = byte;
            while num_byte > 0 {
                self.pos += 1;
                if self.pos >= self.end {
                    self.end = self.reader.read(&mut self.buf).unwrap();
                    if self.end == 0 {
                        return None;
                    }
                    self.pos = 0;
                }
                //println!("b-{:x}", c32);
                c32 =  (c32 << 6) | ((self.buf[self.pos] as u32) & 0b0011_1111);
                num_byte -= 1;
            }
            //println!("{:x}", c32);
            return Some(std::char::from_u32(c32).unwrap_or(std::char::REPLACEMENT_CHARACTER))
        }
        Some(char::from(self.buf[self.pos]))
    }
}

fn open(file: &str) -> io::Result<Reader> {
    let mut res = Reader {
        reader : File::open(file)?,
        pos : 0,
        end : 0,
        line : 1,
        line_offset : 0,
        buf : [0; 256],
    };
    Ok(res)
}

fn read_lex(log: &Log, reader: &mut Reader, mut state: LexState) -> (Lexem, LexState) {
    let mut buffer : [char; MAX_LEX_LEN] = [' '; MAX_LEX_LEN];
    let mut buf_fill: usize = 0;
    let mut last_nb = 0;
    let mut c1 = reader.next();

    while let Some(c) = c1 {
        match c {
            '"' => {
                match state {
                    LexState::Begin => state = LexState::QuotedStart,
                    LexState::InLex | LexState::InParam => {
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                    },
                    LexState::InQtLex => {
                        //let lexstr: String = buffer[0..buf_fill].iter().collect();
                        last_nb = buf_fill;
                        state = LexState::IgnoredBlankToEnd;
                    },
                    LexState::InQtValue => {
                        state = LexState::Begin;
                        return (Lexem::Value(buffer[0..buf_fill].iter().collect()), state);
                    },
                    LexState::Escape => {
                        state = LexState::InQtLex ;
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                    },
                    LexState::EscapeValue => {
                        state = LexState::InQtValue ;
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                    }
                    LexState::EscapeParam => {
                        state = LexState::InQtParam ;
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                    },
                    LexState::InQtParam => {
                        state = LexState::InParam;
                    }
                    LexState::StartParam => {
                        state = LexState::InQtParam;
                    },
                    LexState::StartValue => {
                        state = LexState::InQtValue;
                    },
                    LexState::Comment | LexState::BlankOrEnd => {
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                    },
                    _ => todo!("state: {:?} at {}", state, reader.line)
                }
                
            },
            ' ' | '\t' => {
                match state {
                    LexState::Begin | LexState::BlockStart => (),
                    LexState::QuotedStart => {
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                        state = LexState::InQtLex;
                    },
                    LexState::InLex => {
                        state = LexState::BlankOrEnd;
                        last_nb = buf_fill;
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                    },
                    LexState::Comment => {
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                    },
                    LexState::BlankOrEnd => {
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                    },
                    LexState::InParamBlank | LexState::InQtParam | LexState::InQtValue => {
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                    },
                    LexState::InParam => {
                        state = LexState::InParamBlank;
                        last_nb = buf_fill;
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                    },
                    LexState::InType => {
                        state = LexState::Begin;
                        return (Lexem::Type(buffer[0..buf_fill].iter().collect()), state);
                    },
                    LexState::StartValue | LexState::EndFunction | LexState::BlockEnd | LexState::IgnoredBlankToEnd => {

                    },
                    LexState::InValue => {
                        state = LexState::BlankInValue;
                        last_nb = buf_fill;
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                    },
                    LexState::BlankInValue => {
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                    },
                    LexState::StartParam => {

                    },
                    LexState::EscapeValue => {
                        buffer[buf_fill] = '\\';
                        buf_fill += 1;
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                        state = LexState::InQtValue;
                    },
                    _ => todo!("state: {:?} at {}", state, reader.line)
                }

            },
            '\\' => {
                match state {
                    LexState::InQtLex | LexState::QuotedStart => state = LexState::Escape,
                    LexState::InQtParam => state = LexState::EscapeParam,
                    LexState::InQtValue => state = LexState::EscapeValue,
                    LexState::Escape | LexState::InLex => {
                        state = LexState::InQtLex;
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                    },
                    LexState::Begin => {
                        state = LexState::InLex;
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                    },
                    LexState::BlankOrEnd => {
                        state = LexState::InLex;
                    },
                    LexState::StartParam | LexState::InParam | LexState::InParamBlank => {
                        state = LexState::InParam;
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                    },
                    LexState::Comment => {
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                    },
                    LexState::EscapeValue => {
                        buffer[buf_fill] = '\\';
                        buf_fill += 1;
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                        state = LexState::InQtValue;
                    },
                    LexState::End => break,
                    _ => todo!("state: {:?} at {}", state, reader.line)
                }
            },
            '#' => {
                match state {
                    LexState::Begin | LexState::EndFunction | LexState::Comment | LexState::BlockStart => {
                        state = LexState::Comment;
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                    },
                    LexState::BlankInValue => {
                        state = LexState::Comment;
                        return (Lexem::Value(buffer[0..last_nb].iter().collect()), state);
                    },
                    LexState::InQtValue | LexState::InQtParam | LexState::InQtLex => {
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                    },
                    _ => todo!("state: {:?} at {}", state, reader.line)
                }
            },
            '\n' | '\r' => {
                if c == '\n' {
                    reader.line += 1;
                    reader.line_offset = 0;
                }
                match state {
                    LexState::Comment => {
                        state = LexState::Begin;
                        return (Lexem::Comment(buffer[0..buf_fill].iter().collect()), state);
                    },
                    LexState::Begin | LexState::BlockStart | LexState::StartParam => {
                    },
                    LexState::InValue | LexState::BlankInValue => {
                        state = LexState::Begin;
                        return (Lexem::Value(buffer[0..buf_fill].iter().collect()), state);
                    },
                    LexState::EndFunction | LexState::BlockEnd => {
                        state = LexState::Begin; 
                    },
                    LexState::InType => {
                        state = LexState::Begin;
                        return (Lexem::Type(buffer[0..buf_fill].iter().collect()), state);
                    },
                   LexState::InQtParam | LexState::InParamBlank | LexState::InQtValue  => {
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                    },
                    LexState::InParam => {
                        state = LexState::InParamBlank;
                        last_nb = buf_fill;
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                    },
                    LexState::InLex | LexState::BlankOrEnd => {
                        state = LexState::BlankOrEnd;
                    },
                    _ => todo!("state: {:?} at {}", state, reader.line)
                }
            },
            '[' => {
                match state {
                    LexState::BlankOrEnd => state = LexState::RangeStart,
                    LexState::InQtLex | LexState::InQtValue | LexState::InQtParam => {
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                    },
                    LexState::InLex => {
                        state = LexState::RangeStart;
                        
                        //let lexstr: String = buffer[0..buf_fill].iter().collect();
                        return (Lexem::Variable(buffer[0..buf_fill].iter().collect()), state);
                    },
                    LexState::Comment | LexState::InValue | LexState::InParam => {
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                    },
                    LexState::InParamBlank | LexState::EndFunction | LexState::StartParam=> {
                        state = LexState::InParam;
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                    },
                    _ => todo!("state: {:?} at {}", state, reader.line)
                }
            },
            ']' => {
                match state {
                    LexState::Comment | LexState::InValue | LexState::InParam |
                    LexState::InQtLex | LexState::InQtValue | LexState::InQtParam => {
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                    },
                    _ => todo!("state: {:?} at {}", state, reader.line)
                }
            },
            '{' => {
                match state {
                    LexState::InValue | LexState::InQtParam | LexState::InQtLex | LexState::InParam | LexState::Comment | LexState::InQtValue => {
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                    },
                    LexState::InLex | LexState::BlankOrEnd => {
                        state = LexState::BlockStart;
                        return (Lexem::BlockHdr(buffer[0..buf_fill].iter().collect()), state);
                    },
                    LexState::BlockEnd | LexState::Begin => {
                        state = LexState::BlockStart;
                        return (Lexem::BlockHdr("".to_string()), state);
                    },
                    LexState::InParamBlank => {
                        state = LexState::InParam;
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                    },
                    _ => todo!()
                }
            },
            '}' => {
                //println!("{:?}", state);
                match state {
                    LexState::Begin  => {
                        state = LexState::BlockEnd;
                    
                        return (Lexem::BlockEnd, state);
                    },
                    LexState::InParam | LexState::InValue | LexState::InQtParam | LexState::Comment |
                    LexState::InQtValue => {
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                    },
                    LexState::InParamBlank => {
                        state = LexState::InParam;
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                    },
                    LexState::InLex | LexState::BlankOrEnd | LexState::EndFunction => {
                        state = LexState::BlockEnd;
                    // decide what to do with lex value ????
                        return (Lexem::BlockEnd, state);
                    },
                    _ => todo!("state: {:?}", state)
                }
            },
            ';' => {
                match state {
                    LexState::EndFunction | LexState::BlockEnd => {
                        state = LexState::Begin; 
                    }, 
                    LexState::Comment | LexState::InParam | LexState::InQtParam |
                    LexState::InQtValue | LexState::InQtLex => {
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                    } ,
                    LexState::InParamBlank => {
                        state = LexState::InParam;
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                    },
                    _ => todo!("state: {:?} at {}", state, reader.line)
                }
            },

            ':' => {
                match state {
                    LexState::BlankOrEnd => {
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                        state = LexState::InLex;
                    },
                    LexState::InValue | LexState::BlankInValue => {
                        state = LexState::InType;
                        last_nb = buf_fill;
                        return (Lexem::Value(buffer[0..last_nb].iter().collect()), state);
                    },
                    LexState::InParam | LexState::InLex | LexState::Comment | LexState::InQtParam |
                    LexState::InQtValue | LexState::InQtLex => {
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                    },
                    LexState::InParamBlank => {
                        state = LexState::InParam;
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                    },
                    _ => todo!()
                }
            },
            '=' => {
                match state {
                    LexState::BlankOrEnd | LexState::IgnoredBlankToEnd => {
                        state = LexState::StartValue; 
                        return (Lexem::Variable(buffer[0..last_nb].iter().collect()), state);
                    },
                    LexState::InLex => {
                        state = LexState::StartValue; 
                        return (Lexem::Variable(buffer[0..buf_fill].iter().collect()), state);
                    },
                    LexState::Comment | LexState::InParam | LexState::InQtParam |
                    LexState::InQtValue | LexState::InQtLex => {
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                    } ,
                    LexState::InParamBlank => {
                        state = LexState::InParam;
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                    },
                    _ => todo!("state: {:?} at {}", state, reader.line)
                }
            },
            '(' => { 
                match state {
                    LexState::InLex => {
                        state = LexState::StartParam; 
                        return (Lexem::Function(buffer[0..buf_fill].iter().collect()), state);
                    },
                    LexState::BlankOrEnd => {
                        state = LexState::StartParam; 
                        return (Lexem::Function(buffer[0..buf_fill].iter().collect()), state);
                    },
                    LexState::InValue | LexState::InParam | LexState::InQtParam | LexState::Comment | 
                    LexState::InQtValue | LexState::InQtLex => {
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                    },
                    _ => todo!("state: {:?} at {}", state, reader.line)
                }
            },
            ')' => {
                match state {
                    LexState::InParam  => {
                        state = LexState::EndFunction; 
                        return (Lexem::Parameter(buffer[0..buf_fill].iter().collect()), state);
                    },
                    LexState::InParamBlank  => {
                        state = LexState::EndFunction; 
                        return (Lexem::Parameter(buffer[0..last_nb].iter().collect()), state);
                    },
                    LexState::StartParam => {
                        state = LexState::EndFunction; 
                        return (Lexem::Parameter(buffer[0..buf_fill].iter().collect()), state);
                    },
                    LexState::InValue | LexState::InQtParam | LexState::Comment |
                    LexState::InQtValue | LexState::InQtLex => {
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                    },
                    _ => todo!("state: {:?} at {}", state, reader.line)
                }
            },
            '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {
                match state {
                 LexState::InParam |LexState::InValue | LexState::Comment | LexState::InLex | LexState::InQtParam |
                 LexState::InQtValue | LexState::InQtLex => {
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                    },
                    LexState::StartParam | LexState::InParamBlank => {
                        state = LexState::InParam;
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                    },
                    _ => todo!("state: {:?} at {}", state, reader.line)
                }
            },
            ',' => {
                match state {
                    LexState::InParam | LexState::InParamBlank => {
                        
                        state = LexState::StartParam; 
                        return (Lexem::Parameter(buffer[0..buf_fill].iter().collect()), state);
                    },
                    LexState::StartParam => {
                        state = LexState::InParam; 
                        return (Lexem::Parameter("".to_string() /* EMPTY */), state);
                    },
                    LexState::InValue | LexState::InQtParam | LexState::Comment |
                    LexState::InQtValue | LexState::InQtLex => {
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                    },
                    _ => todo!()
                }
            },
            '.' => {
                //println!("{:?}", state);
                match state {
                    LexState::InValue | LexState::InLex | LexState::InParam | LexState::Comment | LexState::InQtParam |
                    LexState::InQtValue | LexState::InQtLex => {
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                    },
                    LexState::BlankOrEnd => {
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                        state = LexState::InLex;
                    },
                    LexState::StartValue => {
                        state = LexState::InValue;
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                    },
                    LexState::StartParam => {
                        state = LexState::InParam;
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                    },
                    LexState::InParamBlank  => {
                        state = LexState::InParam;
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                    },
                    _ => todo!()
                }

            },
            _ => {
                match state {
                    LexState::InQtLex | LexState::InQtParam |
                    LexState::InQtValue => {
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                    },
                    LexState::InLex => {
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                    },
                    LexState::Begin | LexState::BlockStart | LexState::BlankOrEnd | LexState::BlockEnd => {
                        state = LexState::InLex;
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                    },
                    LexState::QuotedStart => {
                        state = LexState::InQtLex;
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                    },
                    LexState::Comment => {
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                    },
                    LexState::InType => {
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                    },
                    LexState::InValue | LexState::InParam => {
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                    },
                    LexState::StartValue => {
                        state = LexState::InValue;
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                    },
                    LexState::StartParam | LexState::InParamBlank => {
                        state = LexState::InParam;
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                    },
                    LexState::BlankInValue => {
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                        state = LexState::InValue;
                    },
                    LexState::EscapeParam => {
                        buffer[buf_fill] = '\\';
                        buf_fill += 1; 
                        buffer[buf_fill] = c;
                        buf_fill += 1; 
                        state = LexState::InQtParam;
                    },
                    LexState::EscapeValue  => {
                        buffer[buf_fill] = '\\';
                        buf_fill += 1; 
                        buffer[buf_fill] = c;
                        buf_fill += 1; 
                        state = LexState::InQtValue;
                    },
                    _ => todo!("state: {:?} at {}", state, reader.line)
                }
            }
        }
        c1 = reader.next();
    }
    match state {
        LexState::InQtLex => {
            log.error(&"Unexpected ending of the script file in quoted token");
            return (Lexem::EOF, state);
        },
        LexState::EndFunction => {
            //state = 
            return (Lexem::EOF, state);
        },
        LexState::InLex => {
            
        },
        LexState::Begin | LexState::End | LexState::BlockEnd => {
            return (Lexem::EOF, state);
        },
        LexState::InType => {
            state = LexState::End;
            return (Lexem::Type(buffer[0..buf_fill].iter().collect()), state);
        },
        LexState::Comment => {
            state = LexState::End;
            return (Lexem::Comment(buffer[0..buf_fill].iter().collect()), state);
        },
        _ => todo!("state: {:?} at {}", state, reader.line)
    }
    (Lexem::Variable(buffer[0..buf_fill].iter().collect()), state)
}

fn process_lex_header(log: &Log, value : &str, vars: &HashMap<String, VarVal>) -> Box<(String, String, String, String)> {
    let mut buf = [' ';4096* 12];

    let chars = value.chars();
    let mut state = HdrState::InType;
    let mut pos = 0;
    let mut blank_cnt = 0;
    let mut name : String = "".to_string();
    let mut lex_type : String = "".to_string();
    let mut work_dir : String = "".to_string();
    let mut path : String = "".to_string();
    for c in chars {
        match c {
            ' ' => {
                match state {
                    HdrState::InType => {
                        state = HdrState::NameStart;
                        lex_type = buf[0..pos].iter().collect();
                        pos = 0;
                    },
                    HdrState::PathDiv | HdrState::WorkDiv => {
                    },
                    HdrState::NameStart => (),
                    HdrState::InName => {
                        state = HdrState::InNameBlank;
                        blank_cnt = 1;
                    },
                    HdrState::InNameBlank | HdrState::InWorkBlank | HdrState::InPathBlank => {
                        blank_cnt += 1;
                    },
                   // HdrState::WorkDiv | HdrState::PathDiv => {},
                    HdrState::InWork => {
                        state = HdrState::InWorkBlank;
                        blank_cnt = 1;
                    },
                    HdrState::InPath => {
                        state = HdrState::InPathBlank;
                        blank_cnt = 1;
                    },
                     HdrState::InNameQt 
                    | HdrState::InPathQt | HdrState::InWorkQt => {
                        buf[pos] = c;
                        pos += 1;
                    },
                   // _ => todo!("state: {:?}", state)
                }

            },
            ':' => {
                match state {
                    HdrState::InType => {
                        state = HdrState::WorkDiv;
                        lex_type = buf[0..pos].iter().collect();
                        pos = 0;
                    },
                    HdrState::WorkDiv => {
                        state = HdrState::PathDiv;
                    },
                    HdrState::NameStart => {
                        state = HdrState::WorkDiv;
                    },
                    HdrState::InName => {
                        state = HdrState::WorkDiv;
                        name = buf[0..pos].iter().collect();
                        pos = 0;
                    },
                    HdrState::InWork => {
                        state = HdrState::PathDiv;
                        work_dir = buf[0..pos].iter().collect();
                        pos = 0;
                    },
                    HdrState::InNameBlank => {
                        name = buf[0..pos].iter().collect();
                        pos = 0;
                        state = HdrState::WorkDiv;
                    },
                    _ => todo!("state: {:?}", state)
                }

            },
            '"' => {
                match state {
                    HdrState::NameStart => {
                        state = HdrState::InNameQt;
                    },
                    HdrState::InNameQt => {
                        state = HdrState::InName;
                    },
                    HdrState::WorkDiv => {
                        state = HdrState::InWorkQt;
                    },
                    HdrState::InWorkQt => {
                        state = HdrState::InWork;
                    }
                    HdrState::PathDiv => {
                        state = HdrState::InPathQt;
                    },
                    HdrState::InPathQt => {
                        state = HdrState::InPath;
                    }
                    _ => todo!("header state: {:?}", state)
                }
            },
            _ => {
                match state {
                    HdrState::WorkDiv => {
                        state = HdrState::InWork;
                        buf[pos] = c;
                        pos += 1;
                    },
                    HdrState::PathDiv => {
                        state = HdrState::InPath;
                        buf[pos] = c;
                        pos += 1;
                    },
                    HdrState::NameStart | HdrState::InName => {
                        state = HdrState::InName;
                        buf[pos] = c;
                        pos += 1;
                    },
                    
                    HdrState::InNameBlank => {
                        state = HdrState::InName;
                        for _ in 0..blank_cnt {
                            buf[pos] = ' ';
                            pos += 1;
                        }
                        buf[pos] = c;
                        pos += 1;
                    },
                    HdrState::InWorkBlank => {
                        state = HdrState::InWork;
                        for _ in 0..blank_cnt {
                            buf[pos] = ' ';
                            pos += 1;
                        }
                        buf[pos] = c;
                        pos += 1;
                    },
                    HdrState::InPathBlank => {
                        state = HdrState::InPath;
                        for _ in 0..blank_cnt {
                            buf[pos] = ' ';
                            pos += 1;
                        }
                        buf[pos] = c;
                        pos += 1;
                    },
                    HdrState::InWork | HdrState::InPath | HdrState::InNameQt | HdrState::InType 
                    | HdrState::InPathQt | HdrState::InWorkQt => {
                        buf[pos] = c;
                        pos += 1;
                    },
                    //_ => todo!("state: {:?}", state)
                }
            }
        }
    }
    match state {
        HdrState::InType => {
            lex_type = buf[0..pos].iter().collect();
        },
        HdrState::InName | HdrState::InNameBlank => {
            name = buf[0..pos].iter().collect();
        },
        HdrState::InWork |  HdrState::PathDiv | HdrState::InWorkBlank => {
            work_dir = buf[0..pos].iter().collect();
        },
        HdrState::InPath | HdrState::InPathBlank=> {
            path = buf[0..pos].iter().collect();
        },
        HdrState::NameStart => (),
        _ => todo!("state: {:?}", state)
    }
    Box::new((lex_type.to_string(), name.to_string(), work_dir.to_string(), path.to_string()))
}

pub fn process_template_value(log: &Log, value : &str, vars: &GenBlock, res_prev: &Option<VarVal>) -> Box<String> {
    let mut buf = [' ';4096* 1];
    let mut buf_var = [' ';128]; // buf for var name
    let mut name_pos = 0;
    let chars = value.chars();
    let mut pos = 0;
    let mut state = TemplateState::InVal;
    let mut was_replacement = false;
    for c in chars {
        match c {
            '$' => {
                match state {
                    TemplateState::InVal  => state = TemplateState::VarStart,
                    TemplateState::VarStart => {
                        buf[pos] = c;
                        pos += 1;
                    },
                    TemplateState::InVar =>
                    {
                        buf_var[name_pos] = c;
                        name_pos += 1;
                    },
                    _ => todo!()
                }
            },
            '{' => {
                match state {
                    TemplateState::VarStart => state = TemplateState::InVar,
                    TemplateState::InVal  => {
                        buf[pos] = c;
                        pos += 1;
                    },
                    TemplateState::InVar => {
                        buf_var[name_pos] = c;
                        name_pos += 1;
                    },
                    _ => todo!()
                }
            },
            '}' => {
                match state {
                    TemplateState::VarStart => {
                        state = TemplateState::InVal;
                        buf[pos] = '$';
                        pos += 1;
                        buf[pos] = c;
                        pos += 1;
                    },
                    TemplateState::InVal  => {
                        buf[pos] = c;
                        pos += 1;
                    },
                    TemplateState::InVar => {
                        state = TemplateState::InVal;
                        let var : String = buf_var[0..name_pos].iter().collect();
                        //println!("looking {}", var);
                        // check name for ~~ and then use global thread local
                        let res = if var == "~~" {
                            match res_prev {
                                None => None,
                                Some(prev) => Some(prev.clone1())
                            }
                        } else {vars.search_up( &var)};
                        match res {
                            Some(var) => {
                               // println!("found {:?}", var);
                               // TODO avoid replacement in an infinitive loop
                               match var.val_type {
                                    VarType::Environment => {
                                      //  println!("looking for {} in env", var.value);
                                        let _env = match env::var(var.value.to_string()) {
                                            Ok(val) => {
                                                for vc in val.chars() {
                                                    buf[pos] = vc;
                                                    pos += 1;
                                                }
                                            },
                                            Err(_e) => {
                                                for vc in var.value.chars() {
                                                    buf[pos] = vc;
                                                    pos += 1;
                                                } 
                                            },
                                        };
                                    },
                                    VarType::Array => {
                                        let chars = vec_to_str(&var.values);
                                        for vc in chars.chars() {
                                            buf[pos] = vc;
                                            pos += 1;
                                        }
                                    },
                                    _ => {
                                        for vc in var.value.chars() {
                                            buf[pos] = vc;
                                            pos += 1;
                                        }
                                    }
                               }
                               was_replacement = true;
                            },
                            None => {
                                buf[pos] = '$';
                                pos += 1;
                                buf[pos] = '{';
                                pos += 1;
                                for vc in 0..name_pos {
                                    buf[pos] = buf_var[vc];
                                     pos += 1;
                                }
                                buf[pos] = '}';
                                pos += 1;
                            }
                        }
                        name_pos = 0;
                    },
                    _ => todo!()
                }
            },
            _ => {
                match state {
                    TemplateState::InVal  => {
                        buf[pos] = c;
                        pos += 1;
                    },
                    TemplateState::InVar => {
                        buf_var[name_pos] = c;
                        name_pos += 1;
                    },
                    TemplateState::VarStart => {
                        buf[pos] = '$';
                        pos += 1;
                        buf[pos] = c;
                        pos += 1;
                    },
                    _ => todo!()
                }
            }
        }
    }
    // temporay hack (no loop detection )
    let expanded_val:String = buf[0..pos].iter().collect();
    if was_replacement {
        log.debug(&format!{"expanding {}", &expanded_val});
        return process_template_value(&log, &expanded_val, &vars, &res_prev)
    }
    Box::new(expanded_val)
}

fn process_array_value(log: &Log, value : &str) -> Result<Vec<String>, String> {
    let mut buf = [' ';4096* 1];
    let mut state: LexState = LexState::Begin;
    let chars = value.chars();
    let mut res : Vec<String> = Vec::new();
    let mut blank_pos = 0;
    let mut pos = 0;
    for c in chars {
        match c {
            '[' => {
                match state {
                    LexState::Begin  => state = LexState::RangeStart,
                    LexState::InParam | LexState::InQtParam | LexState::RangeStart => {
                        buf[pos] = c;
                        pos += 1;
                    },
                    _ => todo!("state: {:?}", state)
                }
            },
            ']' => {
                match state {
                 LexState::InQtParam  => {
                        buf[pos] = c;
                        pos += 1;
                    },
                    LexState::InParam => {
                        let param = buf[0..pos].iter().collect();
                        res.push(param);
                        return Ok(res)
                    },
                    LexState::EndQtParam => {
                        return Ok(res)
                    } ,
                    LexState::RangeStart => {
                        return Ok(res)
                    },
                    _ => todo!("state: {:?}", state)
                }
            },
            '"' => {
                match state {
                    LexState::RangeStart => {
                        state = LexState::InQtParam;
                    },
                    LexState::InParam   => {
                        buf[pos] = c;
                        pos += 1;
                    },
                    LexState::InQtParam => {
                        state = LexState::EndQtParam;
                        let param = buf[0..pos].iter().collect();
                        res.push(param);
                    },
                    LexState::EscapeParam => {
                        buf[pos] = c;
                        pos += 1;
                        state = LexState::InQtParam;
                    },
                    _ => todo!("state: {:?}", state)
                }
            },
            '\\' => {
                match state {
                    LexState::InParam  => {
                        buf[pos] = c;
                        pos += 1;
                    },
                    LexState::InQtParam  => {
                        state = LexState::EscapeParam;
                    },
                    _ => todo!("state: {:?}", state)
                }
            },
            ',' => {
                match state {
                    LexState::InParam  => {
                        let param = buf[0..pos].iter().collect();
                        res.push(param);
                        state = LexState:: StartParam;
                    },
                    LexState::InQtParam  => {
                        buf[pos] = c;
                        pos += 1;
                    },
                    LexState::EndQtParam => {
                        state = LexState:: StartParam;
                    }, 
                    _ => todo!("state: {:?}", state)
                }
            },
            ' ' | '\t' | '\n' | '\r' => {
                match state {
                    LexState::InQtParam  => {
                        buf[pos] = c;
                        pos += 1;
                    },
                    LexState::InParam  => {
                        blank_pos = pos;
                        buf[pos] = c;
                        pos += 1;
                        state = LexState::BlankOrEnd;
                    },
                    LexState::EndQtParam => { },
                    _ => todo!("state: {:?}", state)
                }
            },
            _ => {
                match state {
                    LexState::InParam | LexState::InQtParam  => {
                        buf[pos] = c;
                        pos += 1;
                    },
                    LexState::EscapeParam => {
                        buf[pos] = '\\';
                        pos += 1;
                        buf[pos] = c;
                        pos += 1;
                    },
                    LexState::BlankOrEnd => {
                        state = LexState::InParam;
                    },
                    _ => todo!("state: {:?}", state)
                }
            }
        }
    }
    Err(value.to_string())
}

pub fn process(log: &Log, file: & str, block: GenBlockTup) -> io::Result<()> {
    let mut all_chars =  match  open(file) {
        Err(e) => return Err(e),
        Ok(r) => r,
    };
    
    //let mut func_stack = Vec::new();
    //let mut block_stack : Vec<&mut GenBlock> = Vec::new();
    let mut state = LexState::Begin;
    // current block
    let mut scoped_block = block; 
    let mut current_name = "".to_string();
    while state != LexState::End {
        let ( lex, mut state2) = read_lex(log, &mut all_chars, state);
        log.debug(&format!("Lex: {:?}, line: {}/{}, state: {:?}", lex, all_chars.line, all_chars.line_offset, state2));
        match lex {
            Lexem::EOF => {
                state2 = LexState::End;
            },
            Lexem::Variable(name) => {
                current_name = name.to_string();
            },
            Lexem::Value(value) => {
               // consider it can be an array in form [v1,v2,...vn]
               let c_b = 
                if value.starts_with("[") && value.ends_with("]") {
                    let res = process_array_value(&log, &value);
                    if res.is_ok() {
                        VarVal::from_vec(&res.unwrap())
                    } else {VarVal::from_string(&value)}
                } else {VarVal::from_string(&value)}
                ;
                scoped_block.0.as_ref().borrow_mut().vars.insert(current_name.to_string(), c_b);
            },
            Lexem::Function(name) => {
                // name can be function + main argument
                let (type_hdr,name,work,path) = *process_lex_header(&log, &name, &scoped_block.0.as_ref().borrow_mut().vars) ;
                let mut func = GenBlock::new(BlockType::Function);
                //fun::GenBlockTup(Rc::new(RefCell::new(GenBlock::new(BlockType::Function))));
                func.name = Some(type_hdr);
                func.flex = if name.is_empty() {None} else { Some(name)};
                func.dir = if work.is_empty() {None} else { Some(work)};
                scoped_block = scoped_block.add(GenBlockTup(Rc::new(RefCell::new(func))));
            },
            Lexem::Type(var_type) => {
                let mut bl = scoped_block.0.as_ref().borrow_mut();
               // println!("name {} in block {:?}", &current_name, bl.block_type);
                match bl.vars.get(&current_name.to_string()) {
                    Some(var) => { 
                        match var_type.as_str() {
                            "file" => {
                                let c_b = VarVal{val_type:VarType::File, value:var.value.clone(), values: Vec::new()};
                                bl.vars.insert(current_name.to_string(), c_b);
                            },
                            "env" => {
                              //  println!("env {} in {:?}", var.value, bl.block_type);
                                let c_b = VarVal{val_type:VarType::Environment, value:var.value.clone(), values: Vec::new()};
                                bl.vars.insert(current_name.to_string(), c_b);
                            },
                            "rep-rust" => {
                                //let at_pos = 
                                //  println!("env {} in {:?}", var.value, bl.block_type);
                                  let c_b = VarVal{val_type:VarType::RepositoryRust, value:var.value.clone(), values: Vec::new()};
                                  bl.vars.insert(current_name.to_string(), c_b);
                              },
                              "rep-maven" => {
                                //let at_pos = 
                                //  println!("env {} in {:?}", var.value, bl.block_type);
                                  let c_b = VarVal{val_type:VarType::RepositoryMaven, value:var.value.clone(), values: Vec::new()};
                                  bl.vars.insert(current_name.to_string(), c_b);
                              },
                            _ => ()
                        }
                        
                    },
                    _ => ()
                }
            },
            Lexem::Parameter(value) => { // collect all parameters and then process function call
                let value1 = value.trim().to_string();
               // println!("trimmed val {}", value1.trim());
               let mut name : Option<String> = None;
               {
               let mut rl_block = scoped_block.0.as_ref().borrow_mut();
                // push param in params vec
                rl_block.params.push(value1);

                  if let Some(name1) = &rl_block.name {
                    name = Some(name1.clone());
                  }
               }  
                if state2 == LexState::EndFunction {
                    log.debug(&format!("end func for {:?}", name));
                    if let Some(name) = name {
                        match name.as_str() {
                            "display" => {
                                let parent_scoped_block = scoped_block.parent().unwrap();
                                if parent_scoped_block.0.borrow().block_type == BlockType::Main {
                                    println!("{}", *process_template_value(&log, &value, &scoped_block.0.as_ref().borrow_mut(), &None));
                                }
                            },
                            "include" => {
                                match scoped_block.search_up(&value) {
                                    Some(var) => {
                                      // println!("found {:?}", var);
                                       match var.val_type {
                                            VarType::File => {
                                                let clone_var = var.value.clone();
                                                let parent_scoped_block = scoped_block.parent();
                                                if let Some(block) = parent_scoped_block {
                                                    process(log, clone_var.as_str(), block.clone())?;
                                                }
                                                
                                            },
                                            _ => ()
                                       }
                                    },
                                    None => {
                                    }
                                }
                            },
                            _ => ()
                        }
                    } 
                    let parent_block =  Rc::clone(&scoped_block.0);
                    let pp2 = parent_block.as_ref().borrow_mut();
                    let pp1 = pp2.parent.as_ref().unwrap();
                    let pp = &pp1.0;
                    scoped_block = GenBlockTup(Rc::clone(pp));
                   
                } 
 
            },
            Lexem::BlockHdr(value) => { 
                // parse header and push in block stack
               // let mut test_block = GenBlock::new(BlockType::Target);
                let (type_hdr,name,work,path) = *process_lex_header(&log, &value, &scoped_block.0.as_ref().borrow_mut().vars) ;
                log.debug(&format!("Type: {}, name: {}, work dir: '{}', path; '{}'", type_hdr,name,work,path));
                match type_hdr.as_str() {
                    "target" => {
                        let mut inner_block = GenBlock::new(BlockType::Target);
                        inner_block.name = Some(name);
                        inner_block.dir = Some(work);
        
                        scoped_block =  scoped_block.add(GenBlockTup(Rc::new(RefCell::new(inner_block))));
                    },
                    "eq" => {
                        let  inner_block = GenBlock::new(BlockType::Eq);
        
                        scoped_block =  scoped_block.add(GenBlockTup(Rc::new(RefCell::new(inner_block))));
                    },
                    "if" => {
                        let inner_block = GenBlock::new(BlockType::If);
        
                        scoped_block =  scoped_block.add(GenBlockTup(Rc::new(RefCell::new(inner_block))));
                    },
                    "then" => {
                        let inner_block = GenBlock::new(BlockType::Then);
        
                        scoped_block =  scoped_block.add(GenBlockTup(Rc::new(RefCell::new(inner_block))));
                    },
                    "neq" => {
                        let inner_block = GenBlock::new(BlockType::Neq);
        
                        scoped_block =  scoped_block.add(GenBlockTup(Rc::new(RefCell::new(inner_block))));
                    },
                    "else" => {
                        let inner_block = GenBlock::new(BlockType::Else);
        
                        scoped_block =  scoped_block.add(GenBlockTup(Rc::new(RefCell::new(inner_block))));
                    },
                    "or" => {
                        let inner_block = GenBlock::new(BlockType::Or);
        
                        scoped_block =  scoped_block.add(GenBlockTup(Rc::new(RefCell::new(inner_block))));
                    },
                    "and" => {
                        let inner_block = GenBlock::new(BlockType::And);
        
                        scoped_block =  scoped_block.add(GenBlockTup(Rc::new(RefCell::new(inner_block))));
                    },
                    "not" => {
                        let inner_block = GenBlock::new(BlockType::Not);
        
                        scoped_block =  scoped_block.add(GenBlockTup(Rc::new(RefCell::new(inner_block))));
                    },
                    "for" => {
                        let mut inner_block = GenBlock::new(BlockType::For);
                        inner_block.name = Some(name);
                        inner_block.dir = Some(work);
                        inner_block.flex = Some(path);
                        scoped_block =  scoped_block.add(GenBlockTup(Rc::new(RefCell::new(inner_block))));
                    },
                    "" => {
                        let inner_block = GenBlock::new(BlockType::Scope);
        
                        scoped_block =  scoped_block.add(GenBlockTup(Rc::new(RefCell::new(inner_block))));// *scoped_block = GenBlock::new(BlockType::Scope);
                    },
                    "dependency" => {
                        let inner_block = GenBlock::new(BlockType::Dependency);
        
                        scoped_block =  scoped_block.add_dep(GenBlockTup(Rc::new(RefCell::new(inner_block))));
                    },
                    _ => todo!("unknown block {}", type_hdr)
                }
                
            },
            Lexem::BlockEnd => {
                //println!(" current {:?}", scoped_block.0.borrow_mut().block_type);
                let parent_block =  Rc::clone(&scoped_block.0);
                let pp2 = parent_block.as_ref().borrow_mut();
                log.debug(&format!("Closing block {:?} at {}/{}", pp2.block_type, all_chars.line, all_chars.line_offset));
                match pp2.parent.as_ref() {
                    None => log.error(&format!("Unmatched block {:?} closing found at {}:{}", pp2.block_type, all_chars.line, all_chars.line_offset)),
                    Some(pp1) => {
                        let pp = &pp1.0;
                        scoped_block = GenBlockTup(Rc::clone(pp));
                    }
                }
                
            },
            Lexem::Comment(value) => {
                log.debug(&format!("Commentary: {}, line: {}/{}", value, all_chars.line, all_chars.line_offset));
            },
            _ => todo!("unprocessed lexem {:?}", lex)
        }
        state = state2;
    }
    
    Ok(())
}