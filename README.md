# CycloneDX Rust Ruby Gem

README in Russian [here](README_RU.md).

This is a port of the existing [CycloneDX Ruby Gem](https://github.com/CycloneDX/cyclonedx-ruby-gem/tree/master) to Rust. The reason for its appearance is the extremely slow performance of the `CycloneDX Ruby Gem` (it polls rubygems.org synchronously in one thread, while you need to make hundreds of requests to generate a `bom`-file for an average `Rails`-project).

## Install
1. Install `Rust`
```shell
$ brew install rust
```
or follow instruction on [official website](https://www.rust-lang.org/tools/install).

2. Clone this repository
```
$ git clone git@github.com:EvgeniyRRU/cyclonedx-rs-gem.git && cd cyclonedx-rs-gem
```
3. Build and install application
```
$ make install
```
## Usage
```shell
$ cyclonedx-rs-gem [options]

```
```
  -p, --path <PATH> - path to the folder containing Gemfile.lock. If not specified, the current folder is used
  -o, --output <OUTPUT> - path to the folder where the bom file will be recorded. If omitted, it will be the same folder where Gemfile.lock is located.
  -f, --format-file <FORMAT_FILE>  [default: json] [possible values: xml, json] - output file format
  -v, --verbose - should to print additional information
  -h, --help                       Print help
  -V, --version                    Print version
```
As a result of successful operation, `bom.json` or `bom.xml` will be generated in the specified directory.
**Example**
```shell
$ cyclonedx-rs-gem -p /Users/ruby/myrailsproject
```
