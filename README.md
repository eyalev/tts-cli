# TTS-CLI

A command-line text-to-speech tool with multiple providers and intelligent caching.

## Features

- **Multiple TTS Providers**: Google Cloud TTS, eSpeak, Festival, macOS Say
- **Intelligent Caching**: Avoid repeated API calls for the same text
- **Multiple Languages**: Support for various languages and voices
- **Audio Playback**: Direct audio playback or save to file
- **Easy Distribution**: Single binary with no dependencies

## Installation

### From Source

```bash
git clone <repository-url>
cd tts-cli
cargo build --release
```

The binary will be available at `target/release/tts-cli`.

### Install to ~/.local/bin

```bash
cargo install --path .
# or
cp target/release/tts-cli ~/.local/bin/
```

## Usage

### Basic Usage

```bash
# Speak text using default provider (Google Cloud)
tts-cli speak "Hello, world!"

# Use a specific provider
tts-cli speak "Hello, world!" --provider espeak

# Specify language and voice
tts-cli speak "Hola mundo" --language es-ES --voice es-ES-Wavenet-C

# Save to file instead of playing
tts-cli speak "Hello, world!" --output hello.mp3
```

### Cache Management

```bash
# Disable cache for this request
tts-cli speak "Hello, world!" --no-cache

# Clear cache for specific text
tts-cli speak "Hello, world!" --clear-cache

# Clear all cache
tts-cli clear-cache

# Show cache statistics
tts-cli cache-stats
```

### Provider Management

```bash
# List available providers
tts-cli providers
```

## Providers

### Google Cloud TTS

Requires Google Cloud credentials:

```bash
# Set up authentication
export GOOGLE_APPLICATION_CREDENTIALS="path/to/service-account-key.json"

# Or use gcloud CLI
gcloud auth application-default login
```

### eSpeak

Install eSpeak on your system:

```bash
# Ubuntu/Debian
sudo apt install espeak

# macOS
brew install espeak

# Fedora
sudo dnf install espeak
```

### Festival

Install Festival on your system:

```bash
# Ubuntu/Debian
sudo apt install festival

# macOS
brew install festival

# Fedora
sudo dnf install festival
```

### macOS Say

Built-in on macOS systems, no installation required.

## Configuration

Configuration is automatically created at `~/.config/tts-cli/config.json`:

```json
{
  "default_provider": "gcloud",
  "default_language": "en-US",
  "default_voice": null,
  "cache_enabled": true,
  "providers": {
    "gcloud": {
      "enabled": true,
      "api_key": null,
      "endpoint": null,
      "voice_mapping": {
        "en-US": "en-US-Wavenet-D",
        "es-ES": "es-ES-Wavenet-C",
        "fr-FR": "fr-FR-Wavenet-D",
        "de-DE": "de-DE-Wavenet-D"
      }
    }
  }
}
```

## Examples

```bash
# Basic usage
tts-cli speak "Welcome to TTS CLI"

# Spanish with specific voice
tts-cli speak "¡Hola mundo!" -l es-ES -v es-ES-Wavenet-C

# Using eSpeak for offline TTS
tts-cli speak "This works offline" -p espeak

# Save to file
tts-cli speak "Save this audio" -o output.mp3

# Long text with caching
tts-cli speak "This is a long text that will be cached for future use"
tts-cli speak "This is a long text that will be cached for future use"  # Uses cache

# Clear specific cache
tts-cli speak "Clear this from cache" --clear-cache
```

## Build Features

- **Async/Await**: Built with Tokio for efficient async operations
- **Error Handling**: Comprehensive error handling with anyhow
- **Configuration**: JSON-based configuration with serde
- **Caching**: SHA256-based cache keys for reliable caching
- **Audio Playback**: Built-in audio playback with rodio

## License

MIT OR Apache-2.0