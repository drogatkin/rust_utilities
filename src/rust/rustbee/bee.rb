# a build script for this project using this project

env =./env.rb:file
project  =rb
RUSTC=/home/dmitriy/AndroidStudioProjects/rust/build/x86_64-unknown-linux-gnu/stage2/bin/rustc
src=main.rs
include(env);
display(Shell ${Shell})
fake rb=${project}-1
crates dir=.crates

target clean {
    dependency {true}
    exec rm  (
        project
    )
}

target install {
    dependency {true}
    {
        if {
            neq(${User}, root)
            then {
                display(Please run the script as an administrator)
            }
            else {
                ask(Are you going to instal the ${project}? [N/y],n)
                if {
                    or{
                    eq(${~~},y)
                    eq(${~~},Y)
                    }
                    then {
                        display(Installing...)
                        exec cp(${project},/usr/local/bin)
                    }
                }
            }
        }
    }
    
}

target dependecies {
    dependency {
      eq {
        timestamp(${crates dir}/http-0.2.8.crate)
        # none
      }
    }
    {
        display(Creating a crate dir)
        exec mkdir (crates dir)
        as_url(Http)
        exec wget (
            ${~~},
            -O,
            ${crates dir}/http-0.2.8.crate
        )
    }
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
        target(dependecies)
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
        timestamp(${project})
        display(last build ${~~})
        exec fake rb (
        ~args~
       )
   }
}
