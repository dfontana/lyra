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

- Fix the TODO bugs in bookmark management
- Get a very basic launcher functionality implemented (perhaps return all items without filtering)
- Use the open crate to open the bookmarks
- Impl remaining functionality for launching/searching bookmarks
- Polish:
  - Launch UI
  - Get icons generated: https://tauri.studio/v1/guides/examples/icons/
  - Optimize app size with:
    - https://tauri.studio/v1/guides/building/app-size#5-allowlist-config
    - https://tauri.studio/v1/guides/building/app-size#6-rust-build-time-optimizations
  - Fix the debug script (for dev tools) hardcoded in index.html. Ideally this is bound in when running dev.

#### React

- Uses css-modules https://github.com/css-modules/css-modules

#### Tauri

- May need to handle keeping app open on last window gone; if so: https://github.com/tauri-apps/tauri/discussions/2684
- Finish enabling linux tray support https://tauri.studio/v1/guides/examples/system-tray/#linux-setup

### Roadmap

#### MVP 1: Bookmarks (MacOS only)

- [ ] App 'branding' (icons, styling)
- [ ] Bookmarklets
  - [ ] Config file to add new bookmarklets; either a URL or a parameterizable url
  - [ ] Autocomplete
  - [ ] Iconography support
  - [ ] Opens in default browser

#### MVP 2: App launcher (MacOS only)

- [ ] fzf on app folder?
  - [ ] Only show results for platform specific file type (exe, app)

#### MVP 3: Windows / Linux Suppport

- [ ] Windows works
- [ ] Linux works

#### MVP 4: Configuration Window

- [ ] Configuration window from tray menu to alter the config file
  - [ ] Paths/Extensions for application search
  - [ ] Bookmarklet management - name, (templatable) link, icon
  - [ ] Command to run when opening application
  - [ ] KeyBindings: open/hide/up/down/confirm

#### MVP 5: File opener

- [ ] File launcher/finder (fzf/rg?)
  - [ ] Ability to search files and present top N options
  - [ ] Abililty to open on enter
  - [ ] Configure paths to search

#### Bonus:

- [ ] Styling
  - [ ] Window Size, location
  - [ ] Colors (theming)
  - [ ] Font sizes
