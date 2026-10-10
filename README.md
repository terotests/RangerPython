# RangerPython

A Python runtime written in [Ranger](https://github.com/terotests/Ranger). It
runs Python 3 scripts on an in-memory filesystem, so a page (Sliqtly), a
Node tool or an AI agent can run a script that reads the files it is given
and writes new ones (an SVG chart, a CSV), and then take those files back.

The same source compiles to JavaScript (browser and Node), Go and Python;
the test suite runs on all three and compares each against CPython.

```js
import { runPython } from "ranger-python";

const r = runPython(source, { files: { "/data/sales.csv": csvText } });
r.ok          // false when the script raised
r.stdout      // what it printed
r.files       // { "/tmp/chart.svg": "<svg …" }  the files it wrote
r.traceback   // CPython-style report when it raised
```

## Sliqtly

`js/sliqtly.mjs` runs a script against a presentation's files in Sliqtly's
store (`web/vfs.js`):

```js
import { runInDeck } from "ranger-python/sliqtly";
const r = await runInDeck(vfs, doc.id, code);
r.saved   // [{ from: "/tmp/chart3d.svg", path: "media/chart3d.svg" }]
```

- The deck's text files are mounted at the root: `data/sales.csv` is
  `/data/sales.csv` (and `data/sales.csv` from the working folder `/`).
- A file the script writes in a deck folder (`media/`, `data/`, `charts/`,
  `apps/`) is saved there; any other path (`/tmp/chart3d.svg`) is saved by
  its name where Sliqtly puts that kind of file (`media/chart3d.svg`). Pass
  `place` to choose differently.
- A deck file the script deletes is deleted from the deck.
- `js/worker.mjs` runs scripts in a Worker so a long script does not block
  the page: `runInDeck(vfs, id, code, { run: workerRunner(worker) })`.

## Command line

```
node js/cli.mjs script.py [args…] [--mount DIR] [--max-steps N]
```

With `--mount`, DIR is the filesystem's root and what the script wrote is
saved back into DIR.

## What runs

The language: functions (defaults, `*args`, `**kw`, keyword-only), closures,
`global`/`nonlocal`, lambdas, decorators, classes with inheritance, `super()`,
properties, static and class methods, operator overloading and the other
dunder protocols, exceptions with `try`/`except`/`else`/`finally`, `raise
from`, `with`, generators and `yield from`, comprehensions, f-strings with
format specs, `%` formatting, `str.format`, slicing, unpacking, the walrus
operator, chained comparisons, `import` of `.py` files in the filesystem.

Number formatting (`repr` of floats, `:.2f`, `:g`, `:e`, grouping) is
correctly rounded and matches CPython.

Modules: `math`, `random` (CPython's Mersenne Twister: a seeded script draws
the same numbers), `json`, `os`, `os.path`, `sys`, `string`, `time`,
`collections` (`defaultdict`, `Counter`, `OrderedDict`, `namedtuple`,
`deque`), `itertools`, `functools`, `statistics`, `copy`, `typing`.

Files: `open` in modes `r`, `w`, `a`, `x`, text only; `os.listdir`,
`makedirs`, `remove`, `rename`, `walk` and the `os.path` functions.

Limits: `maxSteps` stops a runaway script; recursion stops at depth 200
with `RecursionError`.

## What does not run

- Generators run to completion when called, so an endless generator cannot
  be consumed lazily (it stops after a million values with a clear error),
  and `send()`, `itertools.count`, `cycle` and an endless `repeat` are not
  there.
- Integers are exact up to 2^53 on the JavaScript build (64-bit on Go); there
  are no big integers.
- No `bytes` (a `b"…"` literal is a `str`), no binary files.
- No `match` statement, `async`/`await`, complex numbers or `\N{…}` escapes.
- Not yet: `re`, `datetime`, `csv`, `dataclasses`, `decimal`, `fractions`.

## Building and testing

```
npm ci
npm test            # build dist/ and run the suite on the JavaScript build
npm run test:all    # also the Go and Python builds (needs go and python3)
node tests/run.mjs --update   # re-record tests/expected from CPython
```

`dist/pyruntime.mjs` (ESM) and `dist/pyruntime.cjs` are committed so the
runtime can be used without the Ranger compiler; CI checks they are the
build of `src/`.

The source, in `src/`:

| file | |
|---|---|
| `PyLexer.rgr`, `PyParser.rgr`, `PyAst.rgr` | tokens, indentation, the syntax tree |
| `PyInterp.rgr` | the tree-walking evaluator |
| `PyValue.rgr` | values and scopes |
| `PyBuiltins.rgr` | built-in functions and exception classes |
| `PyMethods.rgr` | methods of str, list, dict, set, file, … |
| `PyModules.rgr` | the standard library modules |
| `PyFormat.rgr` | number and string formatting |
| `PyFs.rgr` | the in-memory filesystem |
| `PyOps.rgr` | host operations per target |
| `PyRuntime.rgr` | the public API |
| `PyMain.rgr` | command line runner for the Go and Python builds |

## License

MIT
