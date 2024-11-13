use std::env;
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
    // Generate code for each template in `templates` directory
    let mut generator = Codegenerator::new();
    for template in read_dir("templates")? {
        let file = template?;
        assert!(file.file_type()?.is_file());
        let name = file.file_name();
        let name = name.to_str().expect("UTF-8 name");

        let language = name
            .split_once(".")
            .expect("template name must be in format `language.suffix`")
            .0;
        let template = read_to_string(file.path())?;
        generator.add(template.as_str(), language);
    }
    generator.finalize();
    generator.generate()?;

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
            .value_parser(PossibleValuesParser::new(
                generator.langs.iter().map(|lang| &lang.original),
            )),
    );

    for shell in [Shell::Bash, Shell::Zsh, Shell::Fish] {
        generate_to(shell, &mut app, APP, DIR)?;
    }

    println!("cargo::rerun-if-changed=templates");
    println!("cargo::rerun-if-changed=src/cli.rs");
    println!("cargo::rerun-if-changed=src/target.rs");
    println!("cargo::rustc-cfg=nonrecursive");

    Ok(())
}

struct LangDef {
    original: String,
    idiomatic: String,
}

struct Codegenerator {
    // could be written directly to a file, but templates should be small
    buf: String,
    langs: Vec<LangDef>,
}

impl Codegenerator {
    const S: &str = "__SOURCE__";
    const P: &str = "__PAYLOAD__";

    fn new() -> Self {
        Self {
            buf: String::from("// Generated from `build.rs`. DO NOT EDIT!\n"),
            langs: Vec::new(),
        }
    }

    fn add(&mut self, template: &str, name: &str) {
        let assertion = |tag| move || panic!("template must contain single {tag} directive");
        template
            .split_once(Self::S)
            .and_then(|(_, post)| (!post.contains(Self::S)).then_some(post))
            .unwrap_or_else(assertion(Self::S))
            .split_once(Self::P)
            .and_then(|(_, post)| (!post.contains(Self::P)).then_some(post))
            .unwrap_or_else(assertion(Self::P));

        let language: String = name
            .chars()
            .inspect(|char| assert!(char.is_ascii_alphabetic()))
            .enumerate()
            .map(|(index, char)| match index {
                0 => char.to_ascii_uppercase(),
                _ => char.to_ascii_lowercase(),
            })
            .collect();

        let source = template
            .lines()
            .find(|line| line.contains(Self::S))
            .expect("precognition");
        let (precomment, postcomment) = source.split_once("__SOURCE__").expect("precognition");
        let (start, mid, end) = template
            .split_once(source)
            .and_then(|(start, tmp)| tmp.split_once(Self::P).map(|(mid, end)| (start, mid, end)))
            .expect("assertion at start is correct");

        let [precomment, postcomment, start, mid, end] =
            [precomment, postcomment, start, mid, end].map(str::escape_debug);

        write!(
            self.buf,
            r#"
impl Display for Template<'_, {language}> {{
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
        .expect("internal error: fmt fail");

        self.langs.push(LangDef {
            original: name.into(),
            idiomatic: language,
        });
    }

    fn finalize(&mut self) {
        let i = &mut self.buf;

        // NOTE: This is ugly, but works.
        for lang in &self.langs {
            i.push_str("#[derive(Debug, Copy, Clone, PartialEq, Eq)]\n");
            i.push_str("pub struct ");
            i.push_str(&lang.idiomatic);
            i.push_str(";\n\n");
            i.push_str("impl Display for \n");
            i.push_str(&lang.idiomatic);
            i.push_str("{\n");
            i.push_str("    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {\n");
            i.push_str("        f.write_str(\"");
            i.push_str(&lang.original);
            i.push_str("\")");
            i.push_str("    }\n");
            i.push_str("}\n\n");
        }
        i.push_str("#[derive(Debug, Copy, Clone, PartialEq, Eq, clap::ValueEnum)]\n");
        i.push_str("pub enum Language {\n");
        for lang in &self.langs {
            i.push_str("    ");
            i.push_str(&lang.idiomatic);
            i.push_str(",\n");
        }
        i.push_str("}\n\n");
        i.push_str("impl Display for Template<'_, Language> {\n");
        i.push_str("    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {\n");
        i.push_str("        match self.ctrl {\n");
        for lang in &self.langs {
            i.push_str("            Language::");
            i.push_str(&lang.idiomatic);
            i.push_str(" => self.transform(");
            i.push_str(&lang.idiomatic);
            i.push_str(").fmt(f),\n");
        }
        i.push_str("        }\n");
        i.push_str("    }\n");
        i.push_str("}\n");
    }

    fn generate(&mut self) -> Result<(), Error> {
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
