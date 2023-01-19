use ntex::web;

use ntex_files as fs;

mod wsio;
mod wsclient;
mod wshandler;
mod wsnamepace;

async fn on_message(
  test: web::types::State<String>,
  test2: web::types::State<u64>,
) -> &'static str {
  "gg"
}

#[ntex::main]
async fn main() -> std::io::Result<()> {
  let mut wsio = wsio::WsIo::default();

  wsio.on_connection(|mut socket| {
    println!("new connection {}", socket.id);
    // socket.on("message", |data| {})
  });

  let srv = ntex::web::HttpServer::new(move || {
    ntex::web::App::new()
      .configure(wsio.attach())
      .service(fs::Files::new("/", "./static/").index_file("index.html"))
  });
  srv.bind("0.0.0.0:8080")?.run().await?;
  Ok(())
}
