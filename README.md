# Weather app
CLI tool to get current and forecast weather from multiple providers

### Basic usage

`weather configure [provider]` - configure credentials for `[provider]`

`weather set-default [provider]`

`weather get <address> [--date]` - show weather for the provided `<address>`. By default shows current weather, but you can specify date with `--date/-d` flag

`weather --help` - list commands

### Supported providers
1. OpenWeather
2. Accuweather