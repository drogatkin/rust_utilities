// lex analizer
//use std::io::{BufRead, BufReader};
use std::fs::File;
use std::io::{self, Read};

const BUF_SIZE: usize = 256;

enum LEXEM {
    Variable,
    Value, 
}
struct Lex {
    kind_of : LEXEM
}

#[derive(PartialEq)]
enum State {
    BEGIN,
    END
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

fn read_lex(reader: & Reader) -> LEXEM {
    LEXEM::Value
}

pub fn process(file: & str) -> io::Result<()> {
    let mut all_chars =  match  open(file) {
        Err(e) => return Err(e),
        Ok(r) => r,
    };
    let mut state = State::BEGIN;
    while state != State::END {
        match state {
            State::BEGIN => {
                let lex = read_lex(&all_chars);
            },
            _ => ()
        }
    }
    Ok(())
}