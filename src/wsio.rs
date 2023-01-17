use std::{
  time::{Instant, Duration},
  rc::Rc,
  cell::RefCell,
  future::ready,
};

use ntex::{
  web::{self, ws, HttpResponse, HttpRequest, Error},
  service::fn_factory_with_config,
  channel::oneshot,
  util::{select, Either, Bytes},
  time, Service, rt, fn_service,
};

use crate::{
  wsnamepace::WsNamespace,
  wsclient::{WsClient, self},
  wshandler::WsHandler,
};

#[derive(Debug, Clone)]
pub struct WsIo {
  heartbeat_interval: u64,
  client_timeout: u64,
  namespace: WsNamespace,
}

struct ConnState {
  /// Client must send ping at least once per 10 seconds (CLIENT_TIMEOUT),
  /// otherwise we drop connection.
  hb: Instant,
}

/// helper method that sends ping to client every heartbeat interval
async fn heartbeat(
  state: Rc<RefCell<ConnState>>,
  socket: wsclient::WsClient,
  mut rx: oneshot::Receiver<()>,
) {
  let heartbeat_interval = Duration::from_secs(5);
  let client_timeout = Duration::from_secs(10);
  loop {
    match select(Box::pin(time::sleep(heartbeat_interval)), &mut rx).await {
      Either::Left(_) => {
        // check client heartbeats
        if Instant::now().duration_since(state.borrow().hb) > client_timeout {
          // heartbeat timed out
          println!("Websocket Client heartbeat failed, disconnecting!");
          return;
        }

        // send ping
        if socket
          .sink
          .send(ws::Message::Ping(Bytes::new()))
          .await
          .is_err()
        {
          return;
        }
      }
      Either::Right(_) => {
        println!("Connection is dropped, stop heartbeat task");
        return;
      }
    }
  }
}

/// WebSockets service factory
async fn ws_service(
  req: HttpRequest,
  sink: ws::WsSink,
  wsio: WsIo,
) -> Result<
  impl Service<ws::Frame, Response = Option<ws::Message>, Error = std::io::Error>,
  web::Error,
> {
  let client = wsclient::WsClient::new("test".into(), sink);
  let state = Rc::new(RefCell::new(ConnState { hb: Instant::now() }));

  // disconnect notification
  let (_tx, rx) = oneshot::channel();

  // start heartbeat task
  rt::spawn(heartbeat(state.clone(), client, rx));

  // websockets handler service
  Ok(fn_service(move |frame| {
    println!("WS Frame: {:?}", frame);

    let item = match frame {
      ws::Frame::Ping(msg) => {
        state.borrow_mut().hb = Instant::now();
        Some(ws::Message::Pong(msg))
      }
      ws::Frame::Text(data) => {
        // Copy data into a new vector till we find a non-ascii digit
        let mut count = 0;
        let mut nonce_slice = Vec::new();
        while data[count].is_ascii_digit() {
          nonce_slice.push(data[count]);
          count += 1;
        }
        // convert nonce_slice into a string
        let Ok(nonce_str) = String::from_utf8(nonce_slice) else {
          return ready(Ok(Some(ws::Message::Close(None))));
        };
        // convert nonce_str into a u64
        let Ok(nonce) = nonce_str.parse::<u64>() else {
          return ready(Ok(Some(ws::Message::Close(None))));
        };
        println!("WS : {}", nonce);
        let Ok(payload) = serde_json::from_slice::<serde_json::Value>(&data[count..]) else {
          return ready(Ok(Some(ws::Message::Close(None))));
        };
        println!("WS Payload: {:?}", payload);
        let Some(args) = payload.as_array() else {
          return ready(Ok(Some(ws::Message::Close(None))));
        };
        let Some(event) = args.get(0) else {
          return ready(Ok(Some(ws::Message::Close(None))));
        };
        let Some(event) = event.as_str() else {
          return ready(Ok(Some(ws::Message::Close(None))));
        };
        let args = args[1..].to_vec();
        println!("WS Event: {}", event);
        println!("WS Args : {:?}", args);
        Some(ws::Message::Text("gg".into()))
      }
      ws::Frame::Binary(_binary) => None,
      ws::Frame::Close(reason) => Some(ws::Message::Close(reason)),
      _ => Some(ws::Message::Close(None)),
    };
    ready(Ok(item))
  }))
}

async fn ws_handler(
  req: HttpRequest,
  wsio: web::types::State<WsIo>,
) -> Result<HttpResponse, Error> {
  // do websocket handshake and start web sockets service
  println!(
    "ws_handler heartbeat_interval: {}, client_timeout: {}",
    wsio.heartbeat_interval, wsio.client_timeout
  );
  ws::start(
    req.to_owned(),
    fn_factory_with_config(move |sink| {
      ws_service(req.to_owned(), sink, wsio.get_ref().to_owned())
    }),
  )
  .await
}

impl Default for WsIo {
  fn default() -> Self {
    Self {
      client_timeout: 10,
      heartbeat_interval: 5,
      namespace: WsNamespace::new("/".into()),
    }
  }
}

impl WsIo {
  pub fn attach(&self) -> impl FnOnce(&mut web::ServiceConfig) {
    let this = self.clone();
    |cfg| {
      cfg.state(this);
      cfg.service(
        web::scope("/wsio/")
          .service(web::resource("/").route(web::get().to(ws_handler))),
      );
    }
  }

  pub fn on(&self, event: &str, handler: fn(WsClient)) {}
}
