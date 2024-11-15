use std::env::var_os;
use std::fs::create_dir_all;
use std::fs::exists;
use std::fs::read_dir;
use std::fs::read_to_string;
use std::fs::File;
use std::io::BufWriter;
use std::io::Error;
use std::io::Write;

use clap::builder::PossibleValuesParser;
use clap::Arg;
use clap::CommandFactory;
use clap_complete::generate_to;
use clap_complete::Shell;

const DIR: &str = "completions";
const APP: &str = "rustcodex";

include!("src/cli.rs");

fn main() -> Result<(), Error> {
    let mut output = var_os("OUT_DIR").map(PathBuf::from).unwrap();
    output.push("templates.rs");
    let mut target = BufWriter::new(
        File::options()
            .create(true)
            .write(true)
            .truncate(true)
            .open(output)?,
    );

    let mut codegen = TemplateGen::new();
    for template in read_dir("templates")? {
        let file = template?;
        assert!(file.file_type()?.is_file(), "template must be a file");
        let name = file.file_name();
        let name = name.to_str().expect("UTF-8 name");

        let language = name
            .split_once('.')
            .expect("template name must be in format `language.suffix`")
            .0;
        let template = read_to_string(file.path())?;

        codegen.add(template, language);
    }

    codegen.generate(&mut target)?;

    let langs = codegen
        .langs
        .iter()
        .map(|lang| lang.name.to_ascii_lowercase());
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

    if !exists(DIR)? {
        create_dir_all(DIR)?;
    }

    for shell in [Shell::Bash, Shell::Zsh, Shell::Fish, Shell::PowerShell] {
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
    template: String,
}

impl Language {
    const S: &str = "__SOURCE__";
    const P: &str = "__PAYLOAD__";

    fn new(template: String, name: &str) -> Self {
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
        let msg = "language name must be only Ascii alphabetic";
        Self {
            template,
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

    fn components(&self) -> [&str; 5] {
        let source = self
            .template
            .lines()
            .find(|line| line.contains(Self::S))
            .unwrap();
        let (precomment, postcomment) = source.split_once(Language::S).unwrap();
        let (start, mid, end) = self
            .template
            .split_once(source)
            .and_then(|(s, t)| t.split_once(Self::P).map(|(m, e)| (s, m, e)))
            .unwrap();
        [precomment, postcomment, start, mid, end]
    }
}

struct TemplateGen {
    langs: Vec<Language>,
}

impl TemplateGen {
    /// Setup codegen
    fn new() -> Self {
        Self { langs: Vec::new() }
    }

    /// Add language definition to generator
    fn add(&mut self, template: String, name: &str) {
        self.langs.push(Language::new(template, name));
    }

    fn generate<W: Write>(&self, target: &mut W) -> Result<(), Error> {
        macro_rules! s {
            ($($arg:tt)*) => {
                writeln!(target, $($arg)*)?
            };
        }
        macro_rules! m {
            ($($arg:tt)*) => {
                for lang in &self.langs {
                    s!($($arg)*, lang=lang.name)
                }
            };
        }

        // generate fmt routines
        for lang in &self.langs {
            let [pre, post, start, mid, end] = lang.components().map(str::escape_debug);
            let name = &lang.name;
            s!(r#"/// {name} template parameter, to be used in `Template<'_, {name}>`"#);
            s!(r#"#[derive(Debug, Copy, Clone, PartialEq, Eq)]"#);
            s!(r#"pub struct {name};"#);
            s!(r#"impl Display for Template<'_, {name}> {{"#);
            s!(r#"    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {{"#);
            s!(r#"        let Data {{ payload, sources }} = self.data;"#);
            s!(r#"        let source = CodeInliner {{"#);
            s!(r#"            files: sources,"#);
            s!(r#"            start: "{pre}","#);
            s!(r#"            end: "{post}","#);
            s!(r#"        }};"#);
            s!(r#"        let payload = Compressor {{ payload }};"#);
            s!(r#"        f.write_str("{start}")?;"#);
            s!(r#"        write!(f, "{{source}}")?;"#);
            s!(r#"        f.write_str("{mid}")?;"#);
            s!(r#"        write!(f, "{{payload}}")?;"#);
            s!(r#"        f.write_str("{end}")?;"#);
            s!(r#""#);
            s!(r#"        Ok(())"#);
            s!(r#"    }}"#);
            s!(r#"}}"#);
        }
        let count = self.langs.len();
        // generate enumeration
        s!(r#"/// Enumeration of all available languages"#);
        s!(r#"#[derive(Debug, Copy, Clone, PartialEq, Eq, clap::ValueEnum)]"#);
        s!(r#"pub enum Language {{"#);
        m!(r#"    {lang},"#);
        s!(r#"}}"#);
        s!(r#"impl Language {{"#);
        s!(r#"    /// Number of included languages"#);
        s!(r#"    pub const COUNT: usize = {count};"#);
        s!(r#"    /// Array of all included languages"#);
        s!(r#"    pub const fn all() -> [Self; Self::COUNT] {{"#);
        s!(r#"        ["#);
        m!(r#"            Self::{lang},"#);
        s!(r#"        ]"#);
        s!(r#"    }}"#);
        s!(r#"}}"#);
        s!(r#"impl Display for Template<'_, Language> {{"#);
        s!(r#"    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {{"#);
        s!(r#"        match self.ctrl {{"#);
        m!(r#"            Language::{lang} => self.transform({lang}).fmt(f),"#);
        s!(r#"        }}"#);
        s!(r#"    }}"#);
        s!(r#"}}"#);

        Ok(())
    }
}
