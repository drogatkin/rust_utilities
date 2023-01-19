file=test.txt

target test {
     dependency {true}
     {
           display(Create a new file)
            write(${file},hey new file, line 2)
            writea(${file},   line 3, "
line 4")
            read(${file})
            display(we wrote
              ${~~})
           display(Done in ${~cwd~})

          some_array=[mem2, mem6, carman]
          array(some_array,new1,",,kas")
          assign(res, ~~)
          display(fun array: ${res})
     }
}