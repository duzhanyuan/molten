 
Molten
======

Molten is a programming language which borrows from the ML family of languages,
as well as from Rust and Python.  The compiler is written in Rust and uses
LLVM to generate IR which can be compiled to machine code.

I originally started this project in order to learn Rust.  It is intended to be
a high level language with a full object system that facilitates both functional
and object-oriented programming.  Some syntax elements have been changed from
typical ML languages to follow conventions found in more common languages, such
as C++, Rust, and Python (eg. parenthesis-delimited blocks, conventional class
definitions, generics/type parameters with angle brackets, etc)


Installing
----------

You will need `rustc` and `cargo` installed.  It's recommended that you use
`rustup` to install these.  I've most recently tested it with rustc version 1.28.
You will also need LLVM 5.0 installed.

On Debian/Ubuntu, run:
`sudo apt-get install llvm-5.0 llvm-5.0-runtime llvm-5.0-dev`

You may need to run the following before the rust llvm package will compile:
```
sudo ln -s /usr/bin/llvm-config-5.0 /usr/bin/llvm-config
```

On macOS, run:
`brew install llvm@5`

You may need to add /usr/local/opt/llvm@5/bin to your path

Running
-------

The `molten` script helps with compiling and linking IR files.  To run an example:

```
./molten run examples/fac.ml
```

This will run cargo to build the compiler if needed, then compile the fac.ml
file, as well as the libcore.ml library, link them together, and then run the
output using `lli-5.0`

The `*.ll` files contain IR code for a single file.  The `*.dec` files contain
declarations for use when importing from another file.  The `*.bc` files
contain LLVM Bitcode, which can be executed using `lli-5.0` or compiled using
`llc-5.0`


Example
-------

```
fn fac(x) {
    if x < 1 then
	1
    else
	x * fac(x - 1)
}

println(str(fac(10)))
```

### Types
```
Nil
Bool
Byte
Int
Real
String
() -> Int           // function type
'a                  // type variable
List<Int>           // list of integers
(Int, Real)         // tuple
{ a: Int, b: Real } // record
```

### Declarations
```
let foo = 0
let bar: String = "Hey"
```

### Functions
```
fn foo(x, y) => x + y		    // named inline function

fn foo(x, y) { x + y }		    // named block function

let foo = fn x, y => x + y	    // anonymous function

fn foo(x: Int, y) -> Int { x + y }  // with optional type annotations

```

### Invoking Functions
Unlike in ML, the brackets of a function call are not elidable.  This is a
design decision to improve readability of the code and to make the parser
simpler and more predictable.
```
foo(1, 2)
```

### Classes
```
class Foo {
    let name: String

    fn new(self, name) {
        self.name = name
    }

    fn get(self) => self.name

    fn static(x) => x * 2
}

class Bar extends Foo {
    fn get(self, title) => self.name + " " + title
}

let bar = new Bar("Mischief")
bar.get("The Cat")              // returns "Mischief The Cat"
Foo::static(5)
```

### Flow Control
The return value of an if statement is the result of the expression of the
clause that is evaluated.  The types of both clauses must match.  The `else`
clause can be left out as long as the true clause evaluates to Nil.
```
if x == 5 then
    "It's five"
else
    "It's not five"

match x {
    1 => "It's one"
    5 => "It's five"
    _ => "It's not five"
}
```

### Loops
```
while is_true
    println("looping")

for i in [ 1, 2, 3 ]
    println("counting " + i)
```

### Blocks
A block is a collection of statements which return the result of the last
expression in the block.  They can be used in place of a single expression.
They do not create their own local scope, at least at the moment, so variables
defined inside blocks will appear in the parent scope (usually the function
the block is in).
```
let is_zero = if self.x <= 0 then {
    self.x = 0
    true
} else {
    false
}
```

### Lists
```
let list1 = [ 1, 3, 6 ]
for x in list1
    println(str(x))

let list2 = new List<String>();
list2.insert(0, "Hello")
println(list2[0])
```

### And / Or
The keyword operators `and` and `or` have side-effects and will not execute
the second expression if the result can be determined from the first
expression.  The resulting value is the last expression that was executed.
Operands are not limited to Bool values, although that may change in future.

### Import
```
import libcore
```

### External Functions
A function can be declared without being implemented, and functions can
also be defined with an ABI specifier so that they are accessible to
other languages.  Only C support is currently implemented. A C function
cannot be a closure.
```
decl foo : (Int) -> Int         // external molten function
decl bar : (Int) -> Int / C     // external C function

fn baz(i: Int) / C {
    // molten function that can be called from C
}
```


Yet To Complete
---------------

- Closure conversion is not yet fully implemented.  I haven't decided yet
  whether to make all functions be closures, or to add a distinct type for
  closures vs non-closure functions

- Exceptions haven't been implemented yet.  This somewhat relates to the
  above issue of making all functions support throwing exceptions, which
  means a need to declare C/C++ functions that do not allow exceptions.

- Class members can be declared with an optional initial value, but the
  initial value is ignored.  Instead you must initialize the value in the
  class constructor.  Ideally I'd like to get it working, but given that
  the AST cannot be duplicated without side effects, and it needs to work
  with inherited members too, I haven't gotten it working yet.  The simple
  solution is to just disallow initial values in class members

- Garbage collection is not yet implemented

- Dynamic Dispatch/vtables works now!


I'd be happy to hear of any additional features ideas or suggestions, if
you'd like to leave them under "Issues" on github.


