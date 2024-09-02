# h5_dumper
Simple tag dumper for Halo 5 and Halo 5 Forge written in Rust. Recursively goes through each ".module" file in directory and writes tags to specified path.

## Usage
```
Usage: h5_dumper.exe --module-path <MODULE_PATH> --save-path <SAVE_PATH>

Options:
  -m, --module-path <MODULE_PATH>  Path to where modules are located (deploy folder)
  -s, --save-path <SAVE_PATH>      Path to save tags to
  -h, --help                       Print help
  -V, --version                    Print version
```
