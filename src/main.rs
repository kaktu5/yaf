use argp::{help::HelpStyle, FromArgs};
use dirs::config_dir;
use phf::phf_map;
use std::{
    env,
    fs::File,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    process::{self, Command},
};
use thiserror::Error;

const BUILTIN_CONFIG: &str = include_str!("yaf.conf");
static STYLES: phf::Map<&str, &str> = phf_map! {
    "color0" => "\x1B[0;30m",
    "color1" => "\x1B[0;31m",
    "color2" => "\x1B[0;32m",
    "color3" => "\x1B[0;33m",
    "color4" => "\x1B[0;34m",
    "color5" => "\x1B[0;35m",
    "color6" => "\x1B[0;36m",
    "color7" => "\x1B[0;37m",
    "color8"  => "\x1B[1;30m",
    "color9"  => "\x1B[1;31m",
    "color10" => "\x1B[1;32m",
    "color11" => "\x1B[1;33m",
    "color12" => "\x1B[1;34m",
    "color13" => "\x1B[1;35m",
    "color14" => "\x1B[1;36m",
    "color15" => "\x1B[1;37m",
    "bold" => "\x1B[1m",
    "italic" => "\x1B[3m",
    "underline" => "\x1B[4m",
    "reset" => "\x1B[0m"
};

/// Yet Another Fetch
#[derive(FromArgs)]
struct Args {
    /// Config path, defaults to ~/.config/yaf.conf,
    /// uses builtin config if the file does not exist.
    #[argp(positional, default = "default_config_path()")]
    config_path: String,
    /// Dumps builtin config to stdout.
    #[argp(switch, short = 'd')]
    dump_config: bool,
    /// Prints version info.
    #[argp(switch, short = 'v')]
    version: bool,
}

#[derive(Error, Debug)]
enum MyError {
    #[error("Failed to read file: {0}")]
    FileRead(#[from] io::Error),
    #[error("Failed to execute command: {0}")]
    CommandExecution(String),
    #[error("Missing environment variable: {0}")]
    EnvVariable(String),
    #[error("Unexpected nested '{{' found")]
    NestedBrace,
    #[error("Unmatched '}}' found")]
    UnmatchedBrace,
    #[error("Unclosed '{{' found")]
    UnclosedBrace,
    #[error("Unknown variable: {0}")]
    UnknownBuiltinVariable(String),
}

fn main() {
    let args: Args = argp::parse_args_or_exit(&HelpStyle {
        wrap_width_range: 0..80,
        ..HelpStyle::default()
    });

    if args.dump_config {
        print!("{}", BUILTIN_CONFIG);
        return;
    }

    if args.version {
        print!(
            "yaf {} ({})\n",
            env!("CARGO_PKG_VERSION"),
            env!("GIT_COMMIT_HASH")
        );
        return;
    }

    let config: String = match open_file(Path::new(&args.config_path)) {
        Ok(file) => file,
        Err(_) => BUILTIN_CONFIG.to_string(),
    };

    let mut output: String = String::new();
    for (index, line) in config.lines().enumerate() {
        match parse_line(&line) {
            Ok(line) => output.push_str(&line),
            Err(err) => {
                reset_term_styles();
                eprintln!("Error in line {}: {}", index + 1, err);
                process::exit(1);
            }
        }
    }
    print!("{}", output);
    reset_term_styles();
}

fn parse_line(line: &str) -> Result<String, MyError> {
    let mut buffer: String = String::new();
    let mut output: String = String::new();
    let mut inside_braces: bool = false;
    let mut escape_next: bool = false;

    for char in line.chars() {
        if escape_next {
            match char {
                '{' | '}' | '\\' => buffer.push(char),
                _ => {
                    buffer.push('\\');
                    buffer.push(char);
                }
            }
            escape_next = false;
            continue;
        }

        match char {
            '\\' => {
                if inside_braces {
                    escape_next = true;
                } else {
                    output.push('\\');
                }
            }
            '{' => {
                if inside_braces {
                    return Err(MyError::NestedBrace);
                }
                inside_braces = true;
                buffer.clear();
            }
            '}' => {
                if !inside_braces {
                    return Err(MyError::UnmatchedBrace);
                }
                inside_braces = false;

                if buffer.starts_with('$') {
                    output.push_str(&get_env(&buffer[1..])?);
                } else if buffer.starts_with('@') {
                    if let Some(replacement) = STYLES.get(&buffer.as_str()[1..]) {
                        output.push_str(replacement);
                    } else {
                        return Err(MyError::UnknownBuiltinVariable(buffer.clone()));
                    }
                } else {
                    output.push_str(&run_sh(&buffer)?);
                }
            }
            _ => {
                if inside_braces {
                    buffer.push(char);
                } else {
                    output.push(char);
                }
            }
        }
    }

    if inside_braces {
        return Err(MyError::UnclosedBrace);
    }

    output.push('\n');
    Ok(output)
}

fn run_sh(command: &str) -> Result<String, MyError> {
    let output = Command::new("/bin/sh").arg("-c").arg(command).output()?;

    let stdout = String::from_utf8_lossy(&output.stdout)
        .trim_end()
        .to_string();
    let stderr = String::from_utf8_lossy(&output.stderr)
        .trim_end()
        .to_string();

    if !stderr.is_empty() {
        return Err(MyError::CommandExecution(stderr));
    }

    Ok(stdout)
}

fn get_env(key: &str) -> Result<String, MyError> {
    env::var(key).map_err(|_| MyError::EnvVariable(key.to_string()))
}

fn open_file(path: &Path) -> Result<String, io::Error> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

fn default_config_path() -> String {
    let mut path: PathBuf = config_dir().unwrap();
    path.push("yaf.conf");
    path.to_string_lossy().to_string()
}

fn reset_term_styles() {
    print!("\x1B[0m");
    io::stdout().flush().unwrap();
}
