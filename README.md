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

- Impl Opening apps
  - Union of "/System/Applications" and "/Applications"
  - Anything that ends in ".app"
  - Ideally we can add additional paths to check via settings
  - Ideally paths are cached & updated if dir was touched? (timestamp changed?)
- Polish:
  - Fix log rotation (need to clear old logs out from > N days ago)
  - Setting page validation & cleanup
    - No spaces in shortnames for searchers, need links/labels/icons, templates validity etc
  - Get icons generated: https://tauri.studio/v1/guides/examples/icons/
  - Optimize app size with:
    - https://tauri.studio/v1/guides/building/app-size#5-allowlist-config
    - https://tauri.studio/v1/guides/building/app-size#6-rust-build-time-optimizations
  - Launch UI
- Finish enabling linux tray support https://tauri.studio/v1/guides/examples/system-tray/#linux-setup

#### React

- Uses css-modules https://github.com/css-modules/css-modules

### Roadmap

#### MVP 1: Bookmarks (MacOS only)

- [ ] App 'branding' (icons, styling)
- [ ] Bookmarklets
  - [ ] Config file to add new bookmarklets; 
  -  [x] either a URL 
  -  [ ] or a parameterizable url
  - [x] Iconography support
  - [x] Opens in default browser

#### MVP 2: App launcher (MacOS only)

- [ ] fzf on app folder?
  - [ ] Only show results for platform specific file type (exe, app)

#### MVP 3: Windows / Linux Support

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
