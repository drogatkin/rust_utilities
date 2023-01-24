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

1=1at
2=2in
3in3=3as3

          some_array=[mem2, mem${2}6, carm${3in3}5an]
          array(some_array,new1,",,k${1}as")
          assign(res, ~~)
          display(fun array: ${res})
     }
}