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
random, statistics, copy, struct, asyncio, pathlib, glob) follow CPython's algorithms, so a
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

Best of 5, ms, from `docs/BENCHMARKS.md` ("Ranger→Rust" is the build compiled
from the Ranger sources by the Rust target; its and Go's columns are from
an earlier run):

| program | CPython | JavaScript | Go | Ranger→Rust | rpy native | rpy wasm (Node) |
|---|---:|---:|---:|---:|---:|---:|
| classes | 171 | 901 | 2557 | 1419 | 374 | 721 |
| dicts | 128 | 1251 | 2138 | 1205 | 226 | 499 |
| fib | 130 | 1554 | 4691 | 2615 | 239 | 484 |
| nbody | 271 | 3029 | 8101 | 3767 | 615 | 1228 |
| spectral | 154 | 2031 | 5474 | 2569 | 307 | 572 |
| strings | 132 | 1262 | 2639 | 1391 | 249 | 459 |

Native rpy is 1.8–2.3x CPython's time and 2.4–6.6x faster than the
JavaScript build. Under Node's WASI the wasm build is about 2x slower than
native and still 1.2–3.6x faster than the JavaScript build.

What makes it that fast: the arithmetic and comparisons of ints and floats,
`is`, truth tests, list/tuple/dict indexing and the common iterators run
inline in the interpreter loop; a call to a plain function moves its
arguments straight into the new frame, whose stack comes from a pool;
LoadGlobal, LoadName and StoreName remember the dict entry a name was
found at (and a builtin is remembered while the globals gain no key);
the names of built-ins are interned, so dict lookups match them by
identity; common instruction pairs (two loads, a load and an attribute, a
compare and its jump) run as one; and `src/poolalloc.rs` serves the small
blocks (strs, tuples, list buffers) from free lists, as pymalloc does.

## Not implemented

The datetime and unicodedata modules, subprocesses, and CPython internals
such as reference counts and `sys.getsizeof`.
