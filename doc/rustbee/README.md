# RustBee

## Purpose
RustBee is a lightweight version of 7Bee build tool written in Rust. RB has several
advantages over 7Bee as:
1. more concise and clear syntax of scripts
2. footprint is under 1Mb
3. more friendly to non Java builds
4. can work on systems where Java isn't supported

## Syntax highlights
RB build script defines at least one build target. Several build
targets can be dependent. A script variable can be defined in a form:

    name=value

Name and value can be anything, but if a name includes spaces or symbols like `=, ;, {,( ` then
name has to be quoted. If the name should include quote, then use \ for escaping it.
The same rule is applied for a value. If one of the following characters `:, ;, {, [` is included in 
a value  then the value has to be quoted, for example:

    json lib="org.glassfish:javax.json:1.1.4":rep-maven
Although any name is allowed, all names starting with
*~* and ending with *~* are reserved.

- A name as ~~ is reserved for a result previous operation.
- A separator for parts of a path is \~separator\~
- A paths separator is \~path_separator\~
- An array of a command line arguments is \~args\~
- A string representing the current OS is \~os\~
- A current working directory \~cwd\~
- An index of the current loop iteration \~index\~

You can break a line adding \ at the end.

A target is defined as :
    
     target name:[work dir]:[description] {
        dependency {...}
         ...
        dependency {...}
        {
            target function or operator
             ....
        }
     }

A dependency can be:

- **anynewer**, function with two parameters, path to a file, second file has to be newer
- **eq** block, specifies that all arguments must be equal
- **or** block, one of the arguments has to be true
- **target**, for dependency on the target
- **true**, for unconditional execution of the target

The body of a target contains a sequence of operators and functions. 
Currently *if* and *for*  operators are supported. More details on syntax of the constructions:

### if

     if {
       a condition function or a condition block
       then {
       }
      [ else {
      } ]

### for

    for var_name:array[:array elements separator if array defined as a scalar value] {
      # loop actions
    }

A function can be the following:
- **and**, considers parameters as boolean values and returns true if all parameters are true
- **array**, converts a list of parameters in an array, which can be consumed as a function result
- **as_url**, returns a download URL of an artifact specified by a parameter
- **ask**, prompts a console using first parameter, and then read a user input, second parameter is used for the default answer, when a user press the enter
- **assign**, first parameter is a *name* of variable, the second is a value, the function returns a previous value under the name, if any
- **display** - display a message specified by a parameter
- **element**, set/get an ellement of an array, first parameter specifies an array, second an index, and optional 3rd, when a value has to be set
- **eq**, compares two parameters and returns true if they are equal
- **exec**, executes a process on the underline OS, a name of process separated by a blank from *exec*, 
parameters are parameters of the process
- **filename**, returns a filename of a parameter, no extension
- **file_filter**, shrink an array specified my first parameters by filter values specified by extra parameters
- **newerthan**, compares timestamp of files specified with a pattern path/.ext with timestamp of files specified using path/.ext and
returns an array of files which have later date
- **neq**,  compares two parameters and returns true if they are not equal
- **now**, shows the current time and date in ISO 8601
- **or**, considers parameters as boolean values and returns true of first true parameter,
otherwise returns false
- *panic*, a parameter specifies a panic message
- **read**, reads a file content specified by a parameter
- **scalar**, if a parameter is an array, then concatenates all elements using a separator specified by a second parameter 
- **timestamp**, returns a timestamp of a file specified by a parameter
- **write**, writes to a file specified by first parameter, content of the rest parameters
- **writea**, writes to a file specified by first parameter, content of the rest parameters. It doesn't create a new file if it already exists,
just append content

A result of a function or a block is stored in a temporary variable ~~ and can be consumed in the next operation.

### String interpolation
It allows to extend any value by processing template variables  like:

       ${name}

The name is a name of some variable. Since a substituted value has to be interpolated as well,
the process is recursive. It doesn't do check for looping, and you need to verify if it happens.

### name or value?
Rustbee resolves this ambiguity in the following manner. First it considers the value as a name and is looking for it. If the variable with such name wasn't found, then the value is considered as a literal value. 
A string interpolation is applied at the end of any variant. 

## Examples

An example of a script for building a Java project, can be found [there](https://github.com/drogatkin/JustDSD/blob/master/bee-java.rb).
