# youtui-rs

A feature-rich YouTube terminal client in Rust built with ratatui.

## Features

- Search and play YouTube videos
- Watch history with persistence
- Save/bookmark videos
- Local playlists + YouTube playlist sync
- Download to disk
- 6 themed color schemes (terminal, tokyo, monokai, light, dark, retro)
- Full mouse support (click, scroll, context menus)
- Cross-platform (Linux-primary, macOS/Windows/BSD)

## Requirements

- Rust 1.70+
- mpv (for video playback)
- yt-dlp (for downloads and fallback)

## Installation

```bash
cargo install --path .
```

## Usage

```bash
youtui-rs
```

## Configuration

Configuration is stored at `~/.config/youtui-rs/config.json`

## License

MIT