from client import Client


def fetch(url):
    return Client().get(url)
