# Clidocs

Generating CLI docs

better name
- cleedus 


functionality
- generate yaml/json/toml representation
- generate site from source 
- generate site from yaml/json/toml representation
- generate snippets
- allow user provided sections - We shouldnt dictate sections
  - do we need to allow for specific ordering
  - what format should we allow? 
    - Markdown
    - MDX
    - bring your own template/engine



https://github.com/spf13/cobra/blob/8003b74a10ef0d0d84fe3c408d3939d86fdeb210/doc/yaml_docs.go#L37

common structure
- https://cran.r-project.org/web/packages/optigrab/vignettes/technical-specification.html
- https://clig.dev/
- https://developers.google.com/style/code-syntax
- https://gist.github.com/pksunkara/1485856
- https://pubs.opengroup.org/onlinepubs/009695399/basedefs/xbd_chap12.html
- http://docopt.org/

https://docs.onap.org/projects/onap-cli/en/latest/open_cli_schema_version_1_0.html
https://oclif.io/

### Docopt

argument: `<argument> ARGUMENT` are interpreted as positional arguments

-o --option
`-` short (one-letter) 
`--` long options

short can be stacked
Short options can have arguments specified after optional space:
-f FILE is equivalent to -fFILE.

Long options can have arguments specified after space or equal "=" sign:
--input=ARG is equivalent to --input ARG.

command
All other words that do not follow the above conventions of --options or <arguments> are interpreted as (sub)commands.

optional elements
Elements (options, arguments, commands) enclosed with square brackets "[ ]" are marked to be optional.

Required elements
All elements are required by default, if not included in brackets "[ ]". However, sometimes it is necessary to mark elements as required explicitly with parens "( )"

mutually exclusive elements `element|another`
Mutually-exclusive elements can be separated with a pipe "|"


Repeated elements `element...`
Use ellipsis "..." to specify that the argument (or group of arguments) to the left could be repeated one or more times:

You can flexibly specify the number of arguments that are required. Here are 3 (redundant) ways of requiring zero or more arguments:

```
Usage: my_program [<file>...]
       my_program [<file>]...
       my_program [<file> [<file> ...]]
```

One or more arguments: `Usage: my_program <file>...`
Two or more arguments (and so on): `Usage: my_program <file> <file>...`

`[options]`
"[options]" is a shortcut that allows to avoid listing all options (from list of options with descriptions) in a pattern. For example:

``` 
Usage: my_program [options] <path>

--all             List everything.
--long            Long output.
--human-readable  Display in human-readable format.
```
is equivalent to:

``` 
Usage: my_program [--all --long --human-readable] <path>

--all             List everything.
--long            Long output.
--human-readable  Display in human-readable format.
```

`[--]`
A double dash "--", when not part of an option, is often used as a convention to separate options and positional arguments, in order to handle cases when, e.g., file names could be mistaken for options. In order to support this convention, add "[--]" into your patterns before positional arguments.

``` 
Usage: my_program [options] [--] <file>...
```

Apart from this special meaning, "--" is just a normal command, so you can apply any previously-described operations, for example, make it required (by dropping brackets "[ ]")

`[-]`
A single dash "-", when not part of an option, is often used by convention to signify that a program should process stdin, as opposed to a file. If you want to follow this convention add "[-]" to your pattern. "-" by itself is just a normal command, which you can use with any meaning.


Option descriptions
Option descriptions consist of a list of options that you put below your usage patterns. It is optional to specify them if there is no ambiguity in usage patterns (described in the --option section above).

An option's description allows to specify:

that some short and long options are synonymous,
that an option has an argument,
and the default value for an option's argument.
The rules are as follows:

Every line that starts with "-" or "--" (not counting spaces) is treated as an option description, e.g.:

``` 
Options:
  --verbose   # GOOD
  -o FILE     # GOOD
Other: --bad  # BAD, line does not start with dash "-"
```

To specify that an option has an argument, put a word describing that argument after a space (or equals "=" sign) as shown below. Follow either <angular-brackets> or UPPER-CASE convention for options' arguments. You can use a comma if you want to separate options. In the example below, both lines are valid, however it is recommended to stick to a single style.

``` 
-o FILE --output=FILE       # without comma, with "=" sign
-i <file>, --input <file>   # with comma, without "=" sign
```

Use two spaces to separate options with their informal description.

``` 
--verbose MORE text.    # BAD, will be treated as if verbose
                        # option had an argument MORE, so use
                        # 2 spaces instead
-q        Quit.         # GOOD
-o FILE   Output file.  # GOOD
--stdout  Use stdout.   # GOOD, 2 spaces
```

If you want to set a default value for an option with an argument, put it into the option's description, in the form [default: <the-default-value>].

``` 
--coefficient=K  The K coefficient [default: 2.95]
--output=FILE    Output file [default: test.txt]
--directory=DIR  Some directory [default: ./]
```


Outputs see 
- stripe

Supported frameworks
- rust clap
- python argparse, click
- node oclif
- go cobra
- .net command-line-api
- python fire

example clis
- Git: github, gitlab
- CSPs: azure, aws, heroku
- K8s: minikube, kubectl
- Sites creation: npm, vue, gatsby
- stripe

Further reading:
  clig.dev — Command Line Interface Guidelines
  primer — Design guidelines for GitHub’s command line tool
  dev.to — 14 great tips to make amazing CLI applications
  Liran Tal’s research — Node.js CLI Apps Best Practices
  opensource.com — 3 steps to create an awesome UX in a CLI application
  zapier.com — Best Practices Building a CLI Tool for Your Service
  atlassian.com — 10 design principles for delightful CLIs
  uxdesign.cc — User experience, CLIs, and breaking the world
  gnu.org — POSIX guidelines
  click — Python’s click documentation


## Python Click

Call python program via Py03. See https://github.com/RiveryIO/md-click/blob/main/md_click/main.py

## Golang Click

doc.GenYamlTree(cmd, "./docs")
