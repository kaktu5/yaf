# Yet Another Fetch (yaf)

![Screenshot](screenshot.webp)

## Installation
```sh
cargo install --git https://github.com/kktsdev/yaf
```

## Usage
```
Usage: yaf [-d] [-v] [<config_path>]

Yet Another Fetch

Arguments:
  config_path        Config path, defaults to ~/.config/yaf.conf, uses builtin
                     config if the file does not exist.

Options:
  -d, --dump-config  Dumps builtin config to stdout.
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
