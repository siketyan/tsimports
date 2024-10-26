use std::fs::{read_to_string, write};
use std::io::{stdin, Read};
use std::path::PathBuf;

use anyhow::{anyhow, Result};
use biome_console::{markup, ColorMode, Console, EnvConsole, LogLevel};
use biome_diagnostics::PrintDiagnostic;
use biome_js_syntax::JsFileSource;
use clap::{Parser, ValueEnum};
use glob::glob;
use tsimports::{tsimports, Error};

#[derive(Copy, Clone, Debug, Default, ValueEnum)]
enum Language {
    #[default]
    JS,
    JSX,
    TS,
    TSX,
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
                run_single(Input::File(entry?), &args, console)?;
            }
        }
    } else {
        run_single(Input::Stdin, &args, console)?;
    }

    Ok(())
}

enum Input {
    File(PathBuf),
    Stdin,
}

fn run_single(input: Input, args: &Args, console: &mut impl Console) -> Result<()> {
    let mut source = JsFileSource::js_module();
    let buf = match &input {
        Input::File(path) => {
            if let Some(ext) = path.extension().and_then(|ext| ext.to_str()) {
                match ext {
                    "jsx" => source = JsFileSource::jsx(),
                    "ts" => source = JsFileSource::ts(),
                    "tsx" => source = JsFileSource::tsx(),
                    _ => {}
                }
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
        use Language::*;

        source = match lang {
            JS => JsFileSource::js_module(),
            JSX => JsFileSource::jsx(),
            TS => JsFileSource::ts(),
            TSX => JsFileSource::tsx(),
        }
    } else {
        console.println(LogLevel::Error, markup! {
        <Warn>
            "\u{26a0} Input language is not specified, assuming as an ECMAScript module. Use "
            <Emphasis>"--language <js|jsx|ts|tsx>"</Emphasis>
            " option to override."
        </Warn>
    });
    };

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
        let Input::File(path) = input else {
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
