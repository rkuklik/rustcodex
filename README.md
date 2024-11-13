# rustcodex

Smuggle code from disallowed languages into [ReCodEx](https://github.com/recodex)

## WHY?

Because I refuse to learn `python` the intended way. Why solve an exercise
in an hour, when you can spend couple of them designing and testing a hack
to smuggle precompiled code (from a normal language) into ReCodEx?

## Disclaimer

While really cool, are you sure this is the intended way to solve ReCodEx exercises?

## Usage

`rustcodex` is a CLI tool with its interface documented obtained by running
`rustcodex --help`. Following example demonstrates how to inline release build
of this tool (including its source code) into python:

```bash
rustcodex \
    --input target/release/rustcodex \
    --output main.py \
    --target python \
    --source Cargo.toml build.rs src
```

## Templates

To add a new language, add a new UTF-8 encoded file into the `templates`
directory, which will be parsed and inlined into the resulting binary on rebuild.
Filename is significant. It must contain at least one `.`, where the text
preceding the first `.` will be used as the template's name.

Apart from code itself, the template must contain two directives (literal strings).
`__SOURCE__` directive must be placed in a comment and will be used to inline
source code into the final output. `__PAYLOAD__` directive will replaced with
base64 encoded gzip compressed binary.

Failure to comply with these requirements will result in punishment manifesting
in build error.
