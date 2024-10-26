use std::fs::{read_to_string, write};
use std::io::{stdin, Read};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use biome_console::{markup, ColorMode, Console, EnvConsole, LogLevel};
use biome_diagnostics::PrintDiagnostic;
use biome_js_syntax::JsFileSource;
use clap::{Parser, ValueEnum};
use glob::glob;
use tsimports::{tsimports, Error};
use walkdir::WalkDir;

#[derive(Copy, Clone, Debug, Default, ValueEnum)]
enum Language {
    #[default]
    JS,
    JSX,
    TS,
    TSX,
}

impl Language {
    fn from_extension(ext: &str) -> Option<Self> {
        match ext {
            "js" | "mjs" => Some(Self::JS),
            "jsx" => Some(Self::JSX),
            "ts" | "cts" | "mts" => Some(Self::TS),
            "tsx" => Some(Self::TSX),
            _ => None,
        }
    }

    fn from_path(path: impl AsRef<Path>) -> Option<Self> {
        Self::from_extension(path.as_ref().extension().and_then(|ext| ext.to_str())?)
    }

    fn to_file_source(&self) -> JsFileSource {
        match self {
            Self::JS => JsFileSource::js_module(),
            Self::JSX => JsFileSource::jsx(),
            Self::TS => JsFileSource::ts(),
            Self::TSX => JsFileSource::tsx(),
        }
    }
}

#[derive(Debug, Parser)]
#[command(about, version)]
struct Args {
    /// Paths or globs of files to organize imports. Defaults to the standard input.
    paths: Option<Vec<String>>,

    /// Specify the language to parse the input as. Inferred by the file extension by default.
    #[clap(short, long)]
    language: Option<Language>,

    /// Write the formatted result into the file directly, without printing to the standard output.
    #[clap(short, long)]
    write: bool,
}

fn main() {
    let mut console = EnvConsole::new(ColorMode::Auto);
    if let Err(e) = run(&mut console) {
        console.println(
            LogLevel::Error,
            markup! { <Error>"\u{2716} "{e.to_string()}</Error> },
        )
    }
}

fn run(console: &mut impl Console) -> Result<()> {
    let args = Args::parse();

    if let Some(paths) = &args.paths {
        for path in paths.iter() {
            for entry in glob(path)? {
                let entry = entry?;
                if entry.is_dir() {
                    WalkDir::new(entry)
                        .into_iter()
                        .filter_map(|e| e.ok())
                        .filter_map(|e| {
                            let path = e.path().to_path_buf();
                            match Language::from_path(&path) {
                                Some(lang) => Some((path, lang)),
                                _ => None,
                            }
                        })
                        .map(|(path, lang)| {
                            run_single(Input::File(path, Some(lang)), &args, console)
                        })
                        .collect::<Result<()>>()?;
                } else {
                    let lang = Language::from_path(&entry);

                    run_single(Input::File(entry, lang), &args, console)?;
                }
            }
        }
    } else {
        if args.language.is_none() {
            console.println(LogLevel::Error, markup! {
                <Warn>
                    "\u{26a0} Input language is not specified, assuming as an ECMAScript module. Use "
                    <Emphasis>"--language <js|jsx|ts|tsx>"</Emphasis>
                    " option to override."
                </Warn>
            });
        }

        run_single(Input::Stdin, &args, console)?;
    }

    Ok(())
}

enum Input {
    File(PathBuf, Option<Language>),
    Stdin,
}

fn run_single(input: Input, args: &Args, console: &mut impl Console) -> Result<()> {
    let mut source = JsFileSource::js_module();
    let buf = match &input {
        Input::File(path, lang) => {
            if let Some(lang) = lang {
                source = lang.to_file_source();
            }

            read_to_string(path)?
        }
        _ => {
            let mut buf = String::new();
            stdin().read_to_string(&mut buf)?;
            buf
        }
    };

    if let Some(lang) = args.language {
        source = lang.to_file_source();
    }

    let output = match tsimports(buf.as_str(), source) {
        Ok(o) => o,
        Err(e) => match e {
            Error::Parser(diags) => {
                for diag in diags {
                    console.println(
                        LogLevel::Error,
                        markup! { {PrintDiagnostic::verbose(&diag)} },
                    );
                }

                std::process::exit(1);
            }
            _ => return Err(anyhow!(e)),
        },
    };

    if args.write {
        let Input::File(path, _) = input else {
            return Err(anyhow!(
                "Can't write the result as the input was from the standard input."
            ));
        };

        write(path, output)?;
    } else {
        print!("{}", output);
    }

    Ok(())
}
