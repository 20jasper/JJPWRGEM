# json-parser

I originally started this project to practice finite state machines, but got back into it when hearing about the internals of some formatters and compilers!

I am heavily inspired by the Rust compiler's error messages. I love the idea that unhelpful errors are considered bugs

I checked out several JSON parsers and formatters, and none provided much context on _why_ a key was missing. Errors ranged from "expected closing on byte 10" to a snapshot of source code for that character, but none were up to my standards

## Notes

I went with annotate snippets over other libraries like codespan reporting since it better supports patches and workflows that don't require files, like reading from stdin

## references

https://rustc-dev-guide.rust-lang.org/diagnostics/error-codes.html
https://github.com/rust-lang/rust/pull/27475
