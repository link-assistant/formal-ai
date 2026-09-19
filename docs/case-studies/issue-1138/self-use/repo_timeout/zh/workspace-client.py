DEFAULT_TIMEOUT = 30


class Client:
    def __init__(self, timeout=DEFAULT_TIMEOUT):
        self.timeout = timeout
