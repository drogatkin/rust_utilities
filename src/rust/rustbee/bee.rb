# a build script for the project using this project

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

target install {
    dependency {true}
    {
        if {
            neq(${User}, root)
            then {
                display(Please run the script as an administrator)
            }
            else {
                ask(Are you going to install the ${project}? [N/y],n)
                if {
                    or{
                    eq(${~~},y)
                    eq(${~~},Y)
                    }
                    then {
                        exec cp(${project},/usr/local/bin)
                        display(Installed.)
                    }
                }
            }
        }
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
      (&\"1.00.01-preview\", 3, & \"",${~~},"\")
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
        assign(new_str,)
        for word:a brown lazy fox runs over:" "  {
            display(${word} at ${~index~})
            assign(new_str,${word}_${new_str})
            display(${new_str})
        }
        display(${~~})
        ask(Would you like to run ${project} 好的 ❤? [Y|n] , Y)
        assign(answer, ${~~})
        if {
            eq(${answer},Y)
            then {
                exec fake rb (
                    ~args~
                   )
            }
        }
   }
}
