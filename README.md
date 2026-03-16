[![Codeberg](https://img.shields.io/badge/Codeberg-main_repo-2185d0?logo=codeberg&logoColor=white)](https://codeberg.org/vatia/torrentui)
[![Built With Ratatui](https://img.shields.io/badge/Built_With_Ratatui-000?logo=ratatui&logoColor=fff)](https://ratatui.rs/)

# TorrenTUI
A minimal TUI client written in Rust for the [rqbit](https://github.com/ikatson/rqbit) torrent client. Manage torrents from your terminal.

## Features
* View list of torrents, their status and progress.
* Add torrents via magnet links.
* Pause, resume, delete torrents.
* Play movies or music from a torrent using an external player.
* Support for multiple connectors (multiple rqbit instances).
* Customizable key bindings (with sequence support and which‑key‑like hints).
* Configuration via TOML file or Nix (flake).
* Logging to journald (if available) or to a file.

## Installation

### From source
```bash
git clone https://codeberg.org/vatia/torrentui.git
cd torrentui
cargo build --release
# Binary: target/release/tt
```

### Nix (flake)
```bash
nix profile add git+https://codeberg.org/vatia/torrentui.git
```
Or add to your flake.nix:
```nix
inputs = {
  torrentui.url = "git+https://codeberg.org/vatia/torrentui.git";
}
```
and to your home.nix (for home‑manager users):
```nix
{
  imports = [ torrentui.nixosModules.torrentui ];
  programs.torrentui = {
    enable = true;
    # optional
    settings.keybindings.torrent-list = {
      "<k>" = "Up";
      "<j>" = "Down";
      "<space><o>" = "Play";
      "<ctrl-d>".description = "Delete torrent";
      "<ctrl-d><f>" = "Forget";
      "<ctrl-d><d>" = {
        action = "Delete";
        description = "Forget about the torrent, remove the files";
      };
    };
  };
}
```
## Usage
```bash
tt
```
The full list of key bindings is always available by pressing "Help" (? by default)

## Creating a magnet link from a .torrent file
The current version only supports adding torrents via magnet links. If you have a .torrent file, you can convert it to a magnet link using [imdl](https://github.com/casey/intermodal):

```bash
# Install imdl
cargo install imdl

# Generate magnet link
imdl torrent link file.torrent
```

## Configuration
The application uses a built‑in default configuration. If you wish to override any settings, create a file ~/.config/torrentui/config.toml.

<details>
<summary>Default configuration</summary>

```toml
notification_timeout_millis = 100
player_cmd = "setsid mpv"
auto_show_help = true
auto_insert_magnet = true

[keybindings.torrent-list]
"<k>" = "Up"
"<j>" = "Down"
"<up>" = "Up"
"<down>" = "Down"
"<g><g>" = "GotoTop"
"<g><e>" = "GotoBottom"
"<space><space>" = "PauseToggle"
"<space><o>" = "Play"
"<space><a>" = "AddMagnet"
"<ctrl-d>" = { description = "Delete torrent" }
"<ctrl-d><f>" = { action = "Forget", description = "Forget about the torrent, keep the files" }
"<ctrl-d><d>" = { action = "Delete", description = "Forget about the torrent, remove the files" }
"<esc>" = "Escape"
"<?>" = "Help"
"<q>" = "Quit"

[keybindings.add-magnet.input]
"<k>" = "Up"
"<j>" = "Down"
"<up>" = "Up"
"<down>" = "Down"
"<backspace>" = { action = "Backspace", description = "Clear" }
"<ctrl-v>" = "Paste"
"<enter>" = "Enter"
"<esc>" = "Escape"
"<?>" = "Help"

[keybindings.add-magnet.connectors]
"<k>" = "Up"
"<j>" = "Down"
"<up>" = "Up"
"<down>" = "Down"
"<space>" = { action = "Toggle", description = "Toggle selected" }
"<enter>" = "Send"
"<esc>" = "Escape"
"<tab>" = { action = "Switch", description = "Return to input magnet" }
"<?>" = "Help"

[connectors.localhost]
kind = "rqbit"
url = "http://localhost:3030"
api_version = "v8"
update_interval_secs = 1
selected_by_default = true

[styles.active]
upload = "black on yellow"
download = "black on blue"

[styles.paused]
default = "dark gray on rgb:0,0,0"

[styles.notification]
info = "blue on rgb:0,0,0"
error = "red on rgb:0,0,0"

[styles.which-key]
key = "yellow on rgb:0,0,0"
description = "blue on rgb:0,0,0"
next = "magenta on rgb:0,0,0"
divider = "rgb:80,80,80 on rgb:0,0,0"
default = "blue on rgb:0,0,0"

[styles.default]
default = "white on rgb:0,0,0"
highlight = "yellow on black"
dividers = "white on rgb:0,0,0"

[styles.add-magnet.input]
input_highlight = "black on white"
selected_connector = "gray on rgb:30,30,30"
insert_mode = "blue on rgb:30,30,30"
border = "blue on rgb:30,30,30"
default = "gray on rgb:30,30,30"

[styles.add-magnet.connectors]
input = "dark gray on rgb:30,30,30"
input_highlight = "dark gray on rgb:30,30,30"
selected_connector = "yellow on rgb:30,30,30"
connectors_highlight = "black on white"
border = "blue on rgb:30,30,30"
default = "blue on rgb:30,30,30"
```
</details>


For Nix users, settings can be defined via programs.torrentui.settings

## Customizing key bindings

Default key bindings can be overridden or disabled in the configuration file.
For example, to disable the default `q` quit binding and replace it with a capital `Q`:

```toml
[keybindings.torrent-list]
"<q>" = "NoOp"   # disables the default quit action
"<Q>" = "Quit"   # adds a new quit binding
```
Any action name from the default set can be used. Specifying "NoOp" for a key makes that key do nothing.

## Requirements

* rqbit installed and running (version >= 8.1.1 recommended).
* A terminal that supports 256 colors.

## Logging
If systemd is present, logs are written to the journald (view with journalctl --user -t torrentui).
If journald is not available, logs are stored in ~/.local/share/torrentui/torrentui.log.

## TODO
- [ ] Direct `.torrent` file upload (instead of only magnet links)
- [ ] Support for rqbit API v9 (currently only v8 is stable)
- [ ] Detailed torrent information view (peers, trackers, files)
- [ ] File selection for selective downloads

## Motivation
This project started from a desire to have a lightweight and responsive interface for managing torrents — something fast to open and close, easy to control without leaving the keyboard, and quick enough to play a movie or music from a torrent with just a couple of keystrokes. It was also my first real project in Rust — a chance to learn the language by building something genuinely useful.

## License
This project is licensed under the [MIT License](LICENSE).

