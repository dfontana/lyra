use crate::error::Error;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use wry::{RpcRequest, RpcResponse, WindowProxy};
#[derive(Debug, Serialize, Deserialize)]
struct Params {
  msg: String,
}

enum Event {
  Ping,
  Break,
  Data(Params),
}

pub fn handler(_proxy: WindowProxy, mut req: RpcRequest) -> Option<RpcResponse> {
  let event = match Event::from(req.method.as_ref(), req.params.take()) {
    Ok(e) => e,
    Err(err) => {
      return Some(RpcResponse::new_error(
        req.id.take(),
        Some(Value::String(err.to_string())),
      ))
    }
  };

  match event {
    Event::Ping => Some(RpcResponse::new_result(
      req.id.take(),
      Some(Value::String("pong".to_string())),
    )),
    Event::Break => Some(RpcResponse::new_error(
      req.id.take(),
      Some(Value::String("Failed".to_string())),
    )),
    Event::Data(val) => Some(RpcResponse::new_result(
      req.id.take(),
      Some(Value::String(val.msg)),
    )),
  }
}

fn pull_args<T>(params: Option<Value>) -> Result<T, String>
where
  T: DeserializeOwned,
{
  params
    .ok_or("Missing Args".to_string())
    .and_then(|v| serde_json::from_value::<Vec<T>>(v).or(Err("Failed to parse Args".to_string())))
    .and_then(|mut args| {
      if args.len() == 0 {
        Err("Missing Args".to_string())
      } else {
        Ok(args.swap_remove(0))
      }
    })
}

impl Event {
  pub fn from(method: &str, params: Option<Value>) -> Result<Event, Error> {
    match method {
      "ping" => Ok(Event::Ping),
      "break" => Ok(Event::Break),
      "data" => pull_args::<Params>(params)
        .map_err(|e| Error::RpcEventFailure(method.to_string(), e))
        .map(|args| Event::Data(args)),
      _ => Err(Error::RpcEventFailure(
        method.to_string(),
        format!("{:?}", params),
      )),
    }
  }
}
