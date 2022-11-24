// lex analizer
//use std::io::{BufRead, BufReader};
use std::fs::File;
use std::io::{self, Read};

use log::Log;

const BUF_SIZE: usize = 256;

const MAX_LEX_LEN: usize = 4096;

#[derive(PartialEq, Debug)]
enum Lexem {
    Variable(String, String, String), // name:type:range_constraint
    Value(String), 
    Comment(String),
    Type(String),
    Range(usize, usize),
    Function(String),
    Parameter(String),
    EOF
}

#[derive(PartialEq, Debug, Copy, Clone)]
enum LexState {
    Begin,
    QuotedStart,
    InLex,
    InQtLex,
    Escape,
    RangeOrTypeOrEnd,
    RangeStart,
    Comment,
    InType,
    StartValue,
    InValue,
    EndValue,
    RangeEnd,
    InParam,
    StartParam,
    EndFunction,
    End
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
    let mut c1 = reader.next();
    //let mut state = LexState::Begin; //*state1;
    //let mut state = state1;
    while let Some(c) = c1 {
        match c {
            '"' => {
                match state {
                    LexState::Begin => state = LexState::QuotedStart,
                    LexState::InLex => {
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                    },
                    LexState::InQtLex => {
                        let lexstr: String = buffer[0..buf_fill].iter().collect();
                        state = LexState::RangeOrTypeOrEnd;
                    },
                    LexState::Escape => {
                        state = LexState::InQtLex ;
                        buffer[buf_fill] = c;
                        buf_fill += 1;
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
                    LexState::Begin => (),
                    LexState::QuotedStart => {
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                        state = LexState::InQtLex;
                    },
                    LexState::InLex => {
                        state = LexState::RangeOrTypeOrEnd;
                        //let lexstr: String = buffer[0..buf_fill].iter().collect();
                        return (Lexem::Variable(buffer[0..buf_fill].iter().collect(), "".to_string(), "".to_string()), state);
                    },
                    LexState::Comment => {
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                    },
                    LexState::RangeOrTypeOrEnd => {
                        
                    },
                    LexState::StartValue => {

                    },
                    LexState::InValue => {
                        state = LexState::EndValue;
                        return (Lexem::Value(buffer[0..buf_fill].iter().collect()), state);
                    },
                    LexState::StartParam => {

                    },
                    _ => todo!()
                }

            },
            '\\' => {
                match state {
                    LexState::InQtLex | LexState::QuotedStart => state = LexState::Escape,
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
                    LexState::Begin => {
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
                    LexState::Begin => {
                    },
                    LexState::InValue => {
                        state = LexState::Begin;
                        return (Lexem::Value(buffer[0..buf_fill].iter().collect()), state);
                    },
                    _ => todo!()
                }
            },
            '[' => {
                match state {
                    LexState::RangeOrTypeOrEnd => state = LexState::RangeStart,
                    LexState::InQtLex => {
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                    },
                    LexState::InLex => {
                        state = LexState::RangeStart;
                        
                        //let lexstr: String = buffer[0..buf_fill].iter().collect();
                        return (Lexem::Variable(buffer[0..buf_fill].iter().collect(), "".to_string(), "".to_string()), state);
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
            ':' => {
                match state {
                    LexState::RangeOrTypeOrEnd => {
                        state = LexState::InType;

                    },
                    _ => todo!()
                }
            },
            '=' => {
                match state {
                    LexState::InLex => {
                        
                        state = LexState::StartValue; 
                        return (Lexem::Variable(buffer[0..buf_fill].iter().collect(), "".to_string(), "".to_string()), state);
                    },
                    LexState::RangeOrTypeOrEnd => {
                        state = LexState::StartValue; 
                        //return (Lexem::Variable(buffer[0..buf_fill].iter().collect(), "".to_string(), "".to_string()), state);
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
                    LexState::RangeOrTypeOrEnd => {
                        state = LexState::StartParam; 
                        return (Lexem::Function(buffer[0..buf_fill].iter().collect()), state);
                    },
                    _ => todo!()
                }
            },
            ')' => {
                match state {
                    LexState::InParam => {
                        
                        state = LexState::EndFunction; 
                        return (Lexem::Parameter(buffer[0..buf_fill].iter().collect()), state);
                    },
                    LexState::StartParam => {
                        state = LexState::EndFunction; 
                        return (Lexem::Parameter(buffer[0..buf_fill].iter().collect()), state);
                    },
                    _ => todo!()
                }
            },
            '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {

            },
            '.' => {
                match state {
                    LexState::InValue => {
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                    },
                    LexState::StartValue => {
                        state = LexState::InValue;
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                    },
                    _ => todo!()
                }

            },
            _ => {
                match state {
                    LexState::InQtLex => {
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                    },
                    LexState::InLex => {
                        buffer[buf_fill] = c;
                        buf_fill += 1;
                    },
                    LexState::Begin => {
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
        LexState::InLex => {
            
        },
        LexState::Begin => {
            
        },
        _ => todo!()
    }
    (Lexem::Variable(buffer[0..buf_fill].iter().collect(), "".to_string(), "".to_string()), state)
}

pub fn process(log: &Log, file: & str, args: &Vec<String>) -> io::Result<()> {
    let mut all_chars =  match  open(file) {
        Err(e) => return Err(e),
        Ok(r) => r,
    };
    let mut state = LexState::Begin;
    while state != LexState::End {
        let (mut lex, mut state2) = read_lex(log, &mut all_chars, state);
        log.debug(&format!("Lex: {:?}, state: {:?}", lex, state2));
        match lex {
            Lexem::EOF => {
                state = LexState::End;
            },
            Lexem::Variable(name, type_it, range_it) => {
                
            },
            Lexem::Value(value) => {
                state = LexState::End;
            },
            _ => ()
        }
        state = state2;
    }
    Ok(())
}