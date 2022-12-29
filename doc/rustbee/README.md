# RustBee

## Purpose
RustBee is a light version of 7Bee build tool written in Rust. RB has several
advantages over 7Bee as:
1. more concise and clear syntax of scripts
2. footprint under 1Mb
3. more friendly to non Java builds
4. can work on systems where Java isn't supported

## Syntax highlights
RB build script as 7Bee defines at least one build target. Several build
targets can be dependent, It is possible to define variables in a form:

    name=value

Name and value can be anything, but if a name includes spaces or '=' then
name has to be quoted. If the name should include quote, then use \ for escaping it.
The same rule is applied for a value. Although any name is possible, all name starting with
*~* and ending with *~* are reserved.

Target looks like :
    
     target name {
        dependency {...}
         ...
        dependency {...}
        {
            target function or operator
             ....
        }
     }

Currently *if* and *for* operators are supported.

Function can be the following:
- **assign**, first parameter is a *name* of variable, the second is a value
- *panic*, a parameter specifies a panic message


