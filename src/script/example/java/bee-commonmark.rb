# a script example to build Java project 

project =commonmark
"build_directory" = ./build
#source_directory ="src"
source_directory ="/home/dmitriy/AndroidStudioProjects/commonmark-java/commonmark/src/main/java"
doc_directory=doc
build_file ="${project}.jar"
 mobile= "y"
 domain ="org"
resources ="${domain}.${project}.resources"
manifestf =""
main_class= "${domain}.${project}.Main"

target clean {
    dependency {true}
    exec rm  (
        -r,
        ${build_directory}/${domain},
        ${build_directory}/${build_file}
    )
}

target compile:. {
   dependency {
       or {
             {
                newerthan(${source_directory}/.java,${build_directory}/.class)
                file_filter(~~,package-info.*)
             }
       }
   }
   {
        
       newerthan(${source_directory}/.java,${build_directory}/.class)
       assign(main src,~~)
       file_filter(main src,package-info.*)
       assign(main src,~~)
       
       display(Compiling Java src....)
       exec javac (
         -d,
         ${build_directory},
        -cp,
         ${build_directory},
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
         anynewer(${build_directory}/${domain}/*,${build_directory}/${build_file})
      }
      dependency {
          target(compile)
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