# Weather app
CLI tool to get current and forecast weather from multiple providers

### Basic usage

Configure credentials for `[provider]`:
```
weather configure [provider]
```
Set provider to use by default
```
weather set-default [provider]
```
Show weather for the provided `<address>`. If no date provided, shows current weather
```
weather get <address> [date]
```

### Supported providers

1. OpenWeather
2. Accuweather
