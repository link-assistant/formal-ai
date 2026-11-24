"""Verify the loopback registry fixture without compiling or executing Rust."""
import ast
import http.client
import http.server
import json
from pathlib import Path
import socket
import threading


def verify_controls():
    path = Path(__file__).resolve().parent / 'release-registry/cli_regressions.py'
    tree = ast.parse(path.read_text())
    owner = next(node for node in tree.body if isinstance(node, ast.ClassDef))
    setup = next(node for node in owner.body if isinstance(node, ast.FunctionDef) and node.name == 'setUpClass')
    node = next(node for node in setup.body if isinstance(node, ast.ClassDef))
    fixture = type('Fixture', (), {'requests': [], 'mode': '503', 'scope': '',
                                   'maximum': '1.2.3', 'sparse_reply': None})
    scope = {'cls': fixture, 'http': http, 'json': json, 'socket': socket}
    exec(compile(ast.Module(body=[node], type_ignores=[]), str(path), 'exec'), scope)
    server = http.server.ThreadingHTTPServer(('127.0.0.1', 0), scope['Registry'])
    thread = threading.Thread(target=lambda: server.serve_forever(poll_interval=.05))
    thread.start()
    cases = []
    try:
        for mode, override, route, expected in [
            ('503', (200, '{"name":"fixture","vers":"1.2.3"}'), '/_sparse/fi/xt/fixture', 200),
            ('503', None, '/api/v1/crates/fixture/1.2.3', 503),
            ('success', None, '/_sparse/fi/xt/fixture', 200),
            ('success', None, '/api/v1/crates/fixture/1.2.3', 200),
            ('wrong-identity', None, '/_sparse/fi/xt/fixture', 200),
        ]:
            fixture.mode, fixture.sparse_reply = mode, override
            connection = http.client.HTTPConnection('127.0.0.1', server.server_port, timeout=3)
            try:
                connection.request('GET', route)
                response = connection.getresponse()
                body = response.read().decode()
                assert response.status == expected
                data = json.loads(body)
                if route.startswith('/_sparse/'):
                    assert data['vers'] == ('0.0.0' if mode == 'wrong-identity' else '1.2.3')
                    assert data['name'] == ('unrelated' if mode == 'wrong-identity' else 'fixture')
                cases.append({'mode': mode, 'route': route, 'status': response.status, 'body': data})
            finally:
                connection.close()
    finally:
        server.shutdown()
        server.server_close()
        thread.join()
    assert len(fixture.requests) == len(cases)
    return {'fixtureCasesPassed': len(cases), 'nativeExecutionCredit': 0, 'cases': cases}


if __name__ == '__main__':
    print(json.dumps(verify_controls(), indent=2))
