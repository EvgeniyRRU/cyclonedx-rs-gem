# CycloneDX Rust Ruby Gem

Это порт на Rust существующего [CycloneDX Ruby Gem](https://github.com/CycloneDX/cyclonedx-ruby-gem/tree/master). Причина появления - крайне медленная работа `CycloneDX Ruby Gem` (опрашивает rubygems.org синхронно в один поток, а нужно для формирования `bom`-файла делать сотни запросов для рядового `rails`-проекта).

## Установка
1. Устанавливаем `Rust`
```shell
$ brew install rust
```
или следуем инструкциям на [официальном сайте](https://www.rust-lang.org/tools/install).

2. Клонируем этот репозиторий себе
```
$ git clone git@github.com:EvgeniyRRU/cyclonedx-rs-gem.git && cd cyclonedx-rs-gem
```
3. Собираем и устанавливаем программу
```
$ make install
```
## Использование
```shell
$ cyclonedx-rs-gem [options]

```
```
  -p, --path <PATH> - путь к папке, содержащей Gemfile.lock. Если не указан, то используется текущая папка
  -o, --output <OUTPUT> - путь к папке, куда будет записан bom-файл. Если опущен, то будет та же папка, где лежит Gemfile.lock
  -f, --format-file <FORMAT_FILE>  [default: json] [possible values: xml, json] - формат выходного файла
  -v, --verbose - нужно ли печатать дополнительную информацию
  -h, --help                       Print help
  -V, --version                    Print version
```
В результате успешной работы программы будет сгенерирован `bom.json` или `bom.xml` в указанной директории.
**Пример**
```shell
$ cyclonedx-rs-gem -p /Users/ruby/myrailsproject
```
