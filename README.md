# ndjsonloggercore

ndjsonloggercore is the library powering
[ndjsonlogger](https://github.com/flickpp/ndjsonlogger).
It can be used as a stand-alone crate, althought it leads to verbose code without
the macros provided in ndjsonlogger.

## Example

```toml
[dependencies]
ndjsonlogger = {version = "0.1", features = ["std"]}
```

```rust
use ndjsonloggercore::{log, level, Entry. Value, Atom, StdoutOutputter};

fn main() {
	// StdoutOutputter requires `std` crate feature.
	let mut outputter = StdoutOutputter::new();

	log(None, &mut outputter, "service started", level::Info, &[]);
	
	let my_num: u64 = 15;
	log(None, &mut outputter, "a log line", level::Error, &[
		Entry{ key: "key1", value: Value::Atom(Atom::String("value1")) },
		Entry{ key: "key2", value: Value::Atom(Atom::Uint(my_num)) },
	]);
}
```

```json
{"level": "info", "msg": "service started"}
{"level": "error", "msg": "a log line", "key1": "value1", "key2": 15}
```

## Features

### iso timestamp
To log an iso utc timestamp enable the isotimestamp feature.

NOTE: This implicity enables the `std` feature as well.

```toml
[dependencies]
ndjosnloggercore = {version = "0.1", features = ["isotimestamp", "std"]}
```

```json
{"level": "info", "ts": "2022-07-13T16:47:36.429838", "msg": "example message"}
```

## Contributing

Contributions Welcome! Please open a github issue or pull request.
