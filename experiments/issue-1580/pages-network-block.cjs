// Probe preload: prevent all outbound sockets before loading the official bundle.
const net = require('node:net');
net.Socket.prototype.connect = function () {
  throw Object.assign(new Error('Offline Pages bundle probe blocks network'), {code:'ECONNREFUSED'});
};
globalThis.fetch = async function () {
  throw Object.assign(new Error('Offline Pages bundle probe blocks fetch'), {code:'ECONNREFUSED'});
};
