# Yet Another Fetch (yaf)

![Screenshot](screenshot.webp)

## Installation

```sh
cargo install --git https://github.com/kaktu5/yaf
```

## Usage

```
Usage: yaf [options] [<config_path>]

Yet Another Fetch

Arguments:
  config_path        Config path, defaults to ~/.config/yaf.conf, uses
                     built-in config if the file does not exist.

Options:
  -s, --show-config  Prints built-in config to stdout.
  -v, --version      Prints version info.
  -h, --help         Show this help message and exit.
```

## Configuration

```sh
yaf -d > ~/.config/yaf.conf
nvim ~/.config/yaf.conf
```

## Uninstalling

```sh
cargo uninstall yaf
```
