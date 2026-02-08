# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build Commands

This project uses `just` as a build system. Common commands:

```bash
just                    # Build release (default)
just build-debug        # Build debug
just build-release      # Build release
just check              # Run clippy with pedantic warnings
just run                # Run with RUST_BACKTRACE=full
just clean              # Clean build artifacts
sudo just install       # Install to system (/usr/bin, etc.)
just uninstall          # Remove installed files
```

Direct cargo commands also work:
```bash
cargo build --release
cargo clippy --all-features -- -W clippy::pedantic
```

## Runtime Dependencies

- **mpv**: Required for audio playback (spawned as subprocess)
- **alsa-utils**: Required for ALSA audio support

## Architecture Overview

This is a COSMIC Desktop panel applet that plays internet radio stations via the radio-browser.info API.

### Module Structure

- **main.rs**: Entry point, initializes i18n and runs the cosmic applet
- **app.rs**: Core application model implementing `cosmic::Application` trait
  - Manages popup window state, search, playback, and favorites
  - Uses Elm architecture (Model-View-Update pattern)
- **api.rs**: Radio Browser API client
  - `Station` struct with serde serialization
  - `search_stations()` async function with server redundancy (7 mirrors)
- **audio.rs**: `AudioManager` wrapping mpv subprocess
  - Spawns mpv with `--no-video --volume-max=200 --af=lavfi=[dynaudnorm]`
  - Process managed via `Arc<Mutex<Option<Child>>>`
- **config.rs**: Persistent configuration via `cosmic_config`
  - `Config` struct with favorites list and volume (versioned, currently v9)
- **i18n.rs**: Fluent-based localization setup

### Key Patterns

**libcosmic Applet Pattern**: The app implements `cosmic::Application` with:
- `view()` returns the panel icon button
- `view_window()` returns the popup content
- `update()` handles all `Message` variants via match

**Async Search**: Search uses `Task::perform()` to run async API calls, returning results via `Message::SearchCompleted`.

**Config Persistence**: Uses `CosmicConfigEntry` derive macro with version tracking. Config stored at standard cosmic config location.

### Application ID

`com.marcos.RadioApplet` - used for config storage, desktop file, and resources.

## Adding Translations

1. Create new directory: `i18n/{locale}/`
2. Add `cosmic_radio_applet.ftl` with translations
3. Use `fl!("message-id")` macro in code

## NixOS Installation

### Using the Flake

```bash
# Build the package
nix build .#cosmic-radio-applet

# Enter development shell
nix develop

# Check flake
nix flake check
```

### NixOS System Module

Add to your NixOS configuration:

```nix
{
  inputs.cosmic-radio-applet.url = "github:marcossl10/cosmic-radio-applet";

  outputs = { self, nixpkgs, cosmic-radio-applet, ... }: {
    nixosConfigurations.yourhost = nixpkgs.lib.nixosSystem {
      modules = [
        cosmic-radio-applet.nixosModules.cosmic-radio-applet
        {
          programs.cosmic-radio-applet = {
            enable = true;
            autostart = true;  # Add to XDG autostart
            settings = {
              volume = 75;
              favorites = [
                {
                  stationuuid = "96202c39-0601-11e8-ae97-52543be04c81";
                  name = "SomaFM - Groove Salad";
                  url_resolved = "https://ice1.somafm.com/groovesalad-128-mp3";
                }
              ];
            };
          };
        }
      ];
    };
  };
}
```

### Home Manager Module

Add to your Home Manager configuration:

```nix
{
  imports = [ cosmic-radio-applet.homeManagerModules.cosmic-radio-applet ];

  programs.cosmic-radio-applet = {
    enable = true;
    autostart = true;
    settings = {
      volume = 50;
      favorites = [
        {
          stationuuid = "9617a958-0601-11e8-ae97-52543be04c81";
          name = "Jazz24";
          url_resolved = "https://live.wostreaming.net/direct/ppm-jazz24aac-ibc1";
        }
      ];
    };
  };
}
```

### Module Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `enable` | bool | `false` | Enable the applet |
| `package` | package | flake default | The package to use |
| `autostart` | bool | `true` | Auto-start with COSMIC |
| `settings.volume` | int (0-100) | `50` | Default volume |
| `settings.favorites` | list of stations | `[]` | Pre-configured stations |

### Station Schema

Each station in `settings.favorites` requires:
- `stationuuid`: Unique ID from radio-browser.info
- `name`: Display name
- `url_resolved`: Direct stream URL

Optional fields: `url`, `homepage`, `favicon`, `tags`, `country`, `language`
