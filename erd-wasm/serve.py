#!/usr/bin/env python3
# from https://gist.github.com/prideout/09af26cef84eef3e06a1e3f20a499a48

import http.server
import socketserver

PORT = 8000

Handler = http.server.SimpleHTTPRequestHandler
Handler.extensions_map.update({
    '.wasm': 'application/wasm',
})

socketserver.TCPServer.allow_reuse_address = True
with socketserver.TCPServer(("", PORT), Handler) as httpd:
    httpd.allow_reuse_address = True
    print("serving at port", PORT)
    httpd.serve_forever()
