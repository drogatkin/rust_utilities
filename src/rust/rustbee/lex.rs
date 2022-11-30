// lex analizer
//use std::io::{BufRead, BufReader};
use std::fs::File;
use std::io::{self, Read};
use std::collections::HashMap;

use log::Log;
use std::env;

const BUF_SIZE: usize = 256;

const MAX_LEX_LEN: usize = 4096;

#[derive(Debug)]
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
    Eval,
    Function,
    Url,
    RepositoryMaven,
    RepositoryRust
}

#[derive(PartialEq, Debug)]
enum Lexem {
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
    Escape,
    BlankOrEnd,
    RangeStart,
    Comment,
    InType,
    StartValue,
    InValue,
    EndValue,
    RangeEnd,
    InParam,
    InQtParam,
    StartParam,
    EndFunction,
    IgnoredBlankToEnd,
    BlankInValue,
    BlockStart,
    BlockEnd,
    EscapeParam,
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
 
#[derive(Debug)]
pub struct VarVal {
    val_type: VarType,
    value: String,
    values: Vec<String>,
}

pub struct Reader {
    buf: [u8;BUF_SIZE],
    pos: usize,
    end: usize,
    reader: File,
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
        Some(char::from(self.buf[self.pos]))
    }
}

fn open(file: &str) -> io::Result<Reader> {
    let mut res = Reader {
        reader : File::open(file)?,
        pos : 0,
        end : 0,
        buf : [0; 256],
    };
    Ok(res)
}

fn read_lex(log: &Log, reader: &mut Reader, mut state: LexState) -> (Lexem, LexState) {
    let mut buffer : [char; MAX_LEX_LEN] = [' '; MAX_LEX_LEN];
    let mut buf_fill: usize = 0;
    let mut blank_counter = 0;
    let mut c1 = reader.next();
    //let mut state = LexState::Begin; //*state1;
    //let mut state = state1;
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
                        let lexstr: String = buffer[0..buf_fill].iter().collect();
                        state = LexState::IgnoredBlankToEnd;
                    },
                    LexState::Escape => {
                        state = LexState::InQtLex ;
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                    },
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
                    LexState::Comment => {
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                    },
                    _ => todo!()
                }
                
            },
            ' ' | '\t' => {
                match state {
                    LexState::Begin | LexState::BlockStart=> (),
                    LexState::QuotedStart => {
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                        state = LexState::InQtLex;
                    },
                    LexState::InLex => {
                        state = LexState::BlankOrEnd;
                        blank_counter = 1;
                        //let lexstr: String = buffer[0..buf_fill].iter().collect();
                        //return (Lexem::Variable(buffer[0..buf_fill].iter().collect(), "".to_string(), "".to_string()), state);
                    },
                    LexState::Comment => {
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                    },
                    LexState::BlankOrEnd => {
                        blank_counter += 1;
                    },
                    LexState::InParam | LexState::InQtParam=> {
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                    },
                    LexState::StartValue | LexState::EndFunction => {

                    },
                    LexState::InValue => {
                        state = LexState::BlankInValue;
                        blank_counter = 1;
                        //return (Lexem::Value(buffer[0..buf_fill].iter().collect()), state);
                    },
                    LexState::BlankInValue => {
                        blank_counter += 1;
                    },
                    LexState::StartParam => {

                    },
                    _ => todo!()
                }

            },
            '\\' => {
                match state {
                    LexState::InQtLex | LexState::QuotedStart => state = LexState::Escape,
                    LexState::InQtParam => state = LexState::EscapeParam,
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
                        for _ in 0..blank_counter {
                            buffer[buf_fill] = ' ';
                            buf_fill += 1; 
                        }
                        blank_counter = 0;
                        state = LexState::InLex;
                    },
                    LexState::StartParam | LexState::InParam => {
                        state = LexState::InParam;
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                    },
                    LexState::Comment => {
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                    },
                    LexState::End => break,
                    _ => todo!()
                }
            },
            '#' => {
                match state {
                    LexState::Begin | LexState::EndFunction => {
                        state = LexState::Comment;
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                    },
                    _ => todo!()
                }
            },
            '\n' | '\r' => {
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
                    LexState::InParam | LexState::InQtParam => {
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                    },
                    _ => todo!()
                }
            },
            '[' => {
                match state {
                    LexState::BlankOrEnd => state = LexState::RangeStart,
                    LexState::InQtLex => {
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                    },
                    LexState::InLex => {
                        state = LexState::RangeStart;
                        
                        //let lexstr: String = buffer[0..buf_fill].iter().collect();
                        return (Lexem::Variable(buffer[0..buf_fill].iter().collect()), state);
                    },
                    LexState::Comment => {
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                    },
                    _ => todo!()
                }
            },
            ']' => {

            },
            '{' => {
                match state {
                    LexState::InQtLex | LexState::InParam => {
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
                    LexState::Comment => {
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                    },
                    LexState::InValue | LexState::InQtParam => {
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                    },
                    _ => todo!()
                }
            },
            '}' => {
                println!("{:?}", state);
                match state {
                    LexState::Begin  => {
                        state = LexState::BlockEnd;
                    
                        return (Lexem::BlockEnd, state);
                    },
                    LexState::InParam  => {
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                    },
                    LexState::InLex => {
                        state = LexState::BlockEnd;
                    // decide what to do with lex value ????
                        return (Lexem::BlockEnd, state);
                    },
                    _ => todo!()
                }
            },
            ';' => {

            },

            ':' => {
                // println!("{:?}", state);
                match state {
                    LexState::BlankOrEnd => {
                        for _ in 0..blank_counter {
                            buffer[buf_fill] = ' ';
                            buf_fill += 1; 
                            buffer[buf_fill] = c;
                            buf_fill += 1;
                        }
                        blank_counter = 0;
                        state = LexState::InLex;
                    },
                    LexState::InValue | LexState::BlankInValue => {
                        state = LexState::InType;
                        return (Lexem::Value(buffer[0..buf_fill].iter().collect()), state);
                    },
                    LexState::InParam | LexState::InLex => {
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                    },
                    _ => todo!()
                }
            },
            '=' => {
                match state {
                    LexState::InLex | LexState::BlankOrEnd=> {
                        
                        state = LexState::StartValue; 
                        return (Lexem::Variable(buffer[0..buf_fill].iter().collect()), state);
                    },
                    _ => todo!()
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
                    LexState::InValue | LexState::InParam | LexState::InQtParam | LexState::Comment => {
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                    },
                    _ => todo!()
                }
            },
            ')' => {
                match state {
                    LexState::InParam  => {
                        
                        state = LexState::EndFunction; 
                        return (Lexem::Parameter(buffer[0..buf_fill].iter().collect()), state);
                    },
                    LexState::StartParam => {
                        state = LexState::EndFunction; 
                        return (Lexem::Parameter(buffer[0..buf_fill].iter().collect()), state);
                    },
                    LexState::InValue | LexState::InQtParam | LexState::Comment => {
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                    },
                    _ => todo!()
                }
            },
            '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {
                match state {
                 LexState::InParam | LexState::InQtParam | LexState::InValue => {
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                    },
                    _ => todo!()
                }
            },
            ',' => {
                match state {
                    LexState::InParam => {
                        
                        state = LexState::StartParam; 
                        return (Lexem::Parameter(buffer[0..buf_fill].iter().collect()), state);
                    },
                    LexState::StartParam => {
                        state = LexState::InParam; 
                        return (Lexem::Parameter("".to_string() /* EMPTY */), state);
                    },
                    LexState::InValue | LexState::InQtParam => {
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                    },
                    _ => todo!()
                }
            },
            '.' => {
                //println!("{:?}", state);
                match state {
                    LexState::InValue | LexState::InLex | LexState::InParam | LexState::InQtParam => {
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                    },
                    LexState::BlankOrEnd => {
                        for _ in 0..blank_counter {
                            buffer[buf_fill] = ' ';
                            buf_fill += 1; 
                            buffer[buf_fill] = c;
                            buf_fill += 1; 
                        }
                        blank_counter = 0;
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
                   
                    _ => todo!()
                }

            },
            _ => {
                //println!("{:?}", state);
                match state {
                    LexState::InQtLex | LexState::InQtParam => {
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                    },
                    LexState::InLex => {
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                    },
                    LexState::Begin | LexState::BlockStart => {
                        state = LexState::InLex;
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
                    LexState::InValue => {
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                    },
                    LexState::StartValue => {
                        state = LexState::InValue;
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                    },
                    LexState::StartParam | LexState::InParam => {
                        state = LexState::InParam;
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                    },
                    LexState::BlankOrEnd => {
                        for _ in 0..blank_counter {
                            buffer[buf_fill] = ' ';
                            buf_fill += 1; 
                            buffer[buf_fill] = c;
                            buf_fill += 1; 
                        }
                        blank_counter = 0;
                        state = LexState::InLex;
                    },
                    LexState::BlankInValue => {
                        for _ in 0..blank_counter {
                            buffer[buf_fill] = ' ';
                            buf_fill += 1; 
                            buffer[buf_fill] = c;
                            buf_fill += 1;
                        }
                        blank_counter = 0;
                        state = LexState::InValue;
                    },
                    LexState::EscapeParam => {
                        buffer[buf_fill] = '\\';
                        buf_fill += 1; 
                        buffer[buf_fill] = c;
                        buf_fill += 1; 
                    },
                    _ => todo!()
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
        LexState::Begin | LexState::End  => {
            return (Lexem::EOF, state);
        },
        LexState::InType => {
            state = LexState::End;
            return (Lexem::Type(buffer[0..buf_fill].iter().collect()), state);
        },
        _ => todo!()
    }
    (Lexem::Variable(buffer[0..buf_fill].iter().collect()), state)
}

fn process_template_value(log: &Log, value : &str, vars: &HashMap<String, VarVal>) -> Box<String> {
    let mut buf = [' ';4096* 12];
    let mut buf_var = [' ';128]; // buf for var name
    let mut name_pos = 0;
    let chars = value.chars();
    let mut pos = 0;
    let mut state = TemplateState::InVal;
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
                       // println!("lookinf {}", var);
                        match vars.get(&var) {
                            Some(var) => {
                               // println!("found {:?}", var);
                               match var.val_type {
                                    VarType::Environment => {
                                       // println!("looking for {} in env", var.value);
                                        let env = match env::var(var.value.to_string()) {
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
                                    _ => {
                                        for vc in var.value.chars() {
                                            buf[pos] = vc;
                                            pos += 1;
                                        }
                                    }
                               }
                                
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
    Box::new(buf[0..pos].iter().collect())
}

pub fn process(log: &Log, file: & str, args: &Vec<String>, vars_inscope: &mut HashMap<String, VarVal>) -> io::Result<()> {
    let mut all_chars =  match  open(file) {
        Err(e) => return Err(e),
        Ok(r) => r,
    };
    
    let mut func_stack = Vec::new();
    let mut state = LexState::Begin;
    let mut current_name = "".to_string();
    while state != LexState::End {
        let (mut lex, mut state2) = read_lex(log, &mut all_chars, state);
        log.debug(&format!("Lex: {:?}, state: {:?}", lex, state2));
        match lex {
            Lexem::EOF => {
                state2 = LexState::End;
            },
            Lexem::Variable(name) => {
                current_name = name.to_string();
            },
            Lexem::Value(value) => {
                state = LexState::End;
                
                let c_b = VarVal{val_type:VarType::Generic, value:value, values: Vec::new()};
                vars_inscope.insert(current_name.to_string(), c_b);
            },
            Lexem::Function(name) => {
                
                match name.as_str() {
                    "display" | "include" => func_stack.push(name),
                    _ => ()
                }
            },
            Lexem::Type(var_type) => {
                match vars_inscope.get(&current_name.to_string()) {
                    Some(var) => { 
                        match var_type.as_str() {
                            "file" => {
                                let c_b = VarVal{val_type:VarType::File, value:var.value.clone(), values: Vec::new()};
                                vars_inscope.insert(current_name.to_string(), c_b);
                            },
                            "env" => {
                               // println!("env {}", var.value);
                                let c_b = VarVal{val_type:VarType::Environment, value:var.value.clone(), values: Vec::new()};
                                vars_inscope.insert(current_name.to_string(), c_b);
                            },
                            _ => ()
                        }
                        
                    },
                    _ => ()
                }
            },
            Lexem::Parameter(value) => { // collect all parameters and then process function call
                if state2 == LexState::EndFunction {
                    let name = func_stack.pop();
                    if let Some(name) = name {
                        match name.as_str() {
                            "display" => {
                                println!("{}", *process_template_value(&log, &value, vars_inscope));
                            },
                            "include" => {
                                match vars_inscope.get(&value) {
                                    Some(var) => {
                                      // println!("found {:?}", var);
                                       match var.val_type {
                                            VarType::File => {
                                                let clone_var = var.value.clone();
                                                process(log, clone_var.as_str(), args, vars_inscope)?;
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
                } else {

                    // push param in params vec
                } 
            },
            Lexem::BlockHdr(value) => { 
                // parse header and push in block stack
            },
            _ => ()
        }
        state = state2;
    }
    Ok(())
}