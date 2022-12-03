# a build script for this project using this project

env =./env.rb:file
project  =rb
RUSTC=/home/dmitriy/AndroidStudioProjects/rust/build/x86_64-unknown-linux-gnu/stage2/bin/rustc
src=main.rs
include(env);

target version update : . {
   dependency {
         allnewer(./*.rs,${project})
   }
    dependency {
      eq {
        timestamp(ver.rs)
        eval() # perhaps just omit it
     }
   }
   
   {
       display(Generating ver.rs)
       now=now():fun
       
       write(ver.rs:file,"// auto generated
pub fn version() -> (&'static str, u32, &'static str) {
      (&\"1.00.01\", 1, & \"",${now},"\")")  # or !now() inline
   }
}


target build:. {
   dependency {
        target(version update)
   }
   dependency {
       allnewer(bee-rust.xml,${project})
   }
   {
      display(Compiling ${src} ...)
       exec RUSTC::  (
           -o,
           ${project},
           ${src}
       )
   }
}

target run :.: {
    dependency {
        target(build)
    }
    dependency {true}
    {
        exec project (
        ..
       )
   }
}
