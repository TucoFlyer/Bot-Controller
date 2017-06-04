#!/usr/bin/env python3
#
# Websocket server providing access to gimbal parameters.
# Runs standalone or as a library embedded in other utils.
#

from flyergimbal import GimbalPort
import threading
import functools
import asyncio
import websockets


class SocketServer:
    def __init__(self, gimbal, host='', port=8893):
        self.gimbal = gimbal
        self.host = host
        self.port = port

    def uri(self):
        return "ws://%s:%d" % (self.host, self.port)

    def serve(self):
        return websockets.serve(self.handle_client, self.host, self.port)

    @asyncio.coroutine
    def handle_client(self, websocket, path):
        try:
            while True:
                # Multiple commands can be batched into a websocket packet, one per line.
                # Results are sent immediately, not batched.
                results = []
                for line in (yield from websocket.recv()).split('\n'):
                    # Each line has space separated tokens
                    yield from self.handle_command(websocket, line.split())
        except websockets.exceptions.ConnectionClosed:
            return

    @asyncio.coroutine
    def handle_command(self, websocket, tokens):
        loop = asyncio.get_event_loop()

        if tokens[0] == 'set':
            fn = functools.partial(self.gimbal.setParam, target=int(tokens[1]), number=int(tokens[2]), value=int(tokens[3]))
            yield from loop.run_in_executor(None, fn)
            return

        if tokens[0] == 'get':
            fn = functools.partial(self.gimbal.getParam, target=int(tokens[1]), number=int(tokens[2]))
            r = yield from loop.run_in_executor(None, fn)
            yield from websocket.send('value %s %s %s' % (tokens[1], tokens[2], r))
            return

        if tokens[0] == 'motors':
            fn = functools.partial(self.gimbal.setMotors, int(tokens[1]))
            yield from loop.run_in_executor(None, fn)
            return

        raise ValueError("Unrecognized command %r" % tokens[0])


class ServerThread(threading.Thread):
    def __init__(self, gimbal, **kw):
        threading.Thread.__init__(self)
        self.gimbal = gimbal
        self.server_kwargs = kw
        self.setDaemon(True)

    def run(self):
        asyncio.set_event_loop(asyncio.new_event_loop())
        run_server(self.gimbal, **self.server_kwargs)

def run_server(gimbal, **kw):
    server = SocketServer(gimbal, **kw)
    loop = asyncio.get_event_loop()
    loop.run_until_complete(server.serve())
    print("Server running at %s" % server.uri())
    loop.run_forever()

def run_server_thread(gimbal, **kw):
    ServerThread(gimbal, **kw).start()

def main():
    run_server(GimbalPort())

if __name__ == '__main__':
    main()

