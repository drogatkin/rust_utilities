# comon script part for Rust projects
RUSTC=/home/dmitriy/AndroidStudioProjects/rust/build/x86_64-unknown-linux-gnu/stage2/bin/rustc
executable=./${project}

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

target build:. {

   dependency {
       anynewer(bee.rb,${project})
   }
   {
      display(Compiling ${main} ...)
       exec RUSTC::  (
           -o,
           ${project},
           ${main}.rs
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
        ask(Would you like to run ${project}? [Y|n] , Y)
        assign(answer, ${~~})
        if {
            eq(${answer},Y)
            then {
                exec executable (
                    ~args~
                   )
            }
        }
   }
