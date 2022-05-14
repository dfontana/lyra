# Lyra
### Commands

yarn tauri dev

yarn tauri build
  - Binary built: `src-tauri/target/release/[app name]`
  - Installers built: `src-tauri/target/release/bundle/`

### Notes

#### TODO

- Integrate into lyra-parent repo
- Config file support from `~/.config/lyra`
- Get icons generated: https://tauri.studio/v1/guides/examples/icons/
- Optimize app size with:
  - https://tauri.studio/v1/guides/building/app-size#5-allowlist-config
  - https://tauri.studio/v1/guides/building/app-size#6-rust-build-time-optimizations
- Finish enabling linux tray support https://tauri.studio/v1/guides/examples/system-tray/#linux-setup

#### React

- Uses css-modules https://github.com/css-modules/css-modules

#### Tauri

- May need to handle keeping app open on last window gone; if so: https://github.com/tauri-apps/tauri/discussions/2684

WIP - a cross-platform Spotlight/Alfred look-a-like.

## MVP 1: Basic Functionality on MacOS & Windows

- [x] Ensure running as SystemTray works
- [x] Implement UI
  - [x] Event bus between JS & Rust
    - [x] Basic implementation
    - [x] Handling delgated better
    - [x] Async support
  - [x] Input Box
  - [x] Result List
  - [x] Result List Navigation
  - [x] Trigger open event on 'Enter'
- [x] Breakup main/refactor
- [ ] Validate works on windows

## MVP 2: File opener (MacOS only)

- [ ] File launcher/finder (fzf/rg?)
  - [ ] Ability to search files and present top N options
  - [ ] Abililty to open on enter
  - [ ] Configure paths to search

## MVP 3: App launcher (MacOS only)

- [ ] fzf on app folder?
  - [ ] Only show results for platform specific file type (exe, app)

## MVP 4: Bookmarks (MacOS only)

- [ ] Bookmarklets
  - [ ] Config window to add new bookmarklets; prefixed
  - [ ] Autocomplete
  - [ ] Iconography support

## MVP 5: Configuration Window

- [ ] Specialize Configuration command to alter the config file
  - [ ] Window Size
  - [ ] Window Location
  - [ ] Paths for file search
  - [ ] Paths/Extensions for application search
  - [ ] Command to run when opening file
  - [ ] Command to run when opening application
  - [ ] Key Commands to open/hide
  - [ ] Key Commands to select up/down?

## MVP 5: App launcher, Bookmarks, File Finder on Windows

- [ ] Generalize to windows
- [ ] Get a better placeholder icon

## Stretch

- Linux support
  - Missing details on how this might work.
- UI Touchups
  - [ ] Styling (Text, Background)
  - [ ] Animations
  - [ ] Configurable style sheets
- [ ] Module System
  - [ ] Load modules from binary?
  - [ ] Refactor existing systems into modules
  - [ ] Document for others to develop

## Building

- Ensure you have `yarn` installed globally.
- Run a normal `cargo build`. This will handle initializing all the relevant JS pieces. See `build.rs` to explain this.

You now have a statically compiled executable; inclusive of resources.
