
# ndjsonloggercore

ndjsonloggercore is the library powering [ndjsonlogger][1].
It can be used as a stand-alone crate, althought it leads to verbose code without
the macros provided in ndjsonlogger.

[1] https://github.com/flickpp/ndjsonlogger

## Example

```rust
use ndjsonloggercore::{log, level, Entry. Value, Atom, StdoutOutputter};

fn main() {
	let mut outputter = StdoutOutputter::new();

	log(None, &mut outputter, "service started", level::Info, &[]);
	
	let my_num: f64 = 15.3;
	log(None, &mut outputter, "a log line", level::Error, &[
		Entry{ key: "key1", value: Value::Atom(Atom::String("value1")) },
		Entry{ key: "key2", value: Value::Atom(Atom::Float(my_num)) },
	]);
}
```

```json
{"level": "info", "msg": "service started"}
{"level": "error", "msg": "a log line", "key1": "value1", "key2": 1.53e1}
```

## Features

### iso timestamp
To log an iso timestamp enable the isotimestamp feature.

```toml
ndjosnloggercore = {version = "0.1", features = ["isotimestamp", "std"]}
```

```json
{"level": "info", "ts": "2022-07-13T16:47:36.429838", "msg": "example message"}
```

## Contributing

Contributions Welcome! Please open a github issue or pull request.
