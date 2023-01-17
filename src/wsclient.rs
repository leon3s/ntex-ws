use std::rc::Rc;

use ntex::{
  web::{DefaultError, ErrorRenderer, FromRequest},
  ws,
  util::HashMap,
};

use crate::wshandler::{WsHandlerFn, WsHandler, HandlerWrapper};

#[derive(Clone)]
pub struct WsClient<Err: ErrorRenderer = DefaultError> {
  pub id: String,
  pub sink: ws::WsSink,
  handlers: HashMap<String, Rc<dyn WsHandlerFn<Err>>>,
}

impl<Err: ErrorRenderer> WsClient<Err> {
  pub fn new(id: String, sink: ws::WsSink) -> Self {
    Self {
      id,
      sink,
      handlers: HashMap::default(),
    }
  }

  pub fn on<H, Args>(&mut self, event: &str, hander: H)
  where
    H: WsHandler<Args, Err> + 'static,
    Args: FromRequest<Err> + 'static,
    Args::Error: Into<Err::Container>,
  {
    let handler: Rc<dyn WsHandlerFn<Err>> =
      Rc::new(HandlerWrapper::new(hander));
    self.handlers.insert(event.into(), handler);
  }

  pub async fn emit(&self, event: &str, data: &str) {
    self.sink.send(ntex::ws::Message::Text(data.into())).await;
  }
}
