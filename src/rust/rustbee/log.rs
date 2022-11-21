pub struct Log {
    pub verbose: bool,
    pub debug: bool,
}

impl Log {
    pub fn log(&self, msg: &str) {
        if self.verbose {
            println!("{}", msg);
        }
    }
    
    pub fn debug(&self, msg: &str) {
        if self.debug {
            println!("{}", msg);
        }
    }

    pub fn error(&self, msg: &str) {
        println!("Error: {}", msg);
    }   
}