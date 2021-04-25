use crate::error::Error;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use wry::{RpcRequest, RpcResponse, WindowProxy};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
enum Event {
  Search { value: String },
  Submit { value: usize },
}

pub fn handler(_proxy: WindowProxy, mut req: RpcRequest) -> Option<RpcResponse> {
  let event = match Event::from(req.params.take()) {
    Ok(e) => e,
    Err(err) => {
      return Some(RpcResponse::new_error(
        req.id.take(),
        Some(Value::String(err.to_string())),
      ))
    }
  };

  match _proxy.set_height(38f64 + (18f64 * 2f64)) {
    Ok(_) => (),
    Err(e) => println!("{}", e),
  };

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
