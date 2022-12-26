Based on https://github.com/mdgaziur/findex

# Plugins

## For users
- Download the plugin. If it's not compiled, then compile it in release mode using `cargo build --release`
- Copy `target/release/{platform}/*.so` into your `$LYRA_HOME/plugins/` folder
- Update your `$LYRA_HOME/config.toml` to add the new plugin, where the key is the name of the `.so` file you copied. For example, consider `calc.so`.

```toml
[[plugins]]
calc = { prefix = "=", config = {} }
```

| Property | Description |
|----------|---|
| prefix   | The prefix to invoke this plugin rather than default Lyra behavior |
| config   | Any configuration specific to the plugin can be forwarded here     |


Omission of a prefix (specifically setting `""`) will run the plugin as part of the main list of results without a prefix at all.

## For developers

- First make a `cdylib` library
- Add `lyra-plugin` and `abi_stable` as dependency
- Add the following code into `src/lib.rs`

```rust
use lyra_plugin::{lyra_plugin, PluginResult};
use abi_stable::std_types::*;

fn init(plug_data_dir: &RString, config: &RHashMap<RString, RString>) -> RResult<(), RString>  {
    // Set up your plugin using the config if necessary. You can store anything needed in the provided
    // plug_data_dir
    // Return RErr if something went wrong
    
    // Returning this indicates that the plugin initalization is successful
    ROk(())
}

fn query(query: RStr) -> RVec<PluginResult> {
    let mut result = vec![];
    
    /* Do stuff here */
    
    RVec::from(result)
}

define_plugin!(init, query);
```

### `init()`

Initialize your plugin here, ran once during startup. The argument provided is the configuration passed from the user in the form of KV pairs, which will all be String.

### `handle_query()`

Given user input specific to your plugin (having triggered via the assigned prefix), return a list of items determined in your plugin logic.
