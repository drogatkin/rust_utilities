# a script example to build Java project 

project =fuzzywuzzy
"build_directory" = ./build
source_directory ="src"
source_directory_extra ="diffutils/src"
doc_directory=doc
build_file ="${project}.jar"
 mobile= "y"
 domain ="me"
resources ="${domain}.${project}.resources"
manifestf =""
main_class= "${domain}.${project}.Main"

target compile:. {
   dependency {
       or {
              newerthan(${source_directory_extra}/.java,${build_directory}/.class)
              newerthan(${source_directory}/.java,${build_directory}/.class)
       }
   }
   {
        display(Compiling Java src ...)
       newerthan(${source_directory_extra}/.java,${build_directory}/.class)
       assign(util src,~~)
       newerthan(${source_directory}/.java,${build_directory}/.class)
       assign(main src,~~)
       exec javac (
         -d,
         ${build_directory},
        -cp,
         ${build_directory},
         util src,
         main src

       )     
      if {
         neq(${~~}, 0)
         then {
            panic("Compilation error(s)")
         }
     }
   }
}

target jar {
     dependency {
         anynewer(${build_directory}/${domain},build_file)
     }
     {    display(Jarring ${build_file} ...)
          exec jar (
            -cf,
            ${build_directory}/${build_file},
            -C,
            ${build_directory},
            ${domain}
          )
     }
}