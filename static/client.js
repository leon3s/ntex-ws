const RESERVED_EVENTS = ['connect', 'disconnect', 'error'];

class EventEmitter {
  constructor() {
    this.events = {};
  }
  on(event, listener) {
    if (typeof this.events[event] !== 'object') {
      this.events[event] = [];
    }
    this.events[event].push(listener);
    return () => this.removeListener(event, listener);
  }
  removeListener(event, listener) {
    if (typeof this.events[event] === 'object') {
      const idx = this.events[event].indexOf(listener);
      if (idx > -1) {
        this.events[event].splice(idx, 1);
      }
    }
  }
  emit(event, ...args) {
    if (typeof this.events[event] === 'object') {
      this.events[event].forEach(listener => listener.apply(this, args));
    }
  }
  once(event, listener) {
    const remove = this.on(event, (...args) => {
      remove();
      listener.apply(this, args);
    });
  }
};

class Client {
  static new(url) {
    return new Client(url);
  }

  constructor(url) {
    this.url = url;
    this.timeout = 10000;
    this.heartbeat = 5000;
    this.connected = false;
    this.emiter = new EventEmitter();
    this.conn = new WebSocket(url || (window.location.protocol == 'https:' && 'wss://' || 'ws://') + window.location.host + '/wsio/');
    this.conn.onopen = () => {
      this.connected = true;
      this.emiter.emit('connect');
    }
    this.conn.onclose = () => {
      this.connected = false;
      this.emiter.emit('disconnect');
    }
    this.conn.onmessage = (ev) => {
      console.log(ev);
    }
  }

  emit(event, ...args) {
    if (this.connected) {
      this.conn.send(JSON.stringify([event, ...args]));
    }
  }

  on(event, callback) {
    this.emiter.on(event, callback);
  }
}

function wsIo() {
  return Client.new();
}


$(function () {

  const socket = wsIo();

  socket.on('connect', () => {
    console.log('connected');
  });

  socket.on('message', (data) => {
    console.log(data);
  });

  var conn = null;
  function log(msg) {
    var control = $('#log');
    control.html(control.html() + msg + '<br/>');
    control.scrollTop(control.scrollTop() + 1000);
  }
  function connect() {
    disconnect();
    var wsUri = (window.location.protocol == 'https:' && 'wss://' || 'ws://') + window.location.host + '/wsio/';
    conn = new WebSocket(wsUri);
    log('Connecting...');
    conn.onopen = function () {
      log('Connected.');
      update_ui();
    };
    conn.onmessage = function (e) {
      log('Received: ' + e.data);
    };
    conn.onclose = function () {
      log('Disconnected.');
      conn = null;
      update_ui();
    };
  }
  function disconnect() {
    if (conn != null) {
      log('Disconnecting...');
      conn.close();
      conn = null;
      update_ui();
    }
  }
  function update_ui() {
    var msg = '';
    if (conn == null) {
      $('#status').text('disconnected');
      $('#connect').html('Connect');
    } else {
      $('#status').text('connected (' + conn.protocol + ')');
      $('#connect').html('Disconnect');
    }
  }
  $('#connect').click(function () {
    if (conn == null) {
      connect();
    } else {
      disconnect();
    }
    update_ui();
    return false;
  });
  $('#send').click(function () {
    var text = $('#text').val();
    log('Sending: ' + text);
    conn.send(12342 + JSON.stringify(['test', "dada"]));
    $('#text').val('').focus();
    return false;
  });
  $('#text').keyup(function (e) {
    if (e.keyCode === 13) {
      $('#send').click();
      return false;
    }
  });
});
