<h1 align="center">
  <br>
  Lyra
  <img src="https://raw.githubusercontent.com/dfontana/lyra/master/lyra-ui/icons/app-icon-alt.png" alt="Lyra" width="25">
  <br>
</h1>

<h4 align="center">A cross-platform launcher built with <a href="https://github.com/emilk/egui/" target="_blank">egui</a>.</h4>

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
- [ ] App styling finalizations
- [ ] Release process for GH & homebrew tap

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
  - AppIcon support

#### MVP 3: File opener

- [ ] File launcher/finder (fzf/rg?)
  - [ ] Ability to search files and present top N options
  - [ ] Ability to open on enter
  - [ ] Configure paths to search

#### Bonus:

- [ ] Configuration KeyBindings: open/hide/up/down/confirm
- [ ] `Assets.car` support on MacOS for AppIcon resolving
- [ ] Allow configuration to reload without reboot (eg placing window)
- [ ] Parse window placement better... below is bad. v bad.
  ```
  window_placement = { XY = {0 = 420.0, 1 = 100.0} }
  ```
- [ ] Chatbot integration?
- [ ] Clipboard history/management?

## Development

### Quick Reference

- `cargo tauri icon ../src-icon/app-icon.png` for icons

