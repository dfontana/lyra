# Lyra

WIP - a cross-platform Spotlight/Alfred look-a-like

### Commands

- `yarn install && yarn build` to init the repo
- `yarn tauri dev` for dev server
- `yarn tauri build` to package app
  - Binary built: `src-tauri/target/release/[app name]`
  - Installers built: `src-tauri/target/release/bundle/`
- `yarn react-devtools` to debug

### Notes

#### TODO

- Polish:
  - Fix all the clippy warns
  - Fix log rotation (need to clear old logs out from > N days ago)
  - Setting page validation & cleanup
    - No spaces in shortnames for searchers, need links/labels/icons, templates validity etc
  - Get icons generated: https://tauri.studio/v1/guides/examples/icons/
  - Optimize app size with:
    - https://tauri.studio/v1/guides/building/app-size#5-allowlist-config
    - https://tauri.studio/v1/guides/building/app-size#6-rust-build-time-optimizations

#### React

- Uses css-modules https://github.com/css-modules/css-modules

### Roadmap

#### MVP 1: Bookmarks & App Launcher (MacOS only)

- [ ] App 'branding' (icons, styling)

#### MVP 2: Windows / Linux Support

- [ ] Windows works
  - App launcher can find exes
  - AppIcon support
  - Settings can configure `app_paths` and `app_extensions`
- [ ] Linux works
  - App launcher can find... what now? .Desktop files?
  - Finish enabling linux tray support https://tauri.studio/v1/guides/examples/system-tray/#linux-setup
  - AppIcon support

#### MVP 3: File opener

- [ ] File launcher/finder (fzf/rg?)
  - [ ] Ability to search files and present top N options
  - [ ] Abililty to open on enter
  - [ ] Configure paths to search

#### Bonus:

- [ ] Configuration Styling
  - [ ] Window Size, location
  - [ ] Colors (theming)
  - [ ] Font sizes
- [ ] Configuration KeyBindings: open/hide/up/down/confirm
- `Assets.car` support on MacOS
