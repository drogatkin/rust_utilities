pub struct Log {
    pub verbose: bool,
    pub debug: bool,
    pub quiet: bool,
}

impl Log {
    pub fn log(&self, msg: &str) {
        if self.verbose && !self.quiet {
            println!("{}", msg);
        }
    }
    
    pub fn debug(&self, msg: &str) {
        if self.debug && !self.quiet {
            println!("{}", msg);
        }
    }

    pub fn error(&self, msg: &str) {
        if !self.quiet {
             println!("\x1b[0;31mError: {}\x1b[0m", msg);
        }
    }   
}