// lex analizer
//use std::io::{BufRead, BufReader};
use std::fs::File;
use std::io::{self, Read};

use log::Log;

const BUF_SIZE: usize = 256;

const MAX_LEX_LEN: usize = 4096;

enum LEXEM {
    Variable(String, String, String), // name:type:range_constraint
    Value(String), 
}

#[derive(PartialEq)]
enum State {
    BEGIN,
    END
}

#[derive(PartialEq)]
enum LexState {
    Begin,
    QuotedStart,
    InLex,
    InQtLex,
    Escape,
    RangeOrTypeOrEnd,
    RangeStart,
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

fn read_lex(reader: &mut Reader) -> LEXEM {
    let mut buffer : [char; MAX_LEX_LEN] = [' '; MAX_LEX_LEN];
    let mut buf_fill: usize = 0;
    let mut c1 = reader.next();
    let mut state = LexState::Begin;
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
                    }
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
                        let lexstr: String = buffer[0..buf_fill].iter().collect();
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
                    LexState::End => break,
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
                        
                        let lexstr: String = buffer[0..buf_fill].iter().collect();
                        
                    },
                    _ => todo!()
                }
            },
            ']' => {

            },
            ':' => {

            },
            '=' => {

            },
            _ => todo!()
        }
        c1 = reader.next();
    }
    LEXEM::Variable(buffer[0..buf_fill].iter().collect(), "".to_string(), "".to_string())
}

pub fn process(log: &Log, file: & str, args: &Vec<String>) -> io::Result<()> {
    let mut all_chars =  match  open(file) {
        Err(e) => return Err(e),
        Ok(r) => r,
    };
    let mut state = State::BEGIN;
    while state != State::END {
        match state {
            State::BEGIN => {
                let mut lex = read_lex(&mut all_chars);
            },
            _ => ()
        }
    }
    Ok(())
}