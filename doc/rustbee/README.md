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
The same rule is applied for a value. Although any name is allowed, all names starting with
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
- **display** - display a message specified by a parameter
- **now**, shows the current time and date in ISO 8601
- **write**, writes to a file specified by first parameter, content of the rest parameters
- **neq**,  compares two parameters and returns true if they are not equal
- **eq**, compares two parameters and returns true if they are equal
- **exec**, executes a process on the underline OS, a name of process separated by a blank from *exec*, 
parameters are parameters of the process
- **or**, considers parameters as boolean values and returns true of first true parameter,
otherwise returns false
- **and**, considers parameters as boolean values and returns true if all parameters are true
- **scalar**, if a parameter is an array, then concatenates all elements using a separator specified by a second parameter 
- **filename**, returns a filename of a parameter, no extension
- **ask**, prompts a console using first parameter, and then read a user input, second parameter is used for the default answer, when a user press enter
- **timestamp**, returns a timestamp of a file specified by a parameter
- **newerthan**, compares timestamp of files specified with a pattern path/*ext with timestamp of files specified using path/*ext and
returns an array of file patch which have later date
- **as_url**, returns a download URL of an artifact specified by a parameter

An example of a script for building a Java project, can be found [there](https://github.com/drogatkin/JustDSD/blob/master/bee-java.rb).
