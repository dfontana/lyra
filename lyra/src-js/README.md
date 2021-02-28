# Lyra

WIP

## Backlog (Ranked-ish)

- [ ] Ensure running as Daemon works
- [ ] Implement UI
  - [ ] Event bus between JS & Rust
    - [ ] Invoke JS to pass data (listen). [Tauri did this](https://github.com/tauri-apps/tauri/blob/dev/tauri/src/app/event.rs#L84)
    - [ ] Define a "Callback" that JS can evoke to emit an event to Rust (rust will need to listen). See wry.
  - [ ] Input Box
  - [ ] Result List
  - [ ] Result List Navigation
  - [ ] Trigger open event on 'Enter'
  - [ ] Styling (Text, Background)
  - [ ] Animations
  - [ ] Minify JS/CSS (may require bundler replacement)
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

- `yarn`/`npm` `install` to setup your env
- `yarn --cwd src-js build && cargo run -- -f` to build & launch immediately

You now have a statically compiled executable; inclusive of resources.
