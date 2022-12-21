<h1 align="center">
  <br>
  Lyra
  <img src="https://raw.githubusercontent.com/dfontana/lyra/master/src-icons/app-icon-alt.png" alt="Lyra" width="25">
  <br>
</h1>

<h4 align="center">A cross-platform launcher built with <a href="https://tauri.app/" target="_blank">Tauri</a>.</h4>

<p align="center">
  <a href="#status">Status</a> •
  <a href="#roadmap">Roadmap</a> •
  <a href="#development">Development</a> •
  <a href="#license">License</a>
</p>

## Status

Nearly there! Final polishes on MacOs before we can start focusing back on Linux/Windows experience.

## Roadmap

#### MVP 1: Calc, Bookmarks, Templatables, App Launcher (MacOS only)

- [ ] Setting input for remainder of config items
- [ ] Lingering TODOs
- [ ] App styling finalizations (Notably some blue background peeking through the input)
- [ ] Release process for GH & homebrew tap
- [ ] AppIcon Cache ought to live in it's own config file, given the size makes config editing hard

#### MVP 2: Windows / Linux Support

- [ ] Selection improvements
  - Selected items should stay selected when search gets more specific. Example:
    - type "dash" (2+ results appear)
    - select dashlane
    - type "la"
    - cursor will jump to last item (web search) (due to current index capping behavior). Ideally it should stick to dashlane, as that was selected and still in the list
- [ ] Windows works
  - App launcher can find exes
  - AppIcon support
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

- [ ] Theming/Styling
  - [ ] Colors
  - [ ] (More) Font sizes (which means allowing more flexible option sizing)
  - [ ] Font Style
- [ ] Configuration KeyBindings: open/hide/up/down/confirm
- [ ] `Assets.car` support on MacOS for AppIcon resolving
- [ ] Settings page validation & cleanup
  - No spaces in shortnames for searchers, need links/labels/icons, templates validity, numbers are numbers etc

## Development

### Quick Reference

- `yarn install && yarn build` to init the repo
- `cargo tauri dev` for dev server
- `cargo tauri icon ../src-icon/app-icon.png` for icons (from the `src-tauri` dir)
- `cargo tauri build` to package app
  - Binary built: `src-tauri/target/release/[app name]`
  - Installers built: `src-tauri/target/release/bundle/`
- `yarn react-devtools` to debug

