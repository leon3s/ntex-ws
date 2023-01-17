#[derive(Debug, Clone)]
pub struct WsNamespace {
  name: String,
}

impl WsNamespace {
  pub fn new(name: String) -> Self {
    Self { name }
  }
}
