# lino-rest-api

[![Tests](https://github.com/link-foundation/lino-rest-api/actions/workflows/test.yml/badge.svg)](https://github.com/link-foundation/lino-rest-api/actions/workflows/test.yml)

REST API frameworks using Links Notation (LINO) instead of JSON.

## Overview

This repository provides proof-of-concept implementations of REST API frameworks that use [Links Notation](https://github.com/link-foundation/links-notation) for data serialization instead of JSON. Links Notation is a human-readable format for describing data using references and links.

## Implementations

- **[JavaScript/Bun](./js/)** - Express.js-based implementation for Node.js and Bun
- **[Python](./python/)** - FastAPI-based implementation

## Quick Start

### JavaScript (Express.js/Bun)

```javascript
import { createLinoApp } from "lino-rest-api";

const app = createLinoApp();

app.get("/hello", () => {
  return { message: "Hello, Links Notation!" };
});

app.listen(3000);
```

### Python (FastAPI)

```python
from lino_rest_api import LinoAPI

api = LinoAPI()

@api.get("/hello")
def hello():
    return {"message": "Hello, Links Notation!"}
```

## Content Type

LINO APIs use `text/lino` as the content type for requests and responses.

## Related Projects

- [links-notation](https://github.com/link-foundation/links-notation) - Core Links Notation library
- [link-notation-objects-codec](https://github.com/link-foundation/link-notation-objects-codec) - Object encoding/decoding for Links Notation
- [test-anywhere](https://github.com/link-foundation/test-anywhere) - Universal testing framework

## License

[Unlicense](LICENSE)
