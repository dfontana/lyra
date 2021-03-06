# Lyra

WIP

## Backlog (Ranked-ish)

- [ ] Ensure running as Daemon works
- [ ] Implement UI
  - [ ] Event bus between JS & Rust
    - [x] Basic implementation
    - [ ] Handling delgated better
    - [ ] Async support
  - [ ] Input Box
  - [ ] Result List
  - [ ] Result List Navigation
  - [ ] Trigger open event on 'Enter'
  - [ ] Styling (Text, Background)
  - [ ] Animations
- [ ] Breakup main/refactor
- [ ] Debug Logging
- [ ] Bookmarklets
  - [ ] Config window to add new bookmarklets; prefixed
  - [ ] Autocomplete
  - [ ] Iconography support
- [ ] Windows support
  - [ ] `keys` needs to support windows
  - [ ] Build & test
- [ ] App launcher (fzf on app directory?)
- [ ] Configuration
  - [ ] Window Size
  - [ ] Window Location
  - [ ] Key Commands to open/hide
  - [ ] Key Commands to select up/down
  - [ ] Styles
- [ ] File launcher/finder (fzf/rg?)
- [ ] Module System
  - [ ] Load modules from binary?
  - [ ] Refactor existing systems into modules
  - [ ] Document for others to develop

## Building

- Ensure you have `yarn` installed globally.
- Run a normal `cargo build`. This will handle initializing all the relevant JS pieces. See `build.rs` to explain this.

You now have a statically compiled executable; inclusive of resources.
