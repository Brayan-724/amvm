<h1 align="center">
    Apika's My Virtual Machine (AMVM)
</h1>

AMVM aims to build a virtual machine that runs a custom bytecode. 
This project is vary pairy with [Brayan-724/js-ast](https://github.com/Brayan-724/js-ast), it's a javascript tokenizer, parser and runtime.
The goal is to convert javascript to this bytecode and make it more performant, and use this virtual machine to future project that needs a runtime.

## Goals
1. Build a virtual machine.
    1. Handle successfully bytecode.
    2. Implement primitives (`String`, some numbers `u8`..).
    3. Implement basic operators (`+ - * /`).
    4. Implement basic statements (`if`, `for`, `while`).
2. Convert javascript to bytecode.

### Use it
Working on!

### Collaborate
This project is very young, so it will be very things to change. If you want to collaborate (I appreciate it) you need to know the followings:

This is divided by two: [`lib.rs`] and [`main.rs`].

There're more files that are not for [`main.rs`], but are for [`lib.rs`].

#### [`lib.rs`]
Here's all about internals for parse, generate and run the bytecode, this will be published as `amvm-core` for share it around projects or have a close touch to the engine.

#### [`main.rs`]
This is where the final-user cli is found, so this pretends to have _IO_ operations. Published as `amvm` to run the bytecode output files.

### Bytecode
```
AMVM_HEADER       = "\x08\x48\x30" // Arbitrary value for sign (0B4B30)
COMMAND_SEPARATOR = '\0'

CMD_DCLR_VAR	  = '\x01'
CMD_ASGN_VAR	  = '\x0D'
CMD_PUTS	      = '\x0E'
CMD_EVAL	      = '\x02'

VAR_CONST	      = '\x0B'
VAR_LET	          = '\x0C'

EXPR_VALUE	      = '\x03'
EXPR_VAR	      = '\x0A'
EXPR_ADD	      = '\x09'

VALUE_UNDEFINED	  = '\x04'
VALUE_STRING	  = '\x05'
VALUE_U8	      = '\x06'
VALUE_I16	      = '\x07'
VALUE_F32	      = '\x08'
```

#### (CMD_DCLR_VAR) Command Declaration Variable
Declare a variable to the current context.

``

## Useful links:
- [Brayan-724/js-ast](https://github.com/Brayan-724/js-ast)

[`lib.rs`]: ./src/lib.rs
[`main.rs`]: ./src/main.rs
