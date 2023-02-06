file="test.txt"

target test {
     dependency {true}
     {
           display(Create a new file ${file})
            write(${file},hey new file, line 2)
            writea(${file},   line 3, "
line 4")
            read(${file})
            display(we wrote
              ${~~})
          timestamp(file)
           display(Created on ${~~})
           display(Done in ${~cwd~})

1=1at
2=2in
3in3=3as3\\
con\9\\7

          some_array=[mem2, mem${2}6, carm${3in3}5an]
          array(some_array,new1,",,k${1}as")
          assign(res, ~~)
          display(fun array: ${res})
          testarray=[for1,3four,\
      five-six\
     ,end]
          display(Test array: ${testarray})
          json lib="org.glassfish:javax.json:1.1.4":rep-maven
          as_url(json lib)
          display(Download json lib ${~~})
     }
}