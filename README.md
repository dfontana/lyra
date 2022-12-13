# Lyra

WIP - a cross-platform Spotlight/Alfred look-a-like

### Commands

- `yarn install && yarn build` to init the repo
- `cargo tauri dev` for dev server
- `cargo tauri build` to package app
  - Binary built: `src-tauri/target/release/[app name]`
  - Installers built: `src-tauri/target/release/bundle/`
- `yarn react-devtools` to debug

### Roadmap

#### MVP 1: Calc, Bookmarks, Templatables, App Launcher (MacOS only)

- [ ] Setting input for remainder of config items
- [ ] Lingering TODOs
- [ ] Get Lyra icons generated: https://tauri.studio/v1/guides/examples/icons/
  - [ ] App styling finalizations (Notably some blue background peeking through the input)
  - [ ] Repo name (no longer really needs to be called `-parent`)
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
