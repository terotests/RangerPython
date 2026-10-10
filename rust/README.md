# rpy: the runtime in hand-written Rust

A Python 3 interpreter written directly in Rust, separate from the Ranger
sources in `src/`. It has the same contract as the other builds: a script
runs on an in-memory filesystem, and the host reads back the files it
wrote. It compiles to a native binary and to `wasm32-wasip1`.

```sh
cargo build --release                               # target/release/rpy
cargo build --release --target wasm32-wasip1        # target/wasm32-wasip1/release/rpy.wasm

target/release/rpy script.py [outdir]               # files written to /a/b.txt land in outdir/a__b.txt
node --stack-size=7000 wasi-run.mjs target/wasm32-wasip1/release/rpy.wasm script.py
```

As a library:

```rust
let mut rt = rpy::Runtime::new();
rt.write_file("/data/sales.csv", &csv);
let r = rt.run(&source, "main.py");
// r.ok, r.stdout, r.traceback; rt.changed_files(), rt.read_file(path)
```

## How it is built

| file | what it does |
|---|---|
| `lexer.rs`, `parser.rs`, `ast.rs` | tokens (3.12 f-string rules), a recursive-descent parser to an AST |
| `compile.rs`, `code.rs` | symbol tables (locals, cells, free variables, name mangling) and a bytecode compiler |
| `vm.rs` | the interpreter loop, calls, frames, generators and coroutines, classes, imports |
| `value.rs` | `Value` (16 bytes: scalars inline, heap objects behind `Rc`), the insertion-ordered dict |
| `ops.rs` | attribute lookup, operators and comparisons, iteration, hashing |
| `builtins.rs`, `methods.rs`, `strmethods.rs` | built-in functions and the methods of the built-in types |
| `format.rs` | `repr`, `format()`, `%` and the float formatting rules |
| `modules.rs` | native modules: sys, math, time, os, os.path, json, _random (MT19937), _struct, _hashlib |
| `bigint.rs` | arbitrary-precision integers |
| `fs.rs` | the in-memory filesystem and the list of changed files |
| `stdlib.rs` + `build.rs` | `../lib/*.py` and `pylib/*.py` embedded in the binary |

Exceptions are `Result<_, Exc>` values, so `try`/`except` costs nothing until
something raises, and there is no unwinding (`panic = "abort"`). Natives are
plain `fn(&mut Vm, &[Value], kwargs) -> PyResult<Value>` pointers. A call to a
Python function with only positional arguments reuses a pooled value stack,
so it allocates nothing.

The Python-level modules in `pylib/` (collections, functools, itertools,
random, statistics, copy, struct, asyncio) follow CPython's algorithms, so a
seeded `random` run or a `statistics` result matches CPython's digits.

## Tests

CPython's own tests: 1922 of 3582 pass (53.7 %, the JavaScript build 32.1 %), see
`docs/COMPATIBILITY-RUST.md`. Feature matrix: 101 of 102 (datetime is missing).


```sh
node tests/run.mjs --rpy                 # the test scripts against CPython's output
node tests/features/run.mjs --rpy        # the feature matrix, docs/FEATURES.md
node tests/conformance/run.mjs --rpy     # CPython's own tests, docs/COMPATIBILITY-RUST.md
node tests/bench/run.mjs --rpy --wasm    # speed, docs/BENCHMARKS.md
```

## Speed

Best of 3, ms, from `docs/BENCHMARKS.md` ("Ranger→Rust" is the build compiled
from the Ranger sources by the Rust target):

| program | CPython | JavaScript | Go | Ranger→Rust | rpy native | rpy wasm (Node) |
|---|---:|---:|---:|---:|---:|---:|
| classes | 142 | 984 | 2557 | 1419 | 439 | 889 |
| dicts | 137 | 1211 | 2138 | 1205 | 278 | 550 |
| fib | 128 | 1440 | 4691 | 2615 | 331 | 583 |
| nbody | 289 | 2281 | 8101 | 3767 | 715 | 1441 |
| spectral | 154 | 1767 | 5474 | 2569 | 365 | 553 |
| strings | 203 | 1426 | 2639 | 1391 | 288 | 574 |

Native rpy is 1.4–3.1x CPython's time and 3–5x faster than the JavaScript
build. Under Node's WASI the wasm build is about 2x slower than native and
still 1.1–3.2x faster than the JavaScript build.

## Not implemented

The datetime and unicodedata modules, subprocesses, and CPython internals
such as reference counts and `sys.getsizeof`.
