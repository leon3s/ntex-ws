/// A websocket request
struct WsReq {
  /// The request
  req: WebRequest<Error>,
  /// The websocket handshake
  handshake: Handshake,
  /// The websocket io
  io: WsIo,
}
