mod fetch;

use argp::{help::HelpStyle, FromArgs};
use dirs::config_dir;
use fetch::*;
use std::{
    env,
    fs::File,
    io::{self, Read, Write},
    path::Path,
    process::{self, Command},
};
use thiserror::Error;

const BUILTIN_CONFIG: &str = include_str!("yaf.conf");
const BUILTIN_VARS: &[&str] = &[
    "distro", "hostname", "kernel", "pkgs", "shell", "uptime", "username",
];
const ERROR_STR: &str = "ERROR";
pub const NOT_AVAILABLE_STR: &str = "N/A";

#[derive(Error, Debug)]
enum ConfigError {
    #[error("Failed to read file: {0}")]
    FileRead(#[from] io::Error),
    #[error("Unexpected curly brace found.")]
    UnexpectedCurlyBrace,
    #[error("Unknown variable: {0}")]
    UnknownVariable(String),
}

/// Yet Another Fetch
#[derive(FromArgs)]
struct Args {
    /// Config path, defaults to ~/.config/yaf.conf,
    /// uses built-in config if the file does not exist.
    #[argp(positional, default = "default_config_path()")]
    config_path: String,
    /// Prints built-in config to stdout.
    #[argp(switch, short = 's')]
    show_config: bool,
    /// Prints version info.
    #[argp(switch, short = 'v')]
    version: bool,
}

fn default_config_path() -> String {
    config_dir()
        .unwrap()
        .join("yaf.conf")
        .to_string_lossy()
        .to_string()
}

fn main() {
    let args: Args = argp::parse_args_or_exit(&HelpStyle {
        short_usage: true,
        wrap_width_range: 0..70,
        ..HelpStyle::default()
    });

    if args.show_config {
        print!("{}", BUILTIN_CONFIG);
        return;
    }

    if args.version {
        println!(
            "yaf {} ({})",
            env!("CARGO_PKG_VERSION"),
            env!("GIT_COMMIT_HASH")
        );
        return;
    }

    let config: String = match open_file(Path::new(&args.config_path)) {
        Ok(file) => file,
        Err(_) => String::from(BUILTIN_CONFIG),
    };

    let mut output: String = String::new();
    for (index, line) in config.lines().enumerate() {
        match parse_line(line) {
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

fn parse_line(line: &str) -> Result<String, ConfigError> {
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
                    return Err(ConfigError::UnexpectedCurlyBrace);
                }
                inside_braces = true;
                buffer.clear();
            }
            '}' => {
                if !inside_braces {
                    return Err(ConfigError::UnexpectedCurlyBrace);
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
        return Err(ConfigError::UnexpectedCurlyBrace);
    }

    output.push('\n');
    Ok(output)
}

fn parse_var(var: &str) -> Result<String, ConfigError> {
    match var {
        _ if var.starts_with('@') => Ok(replace_var(&var[1..])?),
        _ if var.starts_with('$') => Ok(get_env(&var[1..])),
        _ if var.starts_with('#') => Ok(run_sh(&var[1..])),
        _ if var.starts_with('c') => Ok(var[1..]
            .trim()
            .parse::<u8>()
            .map(|c| format!("\x1B[38;5;{}m", c))
            .map_err(|_| ConfigError::UnknownVariable(String::from(var)))?),
        _ => Err(ConfigError::UnknownVariable(String::from(var))),
    }
}

fn replace_var(key: &str) -> Result<String, ConfigError> {
    if !BUILTIN_VARS.contains(&key) {
        return Err(ConfigError::UnknownVariable(String::from(key)));
    }

    Ok(match key {
        "distro" => get_distro(),
        "hostname" => get_hostname(),
        "kernel" => get_kernel(),
        "pkgs" => get_pkgs(),
        "shell" => get_shell(),
        "uptime" => get_uptime(),
        "username" => get_username(),
        _ => unreachable!(),
    })
}

fn run_sh(command: &str) -> String {
    let output = Command::new("/bin/sh")
        .args(["-c", command])
        .output()
        .map_err(|_| String::from(ERROR_STR))
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout)
        .trim_end()
        .to_string();
    let stderr = String::from_utf8_lossy(&output.stderr)
        .trim_end()
        .to_string();

    if stderr.is_empty() {
        stdout
    } else {
        stderr
    }
}

fn get_env(key: &str) -> String {
    env::var(key).unwrap_or(String::from(NOT_AVAILABLE_STR))
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
