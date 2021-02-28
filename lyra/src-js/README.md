# Lyra

WIP

## Backlog (Ranked-ish)

- [ ] Ensure running as Daemon works
- [ ] Implement UI
  - [ ] Event bus between JS & Rust
    - 1. Define an [initialization script](https://github.com/tauri-apps/tauri/blob/dev/tauri/src/app/utils.rs#L102) to set a [function on the window](https://github.com/tauri-apps/tauri/blob/dev/tauri/src/app/utils.rs#L165)
      - a. JS subscribes by [eventType](https://github.com/tauri-apps/tauri/blob/dev/tauri/src/app/utils.rs#L107) (you'll want a helper global function for this). Whenever this emit is invoked, the listener's [handler is invoked](https://github.com/tauri-apps/tauri/blob/dev/tauri/src/app/utils.rs#L128). You can create a Hook for this that ties into React State.
    - 2. Rust-Contacts-Js by [invoking this function](https://github.com/tauri-apps/tauri/blob/dev/tauri/src/app/event.rs#L84)
    - 3. Create a [callback function](https://github.com/tauri-apps/tauri/blob/dev/tauri/src/app/utils.rs#L183), which allows [JS to invoke it](https://github.com/tauri-apps/tauri/blob/dev/tauri/src/app/utils.rs#L87-L89)
      - JS passes a [Message object shaped like so](https://github.com/tauri-apps/tauri/blob/dev/tauri/src/app/utils.rs#L22)
      - Rust processes it, and determines which function name to invoke on [success/error using Dispatchers from wry + evaluate_script](https://github.com/tauri-apps/tauri/blob/dev/tauri/src/app/utils.rs#L305)
        - Ideally rust just sends back an event rather than invoking callbacks. I think we can do better.
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
