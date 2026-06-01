# youtui-rs

A fast YouTube client for your terminal, written in Rust with [ratatui](https://ratatui.rs).
Search, play, and download videos or audio without ever opening a browser.

```
  ██╗   ██╗ ██████╗ ██╗   ██╗████████╗██╗   ██╗██╗
  ╚██╗ ██╔╝██╔═══██╗██║   ██║╚══██╔══╝██║   ██║██║
   ╚████╔╝ ██║   ██║██║   ██║   ██║   ██║   ██║██║
    ╚██╔╝  ██║   ██║██║   ██║   ██║   ██║   ██║██║
     ██║   ╚██████╔╝╚██████╔╝   ██║   ╚██████╔╝██║
     ╚═╝    ╚═════╝  ╚═════╝    ╚═╝    ╚═════╝ ╚═╝
```

[![Hits](https://hits.seeyoufarm.com/api/count/incr/badge.svg?url=https%3A%2F%2Fgithub.com%2FZeroNeroIV%2Fyoutui.git&count_bg=%2379C83D&title_bg=%23555555&icon=&icon_color=%23E7E7E7&title=hits&edge_flat=false)](https://hits.seeyoufarm.com)

## Features

- **Search** YouTube via Invidious, with automatic fallback across multiple
  public instances (keeps working even while a download is running)
- **Play** videos/audio through mpv
- **Download** as video (mp4) or audio (mp3) — choose from a popup on `d`
- **Downloads page** lists everything you've downloaded; press Enter to play a
  local file
- **Animated bottom bar** shows live download progress (%, speed, ETA) and
  playback status (loading / playing + elapsed time / errors)
- **Watch history**, **saved videos**, and **local + imported playlists**, all
  persisted in SQLite
- **Settings**: cycle media player (mpv / vlc / mplayer / ffplay), pick your
  download folder with [Yazi](https://github.com/sxyazi/yazi), set quality,
  format, and more
- Uses your **terminal's own color scheme** — no hard-coded themes to clash
  with your setup
- Full mouse support (click, scroll, right-click context menu)

## Requirements

- [Rust](https://rustup.rs) 1.70 or newer (to build)
- [`mpv`](https://mpv.io) — video/audio playback
- [`yt-dlp`](https://github.com/yt-dlp/yt-dlp) — downloads and stream resolution
- [`yazi`](https://github.com/sxyazi/yazi) — *optional*, for the folder picker

On Arch Linux:

```bash
sudo pacman -S mpv yt-dlp yazi
```

On Debian/Ubuntu:

```bash
sudo apt install mpv
# yt-dlp: see https://github.com/yt-dlp/yt-dlp#installation
```

## Installation

### Option A — cargo install (recommended)

```bash
cargo install --path .
# installs the `youtui-rs` binary into ~/.cargo/bin
```

### Option B — build and symlink

Build the optimized release binary, then create a symlink named `youtui` on
your `PATH` so you can launch it from anywhere:

```bash
# 1. Clone and enter the repo
git clone https://github.com/ZeroNeroIV/youtui.git
cd youtui

# 2. Build the release binary
cargo build --release

# 3. Symlink it as `youtui` into a directory on your PATH
mkdir -p ~/.local/bin
ln -sf "$(pwd)/target/release/youtui-rs" ~/.local/bin/youtui-rs

# 4. Make sure ~/.local/bin is on your PATH (add to ~/.bashrc or ~/.zshrc if not)
export PATH="$HOME/.local/bin:$PATH"
```

The symlink points at the build output, so future `cargo build --release` runs
update the `youtui-rs` command automatically — no need to re-link.

## Usage

```bash
youtui-rs            # launch the app
```

### Keybindings

| Key       | Action                                  |
|-----------|-----------------------------------------|
| `s`       | Search                                  |
| `h`       | History                                 |
| `v`       | Saved videos                            |
| `p`       | Playlists                               |
| `Tab`     | Toggle focus (sidebar ↔ list)           |
| `↑` / `↓` | Navigate                                |
| `Enter`   | Play selected (or open / play download) |
| `d`       | Download — opens Video / Audio popup    |
| `a`       | Add to playlist                         |
| `Esc`     | Back / close popup                      |
| `/`       | Show keybindings                        |
| `q`       | Quit                                    |

### Playback controls

While something is playing, these work from anywhere in the app (except while
typing in a search box):

| Key       | Action                          |
|-----------|---------------------------------|
| `Space`   | Pause / resume                  |
| `←` / `→` | Seek backward / forward 10s     |
| `<` / `>` | Previous / next in the list     |

**Hardware media keys (headset / keyboard play-pause-next):** these are routed
on Linux through MPRIS. Install [`mpv-mpris`](https://github.com/hoyon/mpv-mpris)
and youtui will load it automatically, so your headset's play/pause and
next/previous buttons control playback:

```bash
# Arch
sudo pacman -S mpv-mpris
# Fedora
sudo dnf install mpv-mpris
# Debian/Ubuntu
sudo apt install mpv-mpris
```

You can also control playback from any MPRIS client, e.g.:

```bash
playerctl play-pause
playerctl next
```

### Download popup

Pressing `d` on any video opens a popup:

- `1` or `v` → download as **video** (mp4)
- `2` or `a` → download as **audio** (mp3)
- `Esc` → cancel

Files are saved to your configured download folder (defaults to `~/Videos`).
Open the **Downloads** page from the sidebar and press `Enter` to play a saved
file.

## Configuration

Settings live at `~/.config/youtui-rs/config.json` and can be edited in-app from
the **Settings** screen (press `Enter` on a row to change it):

- **Player** — cycle mpv / vlc / mplayer / ffplay
- **Download Path** — opens the Yazi folder picker
- **Quality**, **Format**, **Log Level** — cycle through options
- **Auto-play**, **Loop** — toggle

Other state:

- Download history: `~/.config/youtui-rs/downloads.json`
- Database (history, saved, playlists): `~/.local/share/youtui-rs/youtui.db`

## License

MIT
