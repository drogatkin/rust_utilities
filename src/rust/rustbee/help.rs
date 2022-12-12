pub fn get_help() -> String {
    let help = r#"
rb [target [target2 [target3] ...]] [options] [-- parameter1 [...parameter2..]]
Options: 
  -help, -h              print this message
  -version               print the version information and exit
  -diagnostics           print information that might be helpful to
                         diagnose or report problems
  -quiet, -q             be extra quiet
  -verbose, -v           be extra verbose
  -debug, -d             print debugging information
  -lib <path>            specifies a path to search for jars and classes
  -logfile <file>        use given file for log
  -l     <file>          ''
  -logger <classname>    the class which is to perform logging
  -listener <classname>  add an instance of class as a project listener
  -noinput               do not allow interactive input
  -buildfile <file>      use given buildfile
  -file    <file>        ''
  -f       <file>        ''
  -keep-going, -k        execute all targets that do not depend
                         on failed target(s)
  -dry-run         do not launch any executable, but show all run parameters
  -r                     execute all targets accordingly dependencies even when not required
  -D<property>=<value>   use a value for a given property name
  -propertyfile <name>   load all properties from file with -D
                         properties taking precedence
  -xpropertyfile <name>  '' - from XML file
  -inputhandler <class>  the class which will handle input requests
  -find [<file>]         (s)earch for buildfile towards the root of 
  -s  [<file>]           the filesystem and use it 
  -grammar <file>        use grammar defined in a file, DTD change can require /Java version only/
  -g                     ''
  -targethelp            print all target names in a build file with descriptions/comments
  -th                    ''
  --                     separator of argumets passed to a built executable 
Examples: bee jar -d
         rb compile -s
         rb clean compile -r
         rb run -- arg1
"#;
   help.to_string()
}