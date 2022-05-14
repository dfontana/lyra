#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Event {
  Show,
  Hide,
  Search { value: String },
  Submit { value: usize },
}

pub fn configure() -> Result<(EventLoop<Event>, SystemTray, WebView), wry::Error> {
  let _webview = WebViewBuilder::new(window)?
    .with_transparent(true)
    .with_rpc_handler(handler)
    .build()?;
}

pub fn handler(_proxy: &Window, mut req: RpcRequest) -> Option<RpcResponse> {
  let event = match Event::from(req.params.take()) {
    Ok(e) => e,
    Err(err) => {
      return Some(RpcResponse::new_error(
        req.id.take(),
        Some(Value::String(err.to_string())),
      ))
    }
  };

  let size = _proxy.inner_size();
  let new_size = LogicalSize::new(size.width, 38u32 + (18u32 * 2u32));
  _proxy.set_inner_size(new_size);

  match event {
    Event::Search { value } => Some(RpcResponse::new_result(
      req.id.take(),
      Some(json!([{
        "id": 0,
        "value": "First Result"
      },{
        "id": 1,
        "value": "Second Result"
      }])),
    )),
    _ => None,
  }
}

impl Event {
  pub fn from(params: Option<Value>) -> Result<Event, Error> {
    params
      .ok_or("Missing Args".to_string())
      .and_then(|v| serde_json::from_value::<Vec<Event>>(v).map_err(|e| format!("{}", e)))
      .and_then(|mut args| {
        if args.len() == 0 {
          Err("Missing Args".to_string())
        } else {
          Ok(args.swap_remove(0))
        }
      })
      .map_err(|e| Error::RpcEventFailure(e))
  }
}
