#[cfg(feature = "plugin_internals")]
pub mod plugin_internal;

use abi_stable::std_types::*;

#[derive(Clone, Debug)]
pub struct PluginResult {
    pub matchable: RString,
    pub data: RHashMap<RString, RString>,
}

#[macro_export]
macro_rules! lyra_plugin {
    ($init_function:ident, $query_handler:ident) => {
        #[no_mangle]
        extern "C" fn plugin_init(
            plugin_data_dir: &RString,
            config: &RHashMap<RString, RString>,
        ) -> RResult<(), RString> {
            $init_function(plugin_data_dir, config)
        }

        #[no_mangle]
        extern "C" fn plugin_query(query: RStr) -> RVec<PluginResult> {
            $query_handler(query)
        }
    };
}
