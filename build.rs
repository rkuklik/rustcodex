use std::env::var;
use std::fs::create_dir_all;
use std::fs::exists;
use std::fs::read_dir;
use std::fs::read_to_string;
use std::fs::File;
use std::io::BufWriter;
use std::io::Error;
use std::io::Write;
use std::path::MAIN_SEPARATOR_STR;

use clap::builder::PossibleValuesParser;
use clap::Arg;
use clap::CommandFactory;
use clap_complete::generate_to;
use clap_complete::Shell;

const DIR: &str = "completions";
const APP: &str = "rustcodex";

include!("src/cli.rs");

fn main() {
    let path = var("OUT_DIR").expect("Rust expects UTF-8 paths for `env!`")
        + MAIN_SEPARATOR_STR
        + "templates.rs";
    let path = path.as_str();
    let mut target = BufWriter::new(
        File::options()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)
            .expect("unable to open output file"),
    );

    let mut codegen = TemplateGen::new();
    for template in read_dir("templates").expect("unable to read `templates`") {
        let file = template.expect("critical error while reading `templates`");
        let filename = || file.file_name().to_string_lossy().to_string();
        assert!(
            file.file_type().unwrap().is_file(),
            "template `{}` must be a file",
            filename()
        );

        let mut name = file.file_name().into_encoded_bytes();
        let dot = name
            .iter()
            .position(|byte| *byte == b'.')
            .unwrap_or_else(|| panic!("template `{}` must be in format `lang.suffix`", filename()));
        name.truncate(dot);
        let name = String::from_utf8(name).expect("UTF-8 name");
        let template = read_to_string(file.path())
            .unwrap_or_else(|e| panic!("template `{}` reading failed: {e}", filename()));
        codegen.add(template, name);
    }
    codegen
        .generate(&mut target)
        .expect("code generation failed");

    let langs = codegen
        .langs
        .into_iter()
        .map(|lang| lang.name)
        .map(|mut lang| {
            lang.get_mut(0..1)
                .expect("language name can't be empty")
                .make_ascii_lowercase();
            lang
        });
    // HACK: generate completion
    // keep in sync with `src/cli.rs`
    let mut app = Cli::command().arg(
        Arg::new("target")
            .help("Output language")
            .long("target")
            .short('t')
            .env("TARGET")
            .value_parser(PossibleValuesParser::new(langs)),
    );

    if !exists(DIR).unwrap() {
        create_dir_all(DIR).expect("creating completion directory failed");
    }

    for shell in [Shell::Bash, Shell::Zsh, Shell::Fish, Shell::PowerShell] {
        generate_to(shell, &mut app, APP, DIR)
            .unwrap_or_else(|e| panic!("generating completion for {shell} failed: {e}"));
    }

    println!("cargo::rerun-if-changed=templates");
    println!("cargo::rerun-if-changed=src/cli.rs");
    println!("cargo::rerun-if-changed=src/target.rs");
    println!("cargo::rustc-cfg=generated");
    println!("cargo::rustc-env=GENERATED={path}");
}

struct Language {
    name: String,
    template: String,
}

impl Language {
    const S: &str = "__SOURCE__";
    const P: &str = "__PAYLOAD__";

    fn new(template: String, name: String) -> Self {
        const fn second<'a>((_, second): (&str, &'a str)) -> &'a str {
            second
        }
        const fn nocontain(tag: &'static str) -> impl Fn(&str) -> Option<&str> {
            move |next: &str| (!next.contains(tag)).then_some(next)
        }
        let nref = name.as_str();
        let assertion = |tag| move || panic!("template {nref} must contain single {tag} directive");
        // verify template directive correctness
        template
            .split_once(Self::S)
            .map(second)
            .and_then(nocontain(Self::S))
            .unwrap_or_else(assertion(Self::S))
            .split_once(Self::P)
            .map(second)
            .and_then(nocontain(Self::P))
            .unwrap_or_else(assertion(Self::P));
        for byte in nref.bytes() {
            assert!(
                byte.is_ascii_alphabetic(),
                "language name must be only ASCII alphabetic"
            );
        }
        Self { name, template }
    }

    fn components(&self) -> [&str; 5] {
        let source = self
            .template
            .lines()
            .find(|line| line.contains(Self::S))
            .unwrap();
        let (precomment, postcomment) = source.split_once(Self::S).unwrap();
        let (start, mid, end) = self
            .template
            .split_once(source)
            .and_then(|(s, t)| t.split_once(Self::P).map(|(m, e)| (s, m, e)))
            .unwrap_or_else(|| panic!("directives can't be on one line: in {}", self.name));
        [precomment, postcomment, start, mid, end]
    }
}

struct TemplateGen {
    langs: Vec<Language>,
}

impl TemplateGen {
    /// Setup codegen
    const fn new() -> Self {
        Self { langs: Vec::new() }
    }

    /// Add language definition to generator
    fn add(&mut self, template: String, mut name: String) {
        name.get_mut(0..1)
            .expect("language name can't be empty")
            .make_ascii_uppercase();
        let Err(index) = self.langs.binary_search_by(|lang| lang.name.cmp(&name)) else {
            panic!("language {name} has multiple definitions");
        };
        self.langs.insert(index, Language::new(template, name));
    }

    /// Write rust code to `target`
    fn generate<W: Write>(&self, target: &mut W) -> Result<(), Error> {
        macro_rules! s {
            ($($arg:tt)*) => {
                writeln!(target, $($arg)*)?;
            };
        }
        macro_rules! m {
            ($($arg:tt)*) => {
                for lang in &self.langs {
                    s!($($arg)*, lang=lang.name);
                }
            };
        }

        assert!(
            self.langs.is_sorted_by_key(|lang| lang.name.as_str()),
            "languages are sorted",
        );

        // generate fmt routines
        for lang in &self.langs {
            let [pre, post, start, mid, end] = lang.components().map(str::escape_debug);
            let name = lang.name.as_str();

            s!(r#"/// {name} template parameter, to be used in `Template<'_, {name}>`"#);
            s!(r#"#[derive(Debug, Copy, Clone, PartialEq, Eq)]"#);
            s!(r#"pub struct {name};"#);

            s!(r#"impl ::std::fmt::Display for Template<'_, {name}> {{"#);
            s!(r#"    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {{"#);
            s!(r#"        let Data {{ payload, sources }} = self.data;"#);
            s!(r#"        let source = CodeInliner {{"#);
            s!(r#"            files: sources,"#);
            s!(r#"            start: "{pre}","#);
            s!(r#"            end: "{post}","#);
            s!(r#"        }};"#);
            s!(r#"        let payload = Compressor {{ payload }};"#);

            s!(r#"        f.write_str("{start}")?;"#);
            s!(r#"        f.write_fmt(format_args!("{{source}}"))?;"#);
            s!(r#"        f.write_str("{mid}")?;"#);
            s!(r#"        f.write_fmt(format_args!("{{payload}}"))?;"#);
            s!(r#"        f.write_str("{end}")?;"#);

            s!(r#"        Ok(())"#);
            s!(r#"    }}"#);
            s!(r#"}}"#);
        }

        // generate enumeration
        let count = self.langs.len();
        s!(r#"/// Enumeration of all available languages"#);
        s!(r#"#[derive(Debug, Copy, Clone, PartialEq, Eq, clap::ValueEnum)]"#);
        s!(r#"#[non_exhaustive]"#);
        s!(r#"pub enum Language {{"#);
        m!(r#"    {lang},"#);
        s!(r#"}}"#);

        s!(r#"impl Language {{"#);
        s!(r#"    /// Number of included languages"#);
        s!(r#"    pub const COUNT: usize = {count};"#);
        s!(r#"    /// Array of all included languages"#);
        s!(r#"    pub const ALL: [Self; Self::COUNT] = ["#);
        m!(r#"        Self::{lang},"#);
        s!(r#"    ];"#);
        s!(r#"}}"#);

        s!(r#"impl ::std::fmt::Display for Template<'_, Language> {{"#);
        s!(r#"    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {{"#);
        s!(r#"        #[allow(unused_imports)]"#);
        s!(r#"        use ::std::fmt::Display;"#);
        s!(r#"        match self.ctrl {{"#);
        m!(r#"            Language::{lang} => self.transform({lang}).fmt(f),"#);
        s!(r#"        }}"#);
        s!(r#"    }}"#);
        s!(r#"}}"#);

        Ok(())
    }
}
