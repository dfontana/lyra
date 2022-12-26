use abi_stable::std_types::*;
use libloading::{Error, Library, Symbol};

use crate::PluginResult;

#[derive(Debug)]
pub struct Plugin {
  pub inner: Library,
}

impl Plugin {
  pub unsafe fn invoke_init(
    &self,
    plugin_data_dir: &RString,
    config: &RHashMap<RString, RString>,
  ) -> RResult<(), RString> {
    self.inner
            .get::<Symbol<unsafe extern "C" fn(&RString, &RHashMap<RString, RString>) -> RResult<(), RString>>>(
                b"plugin_init",
            )
            .unwrap()(plugin_data_dir, config)
  }

  pub unsafe fn invoke_query(&self, query: RStr) -> RVec<PluginResult> {
    self
      .inner
      .get::<Symbol<unsafe extern "C" fn(RStr) -> RVec<PluginResult>>>(b"plugin_query")
      .unwrap()(query)
  }
}

pub unsafe fn load_plugin(plugin_path: &str) -> Result<Plugin, Error> {
  let plugin = libloading::Library::new(plugin_path)?;

  // We don't use them right now, but we need this to check whether the necessary functions exist or not
  plugin
    .get::<Symbol<unsafe extern "C" fn(&RHashMap<RString, RString>) -> bool>>(b"plugin_init")?;
  plugin.get::<Symbol<unsafe extern "C" fn(RStr) -> RVec<PluginResult>>>(b"plugin_query")?;

  Ok(Plugin { inner: plugin })
}
