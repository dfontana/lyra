use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::to_value;
use sycamore::futures::spawn_local_scoped;
use sycamore::prelude::*;
use tracing::{error, info};
use wasm_bindgen::prelude::*;
use web_sys::{Event, KeyboardEvent};

#[wasm_bindgen]
extern "C" {
  #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "tauri"])]
  async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}
#[derive(Serialize, Deserialize)]
struct GreetArgs<'a> {
  name: &'a str,
}

fn main() {
  console_error_panic_hook::set_once();
  tracing_wasm::set_as_global_default();
  sycamore::render(App);
}

fn attach_keyup_handler() {
  let doc = web_sys::window().unwrap().document().unwrap();

  let cb = Closure::wrap(Box::new(|e: Event| {
    let event = e.dyn_ref::<KeyboardEvent>().unwrap_throw().to_owned();
    info!("Key hit! {}", event.key());
  }) as Box<dyn FnMut(_)>);

  let res = doc.add_event_listener_with_callback("keyup", cb.as_ref().unchecked_ref());
  if res.is_err() {
    error!("Failed to attach keyup listener");
    return;
  }

  cb.forget();
}

#[component]
fn App<G: Html>() -> View<G> {
  on_mount(|| {
    attach_keyup_handler();
  });

  let name = create_signal(String::new());
  let greet_msg = create_signal(String::new());

  // TODO: Recreate the close window behavior on 'ESC'
  //       Need to invoke tauri https://github.com/dfontana/lyra/blob/c9fd5af76f24a6ecc519563f15d32c2bd87d8a3f/src/app/app.js#L16-L21
  // Also need to figure out how to make greet work
  let greet = move |_| {
    spawn_local_scoped(async move {
      let new_msg = invoke(
        "greet",
        to_value(&GreetArgs { name: &name.take() }).unwrap(),
      )
      .await;
      info!("Greeting {}", &new_msg.as_string().unwrap());
      greet_msg.set(new_msg.as_string().unwrap());
    })
  };

  view! {
    div(class="row") {
                input(id="greet-input",bind:value=name,placeholder="Enter a name...")
                button(type="button",on:click=greet) {
                    "Greet"
                }
            }
            p {
                b {
                    (greet_msg.take())
                }
            }
  }
}
