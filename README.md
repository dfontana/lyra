# Lyra

WIP - a cross-platform Spotlight/Alfred look-a-like.

## MVP 1: Basic Functionality on MacOS

- [ ] Ensure running as Daemon works
- [ ] Implement UI
  - [x] Event bus between JS & Rust
    - [x] Basic implementation
    - [x] Handling delgated better
    - [x] Async support
  - [x] Input Box
  - [x] Result List
  - [x] Result List Navigation
  - [x] Trigger open event on 'Enter'
- [ ] File launcher/finder (fzf/rg?)
  - [ ] Ability to search files and present top N options
  - [ ] Abililty to open on enter
  - [ ] Configure paths to search
- [x] Breakup main/refactor
- [ ] Debug Logging

## MVP 2: App launcher

- [ ] fzf on app folder?
  - [ ] Only show results for platform specific file type (exe, app)

## MVP 3: Bookmarks

- [ ] Bookmarklets
  - [ ] Config window to add new bookmarklets; prefixed
  - [ ] Autocomplete
  - [ ] Iconography support

## MVP 3: Windows

- [ ] `keys` needs to support windows
- [ ] File search needs to know what drives/paths to scan
- [ ] App Launching needs to know where to find programs
- [ ] Build & test

## Stretch

- Linux support
  - Missing details on how this might work.
- UI Touchups
  - [ ] Styling (Text, Background)
  - [ ] Animations
- [ ] Configuration
  - [ ] Window Size
  - [ ] Window Location
  - [ ] Key Commands to open/hide
  - [ ] Key Commands to select up/down
  - [ ] Styles
- [ ] Module System
  - [ ] Load modules from binary?
  - [ ] Refactor existing systems into modules
  - [ ] Document for others to develop

## Building

- Ensure you have `yarn` installed globally.
- Run a normal `cargo build`. This will handle initializing all the relevant JS pieces. See `build.rs` to explain this.

You now have a statically compiled executable; inclusive of resources.
