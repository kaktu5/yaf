mod fetch;

use argp::{help::HelpStyle, FromArgs};
use dirs::config_dir;
use env_logger::{Builder, Env};
use fetch::*;
use log::{error, warn};
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
    "color" => "\x1B[38;5;{}m",
    "bold" => "\x1B[1m",
    "italic" => "\x1B[3m",
    "underline" => "\x1B[4m",
    "reset" => "\x1B[0m",
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

fn default_config_path() -> String {
    let mut path: PathBuf = config_dir().unwrap();
    path.push("yaf.conf");
    path.to_string_lossy().to_string()
}

#[derive(Error, Debug)]
enum YafError {
    #[error("Failed to read file: {0}")]
    FileRead(#[from] io::Error),
    #[error("Unexpected curly brace found.")]
    UnexpectedCurlyBrace,
    #[error("Unknown variable: {0}")]
    UnknownVariable(String),
    #[error("Failed to execute command: {0}")]
    CommandExecution(String),
    #[error("Missing environment variable: {0}")]
    UnknownEnvVariable(String),
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

    Builder::from_env(Env::default().default_filter_or("warn")).init();

    let config: String = match open_file(Path::new(&args.config_path)) {
        Ok(file) => file,
        Err(_) => {
            warn!("Using builtin config.");
            BUILTIN_CONFIG.to_string()
        }
    };

    let mut output: String = String::new();
    for (index, line) in config.lines().enumerate() {
        match parse_line(&line) {
            Ok(line) => output.push_str(&line),
            Err(err) => {
                reset_term_styles();
                error!("Error in line {}: {}", index + 1, err);
                process::exit(1);
            }
        }
    }

    print!("{}", output);
    reset_term_styles();
}

fn parse_line(line: &str) -> Result<String, YafError> {
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
                    return Err(YafError::UnexpectedCurlyBrace);
                }
                inside_braces = true;
                buffer.clear();
            }
            '}' => {
                if !inside_braces {
                    return Err(YafError::UnexpectedCurlyBrace);
                }
                inside_braces = false;
                output.push_str(&parse_var(&buffer)?);
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
        return Err(YafError::UnexpectedCurlyBrace);
    }

    output.push('\n');
    Ok(output)
}

fn parse_var(var: &str) -> Result<String, YafError> {
    match var {
        _ if var.starts_with('$') => get_env(&var[1..]),
        _ if var.starts_with('@') => replace_var(&var[1..]),
        _ if var.starts_with('#') => run_sh(&var[1..]),
        _ => Err(YafError::UnknownVariable(var.to_string())),
    }
}

fn replace_var(key: &str) -> Result<String, YafError> {
    match key {
        "username" => {
            let username = get_username();
            Ok(username)
        }
        "hostname" => {
            let hostname = get_hostname();
            Ok(hostname)
        }
        "distro" => {
            let distro = get_distro();
            Ok(distro)
        }
        "kernel" => Ok(get_kernel()),
        _ if key.starts_with("color") => {
            let suffix = key["color".len()..].trim();
            suffix
                .parse::<u8>()
                .ok()
                .and_then(|color| {
                    STYLES
                        .get("color")
                        .map(|format_string| format_string.replace("{}", &color.to_string()))
                })
                .ok_or_else(|| YafError::UnknownVariable(suffix.to_string()))
        }
        _ => STYLES
            .get(key)
            .map(|&value| value.to_string())
            .ok_or(YafError::UnknownVariable(key.to_string())),
    }
}

fn run_sh(command: &str) -> Result<String, YafError> {
    let output = Command::new("/bin/sh").arg("-c").arg(command).output()?;

    let stdout = String::from_utf8_lossy(&output.stdout)
        .trim_end()
        .to_string();
    let stderr = String::from_utf8_lossy(&output.stderr)
        .trim_end()
        .to_string();

    if !stderr.is_empty() {
        return Err(YafError::CommandExecution(stderr));
    }

    Ok(stdout)
}

fn get_env(key: &str) -> Result<String, YafError> {
    env::var(key).map_err(|_| YafError::UnknownEnvVariable(key.to_string()))
}

fn open_file(path: &Path) -> Result<String, io::Error> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

fn reset_term_styles() {
    print!("\x1B[0m");
    io::stdout().flush().unwrap();
}
