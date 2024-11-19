# rustcodex

Smuggle code from disallowed languages into [ReCodEx](https://github.com/recodex)

## WHY?

Because I refuse to learn `python` the intended way. Why solve an exercise
in an hour, when you can spend a couple of them designing and testing a hack
to smuggle precompiled code (from a normal language) into ReCodEx?

## Disclaimer

While really cool, are you sure this is the intended way to solve ReCodEx exercises?

## Functionality

Smuggles executable into another language, which can then start it. It supports
annotating the output with comments containing source code inline. Do note that
the executable can be anything, which can be started via `exec` (on Linux, this
can even be a script with a shebang, for instance). Every input must have UTF-8
encoding, except for the input binary. All encountered IO errors are bubbled up
to the user for examination. The behavior of produced output shall be identical
to that of the input, except for runtime initialization (speed) overhead.

## Usage

`rustcodex` is a CLI tool with its interface documented inline by running
`rustcodex --help`. Following example demonstrates how to inline release build
of this tool (including its source code) into python:

```bash
rustcodex \
    --input target/release/rustcodex \
    --output main.py \
    --target python \
    --source Cargo.toml build.rs src
```

## Obtaining the binary

### Requirements

- [Rust](https://www.rust-lang.org/) v1.82.0 or greater
- [Computer with tier 1 support with host tools](https://doc.rust-lang.org/nightly/rustc/platform-support.html)

### Build

Run `cargo build --profile release` to obtain optimized binary or
`cargo build --profile small` to get trimmed down binary. Resulting program
will be located in `target/_profile_/rustcodex`.

## Architecture

### Templates

Templates are UTF-8 encoded files present in the `templates` directory, which
will are parsed and inlined into the resulting binary on rebuild via `build.rs`.
Filename is significant. It must contain at least one `.`, where the text
preceding the first `.` will be used as the template's name.

Apart from code itself, the template must contain two directives (literal strings).
`__SOURCE__` directive must be placed in a comment and will be used to inline
source code into the final output. `__PAYLOAD__` directive will be replaced with
base64 encoded gzip compressed binary. `__SOURCE__` must come before `__PAYLOAD__`.

Additionally, the template should `exec` uncompressed payload, passing through
`argv` with `argv[0]` set to `binary`.

Failure to comply with directive requirements will result in punishment manifesting
as build error.

### Internals

Rust code is annotated inline, with `cargo doc --open` opening public documentation.
During compilation, `build.rs` generates shell completions and converts templates
into Rust code (which can be viewed from generated documentation via source button).
However, given that the source code is self-explanatory, canonical info about behavior
is found there. Good entrypoint is `main.rs`, which references basically all internals
of `rustcodex`.

