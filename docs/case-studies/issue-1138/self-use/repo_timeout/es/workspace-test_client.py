from client import Client, DEFAULT_TIMEOUT


def test_default_timeout():
    assert Client().timeout == DEFAULT_TIMEOUT


def test_client_constructs():
    assert Client(timeout=5).timeout == 5
