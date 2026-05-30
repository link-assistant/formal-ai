# lino-objects-codec

[![JS CI](https://github.com/link-foundation/lino-objects-codec/actions/workflows/js.yml/badge.svg)](https://github.com/link-foundation/lino-objects-codec/actions/workflows/js.yml)
[![Python CI](https://github.com/link-foundation/lino-objects-codec/actions/workflows/python.yml/badge.svg)](https://github.com/link-foundation/lino-objects-codec/actions/workflows/python.yml)
[![Rust CI](https://github.com/link-foundation/lino-objects-codec/actions/workflows/rust.yml/badge.svg)](https://github.com/link-foundation/lino-objects-codec/actions/workflows/rust.yml)
[![C# CI](https://github.com/link-foundation/lino-objects-codec/actions/workflows/csharp.yml/badge.svg)](https://github.com/link-foundation/lino-objects-codec/actions/workflows/csharp.yml)

### Package versions

[![npm](https://img.shields.io/npm/v/lino-objects-codec?label=npm&logo=npm)](https://www.npmjs.com/package/lino-objects-codec)
[![PyPI](https://img.shields.io/pypi/v/lino-objects-codec?label=PyPI&logo=pypi&logoColor=white)](https://pypi.org/project/lino-objects-codec/)
[![crates.io](https://img.shields.io/crates/v/lino-objects-codec?label=crates.io&logo=rust)](https://crates.io/crates/lino-objects-codec)
[![NuGet](https://img.shields.io/nuget/v/Lino.Objects.Codec?label=NuGet&logo=nuget)](https://www.nuget.org/packages/Lino.Objects.Codec)
[![Python Version](https://img.shields.io/pypi/pyversions/lino-objects-codec.svg)](https://pypi.org/project/lino-objects-codec/)

> A "no badge / not found" state on any of the registry badges above means the corresponding
> package has not been published yet for that version. The badges read live from the package
> registries, so they always reflect the latest published state.

Universal serialization library to encode/decode objects to/from Links Notation format. Available in **Python**, **JavaScript**, **Rust**, and **C#** with identical functionality and API design.

## 🌍 Multi-Language Support

This library provides universal serialization and deserialization with built-in support for circular references and complex object graphs in:

- **[Python](python/)** - Full implementation for Python 3.8+
- **[JavaScript](js/)** - Full implementation for Node.js 18+
- **[Rust](rust/)** - Full implementation for Rust 1.70+
- **[C#](csharp/)** - Full implementation for .NET 8.0+

All implementations share the same design philosophy and provide feature parity.

## Features

- **Universal Serialization**: Encode objects to Links Notation format
- **Type Support**: Handle all common types in each language:
  - **Python**: `None`, `bool`, `int`, `float`, `str`, `list`, `dict`
  - **JavaScript**: `null`, `undefined`, `boolean`, `number`, `string`, `Array`, `Object`
  - **Rust**: `LinoValue` enum with `Null`, `Bool`, `Int`, `Float`, `String`, `Array`, `Object`
  - **C#**: `null`, `bool`, `int`, `long`, `float`, `double`, `string`, `List<object?>`, `Dictionary<string, object?>`
  - Special float/number values: `NaN`, `Infinity`, `-Infinity`
- **Circular References**: Automatically detect and preserve circular references
- **Object Identity**: Maintain object identity for shared references
- **UTF-8 Support**: Full Unicode string support using base64 encoding
- **Simple API**: Easy-to-use `encode()` and `decode()` functions
- **JSON/Lino Conversion**: Convert between JSON and Links Notation (JavaScript)
- **Reference Escaping**: Properly escape strings for Links Notation format (JavaScript)
- **Fuzzy Matching**: String similarity utilities for finding matches (JavaScript)
- **Indented Format**: Human-readable indented Links Notation format for display and debugging

## Quick Start

### Python

```bash
pip install lino-objects-codec
```

```python
from link_notation_objects_codec import encode, decode

# Encode and decode
data = {"name": "Alice", "age": 30, "active": True}
encoded = encode(data)
decoded = decode(encoded)
assert decoded == data
```

### JavaScript

```bash
npm install lino-objects-codec
```

```javascript
import { formatIndented, parseIndented } from "lino-objects-codec";

// Readable indented Links Notation for repository data
const data = { name: "Alice", age: 30, active: true };
const text = formatIndented({ id: "obj_root", obj: data });
const { obj } = parseIndented({ text });
console.log(JSON.stringify(obj) === JSON.stringify(data)); // true
```

### Rust

```toml
[dependencies]
lino-objects-codec = "0.1"
```

```rust
use lino_objects_codec::{encode, decode, LinoValue};

// Encode and decode
let data = LinoValue::object([
    ("name", LinoValue::String("Alice".to_string())),
    ("age", LinoValue::Int(30)),
    ("active", LinoValue::Bool(true)),
]);
let encoded = encode(&data);
let decoded = decode(&encoded).unwrap();
assert_eq!(decoded, data);
```

### C#

```bash
dotnet add package Lino.Objects.Codec
```

```csharp
using Lino.Objects.Codec;

// Encode and decode
var data = new Dictionary<string, object?>
{
    { "name", "Alice" },
    { "age", 30 },
    { "active", true }
};
var encoded = Codec.Encode(data);
var decoded = Codec.Decode(encoded) as Dictionary<string, object?>;
Console.WriteLine(decoded?["name"]); // Alice
```

## Repository Structure

```
.
├── python/           # Python implementation
│   ├── src/         # Source code
│   ├── tests/       # Test suite
│   ├── examples/    # Usage examples
│   └── README.md    # Python-specific docs
├── js/              # JavaScript implementation
│   ├── src/         # Source code
│   ├── tests/       # Test suite
│   ├── examples/    # Usage examples
│   └── README.md    # JavaScript-specific docs
├── rust/            # Rust implementation
│   ├── src/         # Source code
│   ├── examples/    # Usage examples
│   └── README.md    # Rust-specific docs
├── csharp/          # C# implementation
│   ├── src/         # Source code
│   ├── tests/       # Test suite
│   ├── examples/    # Usage examples
│   └── README.md    # C#-specific docs
└── README.md        # This file
```

## Language-Specific Documentation

For detailed documentation, API reference, and examples, see:

- **[Python Documentation](python/README.md)**
- **[JavaScript Documentation](js/README.md)**
- **[Rust Documentation](rust/README.md)**
- **[C# Documentation](csharp/README.md)**

## Usage Examples

All implementations support the same features with language-appropriate syntax:

### Circular References

**Python:**

```python
from link_notation_objects_codec import encode, decode

# Self-referencing list
lst = [1, 2, 3]
lst.append(lst)
decoded = decode(encode(lst))
assert decoded[3] is decoded  # Reference preserved
```

**JavaScript:**

```javascript
import { encode, decode } from "lino-objects-codec";

// Self-referencing array
const arr = [1, 2, 3];
arr.push(arr);
const decoded = decode(encode(arr));
console.log(decoded[3] === decoded); // true - Reference preserved
```

**Rust:**

```rust
use lino_objects_codec::{encode, decode, LinoValue};

// Self-referencing structures are handled via object IDs
let data = LinoValue::array([LinoValue::Int(1), LinoValue::Int(2)]);
let encoded = encode(&data);
let decoded = decode(&encoded).unwrap();
// Reference semantics preserved through encoding/decoding
```

**C#:**

```csharp
using Lino.Objects.Codec;

// Self-referencing list
var lst = new List<object?>();
lst.Add(lst);
var decoded = Codec.Decode(Codec.Encode(lst)) as List<object?>;
Console.WriteLine(ReferenceEquals(decoded, decoded?[0])); // True - Reference preserved
```

### Complex Nested Structures

**Python:**

```python
data = {
    "users": [
        {"id": 1, "name": "Alice"},
        {"id": 2, "name": "Bob"}
    ],
    "metadata": {"version": 1, "count": 2}
}
assert decode(encode(data)) == data
```

**JavaScript:**

```javascript
const data = {
  users: [
    { id: 1, name: "Alice" },
    { id: 2, name: "Bob" },
  ],
  metadata: { version: 1, count: 2 },
};
console.log(JSON.stringify(decode(encode(data))) === JSON.stringify(data));
```

**Rust:**

```rust
use lino_objects_codec::{encode, decode, LinoValue};

let data = LinoValue::object([
    ("users", LinoValue::array([
        LinoValue::object([("id", LinoValue::Int(1)), ("name", LinoValue::String("Alice".to_string()))]),
        LinoValue::object([("id", LinoValue::Int(2)), ("name", LinoValue::String("Bob".to_string()))]),
    ])),
    ("metadata", LinoValue::object([
        ("version", LinoValue::Int(1)),
        ("count", LinoValue::Int(2)),
    ])),
]);
assert_eq!(decode(&encode(&data)).unwrap(), data);
```

**C#:**

```csharp
var data = new Dictionary<string, object?>
{
    {
        "users", new List<object?>
        {
            new Dictionary<string, object?> { { "id", 1 }, { "name", "Alice" } },
            new Dictionary<string, object?> { { "id", 2 }, { "name", "Bob" } }
        }
    },
    { "metadata", new Dictionary<string, object?> { { "version", 1 }, { "count", 2 } } }
};
var decoded = Codec.Decode(Codec.Encode(data));
```

### Indented Links Notation Format

The indented format provides a human-readable representation for displaying objects:

**JavaScript:**

```javascript
import { formatIndented, parseIndented } from "lino-objects-codec";

// Format an object with an identifier
const formatted = formatIndented({
  id: "6dcf4c1b-ff3f-482c-95ab-711ea7d1b019",
  obj: {
    uuid: "6dcf4c1b-ff3f-482c-95ab-711ea7d1b019",
    status: "executed",
    command: "echo test",
    exitCode: "0",
  },
});
console.log(formatted);
// Output:
// 6dcf4c1b-ff3f-482c-95ab-711ea7d1b019:
//   uuid '6dcf4c1b-ff3f-482c-95ab-711ea7d1b019'
//   status executed
//   command 'echo test'
//   exitCode '0'

// Parse it back
const { id, obj } = parseIndented({ text: formatted });
```

**Python:**

```python
from link_notation_objects_codec import format_indented, parse_indented

# Format an object with an identifier
formatted = format_indented(
    '6dcf4c1b-ff3f-482c-95ab-711ea7d1b019',
    {'uuid': '6dcf4c1b-ff3f-482c-95ab-711ea7d1b019', 'status': 'executed'}
)

# Parse it back
id, obj = parse_indented(formatted)
```

**Rust:**

```rust
use lino_objects_codec::format::{format_indented_ordered, parse_indented};

// Format an object with an identifier
let pairs = [("status", "executed"), ("exitCode", "0")];
let formatted = format_indented_ordered("my-uuid", &pairs, "  ").unwrap();

// Parse it back
let (id, obj) = parse_indented(&formatted).unwrap();
```

**C#:**

```csharp
using Lino.Objects.Codec;

// Format an object with an identifier
var obj = new Dictionary<string, string?> { { "status", "executed" }, { "exitCode", "0" } };
var formatted = Format.FormatIndented("my-uuid", obj);

// Parse it back
var (id, parsedObj) = Format.ParseIndented(formatted);
```

## How It Works

The library uses the [links-notation](https://github.com/link-foundation/links-notation) format as the serialization target. Each object is encoded as a Link with type information:

- Basic types are encoded with type markers: `(int 42)`, `(str aGVsbG8=)`, `(bool True)`
- Strings are base64-encoded to handle special characters and newlines
- Collections with self-references use built-in links notation self-reference syntax:
  - **Format**: `(obj_id: type content...)`
  - **Python example**: `(obj_0: dict ((str c2VsZg==) obj_0))` for `{"self": obj}`
  - **JavaScript example**: `(obj_0: array (int 1) (int 2) obj_0)` for self-referencing array
- Simple collections without shared references use format: `(list item1 item2 ...)` or `(dict (key val) ...)`
- Circular references use direct object ID references: `obj_0` (without the `ref` keyword)

This approach allows for:

- Universal representation of object graphs
- Preservation of object identity
- Natural handling of circular references using built-in links notation syntax
- Cross-language compatibility

## Development

### Python

```bash
cd python
python -m venv .venv
source .venv/bin/activate
pip install -e ".[dev]"
pytest tests/ -v
```

### JavaScript

```bash
cd js
npm install
npm test
npm run example
```

### Rust

```bash
cd rust
cargo test
cargo run --example basic_usage
```

### C#

```bash
cd csharp
dotnet build
dotnet test
dotnet run --project examples/BasicUsage.csproj
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Add tests for your changes
4. Ensure all tests pass
5. Commit your changes (`git commit -m 'Add amazing feature'`)
6. Push to the branch (`git push origin feature/amazing-feature`)
7. Open a Pull Request

## License

This project is licensed under the Unlicense - see the [LICENSE](LICENSE) file for details.

## Links

- [GitHub Repository](https://github.com/link-foundation/lino-objects-codec)
- [Links Notation Specification](https://github.com/link-foundation/links-notation)
- [PyPI Package](https://pypi.org/project/lino-objects-codec/) (Python)
- [npm Package](https://www.npmjs.com/package/lino-objects-codec/) (JavaScript)
- [crates.io Package](https://crates.io/crates/lino-objects-codec/) (Rust)
- [NuGet Package](https://www.nuget.org/packages/Lino.Objects.Codec/) (C#)

## Acknowledgments

This project is built on top of the [links-notation](https://github.com/link-foundation/links-notation) library.
