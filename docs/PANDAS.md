# pandas in rpy

`import pandas as pd` works in the Rust runtime (native and wasm) without
numpy or any C extension. The package is pure Python in
`rust/pylib/pandas/` and is embedded in the binary like the rest of the
standard library. It covers the part of pandas that analysis scripts use,
and prints what pandas prints, so a script written for pandas runs here
unchanged.

`node tests/pandas/run.mjs` (or `--wasm`) runs the scripts in
`tests/pandas/` and compares their output with pandas 3.0's, recorded in
`pd_*.out` (`--record` records it again). All of them match byte for byte,
including the float formatting, column alignment, truncation of long and
wide frames, and numpy's summation order.

## What is there

- **Reading and writing:** `read_csv` (sep, decimal, thousands, usecols,
  dtype, parse_dates, index_col, nrows, skiprows, na_values, quoting),
  `read_json` (records, columns, split, index, lines), `json_normalize`,
  `to_csv`, `to_json`, `to_dict`, `to_markdown`, `to_html`, `to_string`.
- **Frames and series:** construction from dicts, records and lists;
  `head`/`tail`, `columns`, `dtypes`, `shape`, `info`, `describe`,
  selection with `[]`, boolean masks, `loc`/`iloc`/`at`/`iat` (also as
  assignment targets), `query`, `eval`, `assign`, `drop`, `rename`,
  `astype`, `fillna`/`dropna`/`ffill`/`interpolate`, `replace`, `where`,
  `isin`, `between`, `sort_values`/`sort_index`, `nlargest`,
  `drop_duplicates`, `set_index`/`reset_index`, `apply`/`map`, arithmetic
  and comparisons with index alignment, `cumsum`, `diff`, `pct_change`,
  `shift`, `rank`, `round`, `corr`, `value_counts`, `unique`, `.str` and
  `.dt` accessors.
- **Grouping and reshaping:** `groupby` (by columns, Series, functions or
  levels; `sum`, `mean`, `count`, `size`, `agg` with lists, dicts and named
  aggregations, `transform`, `apply`, `filter`, `as_index=False`),
  `pivot_table` (with margins), `pivot`, `crosstab`, `melt`, `stack`/
  `unstack`, `merge` (inner, left, right, outer, cross), `concat`,
  `cut`/`qcut`, `get_dummies`.
- **Time:** `to_datetime`, `date_range`, `resample` (D, W, MS, ME, QE,
  YE), `rolling`, `expanding`, `ewm`.

## Differences from pandas

- There is no numpy: `.values` and `unique()` return a list that prints like
  a numpy array and has `tolist()`. `import numpy` fails.
- `cut` returns plain labels (object dtype), not a categorical.
- Datetime columns report `datetime64[ns]` (pandas 3.0 says
  `datetime64[us]` for parsed strings).
- No plotting (`df.plot` explains to write SVG instead), no `read_excel`
  or `read_parquet`, no time-based rolling windows, no extension dtypes.
- Sorting is stable everywhere; pandas' default quicksort can order ties
  differently.

## Speed

Columns are Python lists, so everything runs at interpreter speed. On the
native build a 50 000-row CSV with a date column takes about 0.35 s to
read, and a groupby, a month aggregation, a pivot table and a sort over it
take about 0.8 s together. The same package run on CPython 3.13 takes
0.09 s and 0.27 s, and real pandas 0.03 s for each part. That is enough
for the data a slide shows; it is not meant for millions of rows.
