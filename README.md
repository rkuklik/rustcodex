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
to the user for examination. The output shall be source code for user selected
programming language. The observable behavior of produced output, when compiled
or interpreted, shall be identical to that of the input, except for runtime
speed and memory overhead. There are also no guarantees on other resource usage,
such as threads and files.

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

### Known limitations

1. Only latest version of each language (as of today, 2024-11-25), are supported.
   Other may work though.
2. Produced source code is not guaranteed to compile if produced payload is larger
   than implementation defined literal size limit. The same goes with file size due
   to inlined comments. For example, `JVM` languages set literal limit at $2^{16}-1$
   bytes.
3. Produced source code may require additional configuration to compile.
   For instance, `C#` requires valid `csproj` with `.NET` framework.
4. Inlined executable will work only if its runtime dependencies are present on
   the machine and `exec` style functions are able to start it.

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
are parsed and inlined into the resulting binary on rebuild via `build.rs`.
Filename is significant. It must contain at least one `.`, where the text
preceding the first `.` will be used as the template's name.

Apart from code itself, the template must contain two directives (literal strings).
`__SOURCE__` directive must be placed in a comment and will be used to inline
source code into the final output. `__PAYLOAD__` directive will be replaced with
base64 encoded gzip compressed binary. `__SOURCE__` must come before `__PAYLOAD__`.

If possible, the template should `exec` uncompressed payload, passing through
`argv`. Failure to comply with directive requirements will result in punishment
manifesting as build error.

### Template compilation

Templates are parsed and inlined into the source code at build-time. While
it would be fairly trivial to load them at runtime, there is intentionally
no support for that.

This tool is intended to be without runtime dependencies, apart from those
required by target triple. Loading templates at runtime would require the
user to install them into a suitable directory. They would also pose a
security concern, as any user with write permissions could (maliciously)
overwrite them and thanks to that execute arbitrary code. Only suitable
location would be read-only system directory, complicating install process
and requiring administrator privileges.

While parsing the templates at build time slightly complicates the codebase,
prolongs build time and increases binary size, it was deemed to be worth the
hassle. Moreover, as a nice side effect of this decision, the **only** way to
define new or modify old templates is by editing files in `templates` directory.
This couples them tightly to the codebase, improves integration and prevents
breaking changes to their syntax or semantics from being observed by the user.
If they change in backward incompatible ways, the tool will not build until
all errors are fixed. This will be the case if/when custom payload pipelines
are implemented, such as splitting payload to compilable chunks for `JVM`
languages or multiple outputs. Perhaps in second semester?

### Internals

Rust code is annotated inline, with `cargo doc --open` opening public documentation.
During compilation, `build.rs` generates shell completions and converts templates
into Rust code (which can be viewed from generated documentation via source button).
However, given that the source code is self-explanatory, canonical info about behavior
is found there. Good entrypoint is `main.rs`, which references basically all internals
of `rustcodex`.
