use std::env;
use std::fmt;
use std::fmt::Write;
use std::fs::create_dir_all;
use std::fs::exists;
use std::fs::read_dir;
use std::fs::read_to_string;
use std::fs::File;
use std::io::Error;

use clap::builder::PossibleValuesParser;
use clap::Arg;
use clap::CommandFactory;
use clap_complete::generate_to;
use clap_complete::Shell;

const DIR: &str = "completions";
const APP: &str = "rustcodex";

include!("src/cli.rs");

fn main() -> Result<(), Error> {
    // hold onto each language discovered for completion generation
    let mut langs = Vec::<String>::new();
    let mut codegen = TemplateGen::new();
    // generate code for each template in `templates` directory
    for template in read_dir("templates")? {
        let file = template?;
        assert!(file.file_type()?.is_file());
        let name = file.file_name();
        let name = name.to_str().expect("UTF-8 name");

        let language = name
            .split_once('.')
            .expect("template name must be in format `language.suffix`")
            .0;
        let template = read_to_string(file.path())?;

        codegen.add(template.as_str(), language);
        langs.push(language.into())
    }
    codegen.generate()?;

    if !exists(DIR)? {
        create_dir_all(DIR)?;
    }

    // HACK: generate completion
    let mut app = Cli::command();
    app = app.arg(
        Arg::new("target")
            .help("Language to target as host")
            .long("target")
            .short('t')
            .env("TARGET")
            .value_parser(PossibleValuesParser::new(langs)),
    );

    for shell in [Shell::Bash, Shell::Zsh, Shell::Fish] {
        generate_to(shell, &mut app, APP, DIR)?;
    }

    // add cargo directives
    println!("cargo::rerun-if-changed=templates");
    println!("cargo::rerun-if-changed=src/cli.rs");
    println!("cargo::rerun-if-changed=src/target.rs");
    println!("cargo::rustc-cfg=nonrecursive");

    Ok(())
}

struct Language {
    name: String,
}

impl Language {
    fn new(name: &str) -> Self {
        let msg = "language name must be only Ascii alphabetic";
        Self {
            name: name
                .chars()
                .inspect(|char| assert!(char.is_ascii_alphabetic(), "{msg}"))
                .enumerate()
                .map(|(index, char)| match index {
                    0 => char.to_ascii_uppercase(),
                    _ => char.to_ascii_lowercase(),
                })
                .collect(),
        }
    }
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.name.fmt(f)
    }
}

struct TemplateGen {
    // could be written directly to a file, but templates should be small
    buf: String,
    langs: Vec<Language>,
}

impl TemplateGen {
    const S: &str = "__SOURCE__";
    const P: &str = "__PAYLOAD__";

    /// Setup codegen
    fn new() -> Self {
        Self {
            buf: String::from("// Generated from `build.rs`. DO NOT EDIT!\n"),
            langs: Vec::new(),
        }
    }

    /// Add language definition to generator
    fn add(&mut self, template: &str, name: &str) {
        let assertion = |tag| move || panic!("template must contain single {tag} directive");
        fn second<'a>((_, second): (&str, &'a str)) -> &'a str {
            second
        }
        fn nocontain(tag: &'static str) -> impl Fn(&str) -> Option<&str> {
            move |next: &str| (!next.contains(tag)).then_some(next)
        }
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

        let source = template
            .lines()
            .find(|line| line.contains(Self::S))
            .unwrap();
        let (precomment, postcomment) = source.split_once(Self::S).unwrap();
        let (start, mid, end) = template
            .split_once(source)
            .and_then(|(start, tmp)| tmp.split_once(Self::P).map(|(mid, end)| (start, mid, end)))
            .unwrap();

        let [precomment, postcomment, start, mid, end] =
            [precomment, postcomment, start, mid, end].map(str::escape_debug);

        let language = Language::new(name);
        let name = &language.name;

        write!(
            self.buf,
            r#"
impl Display for Template<'_, {name}> {{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {{
        let Data {{ payload, sources }} = self.data;

        let source = CodeInliner {{
            files: sources,
            start: "{precomment}",
            end: "{postcomment}",
        }};

        let payload = Compressor {{ payload }};

        f.write_str("{start}")?;
        write!(f, "{{source}}")?;                
        f.write_str("{mid}")?;
        write!(f, "{{payload}}")?;                
        f.write_str("{end}")?;

        Ok(())
    }}
}}
"#
        )
        .unwrap();

        self.langs.push(language);
    }

    fn generate(mut self) -> Result<(), Error> {
        macro_rules! s {
            ($($arg:tt)*) => {
                write!(self.buf, $($arg)*).unwrap()
            };
        }
        macro_rules! m {
            ($($arg:tt)*) => {
                for lang in &self.langs {
                    write!(self.buf, $($arg)*, lang=lang).unwrap()
                }
            };
        }

        // NOTE: This is ugly, but works.
        m!("#[derive(Debug, Copy, Clone, PartialEq, Eq)]pub struct {lang};");
        s!("#[derive(Debug, Copy, Clone, PartialEq, Eq, clap::ValueEnum)]");
        s!("pub enum Language {{");
        m!("    {lang},");
        s!("}}");
        s!("impl Display for Template<'_, Language> {{");
        s!("    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {{");
        s!("        match self.ctrl {{");
        m!("            Language::{lang} => self.transform({lang}).fmt(f),");
        s!("        }}");
        s!("    }}");
        s!("}}");

        use std::io::Write;
        let mut output = env::var_os("OUT_DIR").map(PathBuf::from).unwrap();
        output.push("templates.rs");
        File::options()
            .create(true)
            .write(true)
            .truncate(true)
            .open(output)?
            .write_all(self.buf.as_bytes())
    }
}
