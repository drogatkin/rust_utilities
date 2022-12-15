# a build script for this project using this project

env =./env.rb:file
project  =rb
RUSTC=/home/dmitriy/AndroidStudioProjects/rust/build/x86_64-unknown-linux-gnu/stage2/bin/rustc
src=main.rs
include(env);
display(Shell ${Shell})
fake rb=${project}-1

target clean {
    dependency {true}
    exec rm  (
        project
    )
}

target version update : . {
   dependency {
         anynewer(./*.rs,${project})
   }
    dependency {
      eq {
        timestamp(ver.rs)
        # none
     }
   }
   
   {
       display(Generating ver.rs)
       now()
       
       write(ver.rs,"// auto generated
pub fn version() -> (&'static str, u32, &'static str) {
      (&\"1.00.01-preview\", 2, & \"",${~~},"\")
      }")  # or !now() inline
   }
}

target build:. {
   dependency {
        target(version update)
   }
   dependency {
       anynewer(bee.rb,${project})
   }
   {
      display(Compiling ${src} ...)
       exec RUSTC::  (
           -o,
           ${project},
           ${src}
       )
     if {
         neq(${~~}, 0)
         then {
            panic("compilation error(s)")
         }
     }
   }
}

target run :.: {
    dependency {
        target(build)
    }
    dependency {true}
    {
        exec fake rb (
        ~args~
       )
   }
}
