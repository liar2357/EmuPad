[日本語](README.jp.md)

A tool for using a touchscreen as a virtual trackpad on Linux.

# Overview

EmuPad is a virtual trackpad for Linux environments.
It directly reads touchscreen input and sends it to the OS through a virtual mouse using uinput.
You can operate the mouse using only your fingers.

> **_This project has not reached version 1.0 yet._**
> **_Unstable behavior and breaking changes may occur._**

# Tech Stack

- Language: Rust
- Configuration: TOML

# Supported Environment

A touchscreen that can be operated with your fingers, such as on a 2-in-1 laptop, is required.
It is expected to work on Linux distributions in general.

## Tested Environment

| Distribution | DE/WM |
| ------------ | ----- |
| NixOS 26.11  | Hyprland 0.56.2 |

Support for other environments is planned for the future.

# Installation

Currently, only installation via Nix (flake) is supported.
Specify the URL in `flake.nix` and install EmuPad from the package definition.

### flake.nix

```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    emu-pad.url = "github:liar2357/EmuPad";
  };

  outputs = { self, nixpkgs, emu-pad, ... }:
    {
      nixosConfigurations.my-pc = nixpkgs.lib.nixosSystem {
        system = "x86_64-linux";

        modules = [
          ./configuration.nix
        ];
      };
    };
}
```

### configuration.nix

```nix
{ pkgs, emu-pad, ... }:

{
  environment.systemPackages = [
    emu-pad.packages.${pkgs.system}.default
  ];
}
```

See also: [Actual usage example](https://github.com/liar2357/Dotfiles/blob/main/flake.nix)

For other environments, please clone the repository and build it manually.
See the following files for details.

- [package.nix (Nix package definition)](./package.nix)
- [rust.nix (Rust development dependencies)](https://github.com/liar2357/nix-dev-common/blob/main/rust.nix)

# Usage

This project includes two binaries: `emu-pad` and `emu-padctl`.

- `emu-pad`: The virtual trackpad itself
- `emu-padctl`: A CLI tool for controlling temporary settings and other runtime behavior

## emu-pad

Running the `emu-pad` command starts the CLI tool, begins monitoring the touchscreen, and creates a virtual mouse.
Fine-grained behavior can be controlled through the configuration file (`$HOME/.config/emu-pad/config.toml`).
If a setting is not specified, the default value is used.

> EmuPad directly monitors Linux device files, so access to `/dev/input` is required.
> It also uses `uinput`, so access to `/dev/uinput` is required.
> The method for configuring these permissions varies depending on the distribution.

### Basic Operations

| Operation | Input |
| --------- | ----- |
| Two-finger swipe | Move the mouse cursor |
| Two-finger tap | Left click |
| Two-finger long press | Right click |
| Three-finger swipe | Scroll (mouse wheel) |
| Three-finger tap | Middle click |

### Configuration Options

| Option | Description | Default |
| ------ | ----------- | ------- |
| `layout` | Keyboard layout | `"JIS-QWERTY"` |
| `hold_mode` | Modifier key behavior | `"None"` |
| `default_monitor` | Monitor to display | `"auto"` |
| `default_ui_view` | UI visibility on startup | `true` |
| `default_ui_place` | UI position | `"Lower"` |
| `ui_height` | UI height | `"30%"` |
| `ui_width` | UI width | `"100%"` |

### Configuration File Example

```toml
# $HOME/.config/emu-pad/config.toml

# Multiple configurations can be defined as profiles
[[configs]]

# Device section
# Settings for the touchscreen to capture
[device]

# selector
# Specifies the device to capture.
# "path:<device file path>" -> Specify by path
# (Not recommended because the path may change each time the system starts)
# "name:<device display name>" -> Specify by name
# "vp:<vendor ID>/<product ID>" -> Specify by ID
# "auto" -> Automatic selection
# (Scans /dev/input and uses the first touchscreen found)
selector = "auto"

# polling_rate_ms
# Specifies how often touchscreen input is detected, in milliseconds (minimum: 1).
polling_rate_ms = 3

# Gesture section
# Settings for interpreting touch input
[gesture]

# tap_threshold_ms
# Specifies the time threshold for detecting a tap, in milliseconds.
# Inputs at or below this threshold are treated as taps.
tap_threshold_ms = 250

# long_press_threshold_ms
# Specifies the time threshold for detecting a long press, in milliseconds.
# Inputs at or above this threshold are treated as long presses.
long_press_threshold_ms = 500

# movement_threshold
# Specifies the distance threshold for detecting movement, in pixels.
# Movement at or above this threshold is detected as movement.
# Note that this value must be specified as a decimal number.
movement_threshold = 15.0

# release_grace_threshold_ms
# Specifies the time threshold for detecting the end of a gesture, in milliseconds.
# If this amount of time passes, the current gesture is considered finished.
release_grace_threshold_ms = 100

# default_direction
# Specifies the default gesture direction.
# See "Gesture Recognition Direction" below for details.
default_direction = 0

# move_sensitivity_percent
# Specifies the mouse cursor movement multiplier as a percentage (minimum: 1).
move_sensitivity_percent = 80

# scroll_sensitivity_percent
# Specifies the mouse wheel movement multiplier as a percentage (minimum: 1).
scroll_sensitivity_percent = 50
```

### Command-Line Options

```bash
# Display version information
emu-pad -V
emu-pad --version

# Display available options and usage
emu-pad -h
emu-pad --help

# Specify an arbitrary file path as the configuration file
emu-pad -c <path>
emu-pad --config <path>

# Set the log level
emu-pad        # (default) error and above
emu-pad -v     # warn and above
emu-pad -vv    # info and above
emu-pad -vvv   # trace and above
```

### Hot Reload

Changing the configuration file automatically regenerates the UI.

## emu-padctl

### Command List

| Name | Description |
| ---- | ----------- |
| `set_direction <number>` | Change the direction used for gesture recognition |
| `switch_profile <index>` | Switch between defined profiles (zero-based / loops) |
| `lower_ui_place` | Move the UI position to the bottom |
| `shutdown_app` | Shut down the application |

### Gesture Recognition Direction

| Number | Direction |
| ------ | --------- |
| 0 | Default orientation (standard touchscreen orientation) |
| 1 | 90° rotation |
| 2 | 180° rotation |
| 3 | 270° rotation |

### Examples

```bash
# Change the gesture recognition direction to portrait orientation
emu-padctl set_direction 3

# Shut down the application
emu-padctl shutdown_app
```

### Command-Line Options

```bash
# Display version information
emu-padctl -V
emu-padctl --version

# Display usage information
emu-padctl -h
emu-padctl --help
```

# Trivia

**EmuPad**

Short for "Emulated Trackpad".

Pronounced "Emu-pad".

# License

[MIT License](./LICENCE)
