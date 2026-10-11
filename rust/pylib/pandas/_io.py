# SPDX-License-Identifier: MIT
# CSV, JSON and HTML input and output for rpy's pandas.

import csv
import io
import json
import math

from ._util import (NaN, NaT, isna_scalar, infer_dtype, normalize, convert,
                    parse_datetime, DT_OBJ, DT_INT, DT_FLOAT, DT_BOOL, DT_DATE)
from ._index import Index, default_index

_NA_STRINGS = {"", "#N/A", "#N/A N/A", "#NA", "-1.#IND", "-1.#QNAN", "-NaN", "-nan",
               "1.#IND", "1.#QNAN", "<NA>", "N/A", "NA", "NULL", "NaN", "None", "n/a",
               "nan", "null"}
_TRUE = {"True", "TRUE", "true"}
_FALSE = {"False", "FALSE", "false"}


def _read_text(src, encoding=None):
    if hasattr(src, "read"):
        t = src.read()
        if isinstance(t, bytes):
            t = t.decode(encoding or "utf-8")
        return t
    path = str(src)
    if "\n" in path and not path.endswith((".csv", ".json", ".txt", ".tsv")):
        raise FileNotFoundError(path[:40])
    with open(path, "rb") as f:
        data = f.read()
    text = data.decode(encoding or "utf-8") if not (encoding or "").lower().startswith("utf-8-sig") else data.decode("utf-8-sig")
    if text.startswith("﻿"):
        text = text[1:]
    return text


def _parse_rows(text, sep, quotechar, skipinitialspace, comment):
    if quotechar not in text and sep != r"\s+" and "\r" not in text:
        lines = text.split("\n")
        if lines and lines[-1] == "":
            lines.pop()
        rows = [l.split(sep) for l in lines]
    elif sep == r"\s+" or sep is None and False:
        rows = [l.split() for l in text.splitlines()]
    else:
        rows = list(csv.reader(io.StringIO(text), delimiter=sep, quotechar=quotechar, skipinitialspace=skipinitialspace))
    if skipinitialspace and quotechar not in text:
        rows = [[c.lstrip() for c in r] for r in rows]
    if comment:
        out = []
        for r in rows:
            if r and r[0].startswith(comment):
                continue
            out.append(r)
        rows = out
    return [r for r in rows if r and not (len(r) == 1 and r[0] == "")]


def _sniff(text):
    first = text.split("\n", 1)[0]
    counts = {d: first.count(d) for d in (",", ";", "\t", "|")}
    best = max(counts, key=lambda d: counts[d])
    return best if counts[best] else ","


def _convert_column(raw, na, thousands=None, decimal=".", true_values=None, false_values=None):
    """Strings from the file to the column's values and dtype."""
    n = len(raw)
    vals = [None] * n
    missing = [False] * n
    for i, s in enumerate(raw):
        if s in na:
            missing[i] = True
    # int
    ok = True
    has_na = False
    out = []
    for i, s in enumerate(raw):
        if missing[i]:
            has_na = True
            out.append(NaN)
            continue
        t = s
        if thousands:
            t = t.replace(thousands, "")
        try:
            out.append(int(t))
        except ValueError:
            ok = False
            break
    if ok:
        if has_na:
            return [float(v) if v == v else v for v in out], DT_FLOAT
        return out, DT_INT
    ok = True
    out = []
    for i, s in enumerate(raw):
        if missing[i]:
            out.append(NaN)
            continue
        t = s
        if thousands:
            t = t.replace(thousands, "")
        if decimal != ".":
            t = t.replace(decimal, ".")
        try:
            out.append(float(t))
        except ValueError:
            ok = False
            break
    if ok:
        return out, DT_FLOAT
    tv = _TRUE | set(true_values or [])
    fv = _FALSE | set(false_values or [])
    if all(missing[i] or s in tv or s in fv for i, s in enumerate(raw)):
        if not any(missing):
            return [s in tv for s in raw], DT_BOOL
        return [NaN if missing[i] else (s in tv) for i, s in enumerate(raw)], DT_OBJ
    return [NaN if missing[i] else s for i, s in enumerate(raw)], DT_OBJ


def read_csv(filepath_or_buffer, sep=",", delimiter=None, header="infer", names=None,
             index_col=None, usecols=None, dtype=None, parse_dates=None, nrows=None,
             skiprows=None, na_values=None, keep_default_na=True, na_filter=True,
             thousands=None, decimal=".", encoding=None, comment=None,
             skipinitialspace=False, quotechar='"', true_values=None, false_values=None,
             date_format=None, dayfirst=False, converters=None, skipfooter=0,
             engine=None, low_memory=True, on_bad_lines=None, encoding_errors=None, **kw):
    from ._frame import DataFrame
    text = _read_text(filepath_or_buffer, encoding)
    if delimiter is not None:
        sep = delimiter
    if sep is None:
        sep = _sniff(text)
    rows = _parse_rows(text, sep, quotechar, skipinitialspace, comment)
    if skiprows is not None:
        if isinstance(skiprows, int):
            rows = rows[skiprows:]
        else:
            skip = set(skiprows)
            rows = [r for i, r in enumerate(rows) if i not in skip]
    if skipfooter:
        rows = rows[:-skipfooter]
    if header == "infer":
        header = None if names is not None else 0
    if header is None:
        width = max((len(r) for r in rows), default=0)
        cols = list(names) if names is not None else list(range(width))
        body = rows
    else:
        cols = [c.strip() if isinstance(c, str) else c for c in rows[header]] if rows else []
        body = rows[header + 1:]
        if names is not None:
            cols = list(names)
    # de-duplicate column names like pandas (a, a.1)
    seen = {}
    fixed = []
    for c in cols:
        if c in seen:
            seen[c] += 1
            fixed.append("%s.%d" % (c, seen[c]))
        else:
            seen[c] = 0
            fixed.append(c)
    cols = fixed
    if header is not None and cols and len(body) and all(len(r) == len(cols) + 1 for r in body[:5]) and index_col is None:
        index_col = 0
        cols = [""] + cols
    if nrows is not None:
        body = body[:nrows]
    width = len(cols)
    na = set(_NA_STRINGS) if keep_default_na and na_filter else set()
    if na_values is not None and na_filter:
        if isinstance(na_values, str):
            na.add(na_values)
        elif isinstance(na_values, dict):
            for v in na_values.values():
                na.update([v] if isinstance(v, str) else v)
        else:
            na.update(str(v) for v in na_values)
    columns = [[] for _ in range(width)]
    for r in body:
        if len(r) < width:
            r = r + [""] * (width - len(r))
        for j in range(width):
            columns[j].append(r[j])
    if usecols is not None:
        if callable(usecols):
            keep = [j for j, c in enumerate(cols) if usecols(c)]
        else:
            ul = list(usecols)
            keep = [j for j, c in enumerate(cols) if c in ul or j in ul]
    else:
        keep = list(range(width))
    data = {}
    dts = {}
    dtype_map = dtype if isinstance(dtype, dict) else ({c: dtype for c in cols} if dtype is not None else {})
    for j in keep:
        c = cols[j]
        raw = columns[j]
        if converters and c in converters:
            data[c] = [converters[c](s) for s in raw]
            dts[c] = None
            continue
        if c in dtype_map:
            want = dtype_map[c]
            if want in (str, "str", "string", object, "object"):
                data[c] = [NaN if s in na else s for s in raw]
                dts[c] = DT_OBJ
                continue
            vals, dt = _convert_column(raw, na, thousands, decimal, true_values, false_values)
            vals, dt = convert(vals, want)
            data[c], dts[c] = vals, dt
            continue
        data[c], dts[c] = _convert_column(raw, na, thousands, decimal, true_values, false_values)
    if parse_dates:
        pd_cols = parse_dates if isinstance(parse_dates, (list, tuple)) else ([index_col] if parse_dates is True and index_col is not None else [])
        for c in pd_cols:
            if isinstance(c, int) and c not in data:
                c = cols[c]
            seen = {}
            out = []
            for v in data[c]:
                d = seen.get(v)
                if d is None:
                    d = parse_datetime(v, date_format, dayfirst) if not isna_scalar(v) else NaT
                    if not isna_scalar(v):
                        seen[v] = d
                out.append(d)
            data[c] = out
            dts[c] = DT_DATE
    df = DataFrame._new(data, dts, default_index(len(body)))
    if index_col is not None and index_col is not False:
        ic = index_col if isinstance(index_col, list) else [index_col]
        ic = [cols[c] if isinstance(c, int) else c for c in ic]
        df = df.set_index(ic)
        if ic == [""]:
            df.index.name = None
    return df


def read_table(filepath_or_buffer, sep="\t", **kw):
    return read_csv(filepath_or_buffer, sep=sep, **kw)


def read_json(path_or_buf, orient=None, typ="frame", lines=False, dtype=None, convert_dates=True, encoding=None, **kw):
    from ._frame import DataFrame
    from ._series import Series
    if isinstance(path_or_buf, str) and path_or_buf.lstrip()[:1] in ("{", "["):
        text = path_or_buf
    else:
        text = _read_text(path_or_buf, encoding)
    if lines:
        data = [json.loads(l) for l in text.splitlines() if l.strip()]
    else:
        data = json.loads(text)
    if typ == "series":
        if isinstance(data, dict):
            return Series(data)
        return Series(data)
    if orient == "split" or (isinstance(data, dict) and set(data.keys()) >= {"columns", "data"}):
        return DataFrame(data["data"], columns=data["columns"], index=data.get("index"))
    if orient == "index":
        return DataFrame.from_dict_index(data) if False else DataFrame([dict(v) for v in data.values()], index=list(data.keys()))
    if orient == "values":
        return DataFrame(data)
    if isinstance(data, list):
        df = DataFrame(data)
    elif isinstance(data, dict):
        vals = list(data.values())
        if vals and all(isinstance(v, dict) for v in vals):
            keys = []
            seen = set()
            for v in vals:
                for k in v:
                    if k not in seen:
                        seen.add(k)
                        keys.append(k)
            df = DataFrame({c: [data[c].get(k, NaN) for k in keys] for c in data}, index=_json_index(keys))
        elif vals and all(isinstance(v, list) for v in vals):
            df = DataFrame(data)
        else:
            df = DataFrame([data])
    else:
        raise ValueError("Unexpected JSON for a DataFrame")
    if convert_dates:
        for c in list(df._data):
            if isinstance(c, str) and (c.endswith("_at") or c.endswith("_time") or c.lower() in ("date", "datetime", "timestamp", "modified") or c.endswith("date")):
                try:
                    df._data[c] = [parse_datetime(v) for v in df._data[c]]
                    df._dtypes[c] = DT_DATE
                except (ValueError, TypeError):
                    pass
    if dtype:
        df = df.astype(dtype) if isinstance(dtype, dict) else df
    return df


def _json_index(keys):
    try:
        ints = [int(k) for k in keys]
        if [str(i) for i in ints] == list(keys):
            return Index(ints)
    except (ValueError, TypeError):
        pass
    return Index(keys)


def json_normalize(data, record_path=None, meta=None, sep=".", max_level=None, record_prefix=None, meta_prefix=None):
    from ._frame import DataFrame
    if isinstance(data, dict):
        data = [data]

    def flatten(d, prefix="", level=0):
        out = {}
        for k, v in d.items():
            key = prefix + sep + k if prefix else k
            if isinstance(v, dict) and (max_level is None or level < max_level):
                out.update(flatten(v, key, level + 1))
            else:
                out[key] = v
        return out

    if record_path is not None:
        path = record_path if isinstance(record_path, list) else [record_path]
        rows = []
        for item in data:
            recs = item
            for p in path:
                recs = recs[p]
            for r in recs:
                row = flatten(r) if isinstance(r, dict) else {0: r}
                if record_prefix:
                    row = {record_prefix + str(k): v for k, v in row.items()}
                for m in meta or []:
                    mk = m if isinstance(m, str) else sep.join(m)
                    mv = item
                    for p in ([m] if isinstance(m, str) else m):
                        mv = mv.get(p, NaN) if isinstance(mv, dict) else NaN
                    row[(meta_prefix or "") + mk] = mv
                rows.append(row)
        return DataFrame(rows)
    return DataFrame([flatten(d) for d in data])


# ------------------------------------------------------------------ output

def _csv_value(v, dt, na_rep, float_format, date_format, decimal):
    if isna_scalar(v):
        return na_rep
    if isinstance(v, bool):
        return str(v)
    if isinstance(v, float):
        if float_format:
            s = float_format % v if isinstance(float_format, str) else float_format(v)
        else:
            s = repr(v)
        if decimal != ".":
            s = s.replace(".", decimal)
        return s
    if hasattr(v, "year") and hasattr(v, "hour"):
        if date_format:
            from ._util import strftime
            return strftime(v, date_format)
        return None
    return str(v)


def to_csv(df, path_or_buf=None, sep=",", na_rep="", float_format=None, columns=None,
           header=True, index=True, index_label=None, mode="w", lineterminator="\n",
           quotechar='"', date_format=None, decimal="."):
    from ._fmt import date_only
    from ._util import fmt_datetime
    cols = list(columns) if columns is not None else list(df._data.keys())
    buf = io.StringIO()
    w = csv.writer(buf, delimiter=sep, quotechar=quotechar, lineterminator=lineterminator)
    idx = df.index
    if header:
        row = []
        if index:
            if index_label is not None:
                row.extend(index_label if isinstance(index_label, list) else [index_label])
            elif idx.nlevels > 1:
                row.extend("" if n is None else n for n in idx._names)
            else:
                row.append("" if idx.name is None else idx.name)
        hdr = header if isinstance(header, list) else cols
        row.extend(".".join(str(x) for x in c) if False else (str(c) if not isinstance(c, tuple) else str(c)) for c in hdr)
        if df.columns.nlevels > 1 and header is True:
            for li in range(df.columns.nlevels):
                lrow = ([("" if idx.name is None else idx.name) if li == df.columns.nlevels - 1 else ""] if index else []) + [str(c[li]) for c in cols]
                w.writerow(lrow)
        else:
            w.writerow(row)
    date_cols = {}
    for c in cols:
        if df._dtypes[c] == "datetime64[ns]":
            date_cols[c] = date_only(df._data[c])
    idx_date = idx.dtype == "datetime64[ns]" and date_only(idx._values)
    for i in range(len(idx)):
        row = []
        if index:
            lab = idx._values[i]
            if isinstance(lab, tuple):
                row.extend(_csv_value(x, None, na_rep, float_format, date_format, decimal) or fmt_datetime(x, False) for x in lab)
            elif idx.dtype == "datetime64[ns]":
                row.append(_csv_value(lab, None, na_rep, float_format, date_format, decimal) or fmt_datetime(lab, idx_date))
            else:
                row.append(_csv_value(lab, None, na_rep, float_format, date_format, decimal))
        for c in cols:
            v = df._data[c][i]
            s = _csv_value(v, df._dtypes[c], na_rep, float_format, date_format, decimal)
            if s is None:
                s = fmt_datetime(v, date_cols.get(c, False))
            row.append(s)
        w.writerow(row)
    out = buf.getvalue()
    if path_or_buf is None:
        return out
    if hasattr(path_or_buf, "write"):
        path_or_buf.write(out)
    else:
        with open(str(path_or_buf), mode) as f:
            f.write(out)
    return None


def _json_val(v, double_precision=10, date_format=None):
    if isna_scalar(v):
        return None
    if isinstance(v, float):
        if v != v or v in (math.inf, -math.inf):
            return None
        r = round(v, double_precision)
        return r
    if hasattr(v, "isoformat"):
        if date_format == "iso":
            return v.isoformat(timespec="milliseconds") if hasattr(v, "hour") else v.isoformat()
        import datetime as _d
        if isinstance(v, _d.datetime):
            return int((v - _d.datetime(1970, 1, 1)).total_seconds() * 1000)
        return v.isoformat()
    return v


def _dump(obj, indent, force_ascii):
    if indent:
        return json.dumps(obj, indent=indent, ensure_ascii=force_ascii)
    return json.dumps(obj, separators=(",", ":"), ensure_ascii=force_ascii)


def _write_out(s, path_or_buf):
    if path_or_buf is None:
        return s
    if hasattr(path_or_buf, "write"):
        path_or_buf.write(s)
    else:
        with open(str(path_or_buf), "w") as f:
            f.write(s)
    return None


def frame_to_json(df, path_or_buf=None, orient=None, indent=None, date_format=None,
                  double_precision=10, force_ascii=True, lines=False, index=None):
    orient = orient or ("records" if lines else "columns")
    cols = list(df._data.keys())
    dp = double_precision
    lab = lambda l: str(l) if not isinstance(l, str) else l
    if orient == "records":
        rows = [{str(c): _json_val(df._data[c][i], dp, date_format) for c in cols} for i in range(len(df))]
        if lines:
            return _write_out("\n".join(_dump(r, None, force_ascii) for r in rows) + ("\n" if rows else ""), path_or_buf)
        obj = rows
    elif orient == "columns":
        obj = {str(c): {lab(_json_val(l, dp, date_format)): _json_val(df._data[c][i], dp, date_format) for i, l in enumerate(df.index._values)} for c in cols}
    elif orient == "index":
        obj = {lab(_json_val(l, dp, date_format)): {str(c): _json_val(df._data[c][i], dp, date_format) for c in cols} for i, l in enumerate(df.index._values)}
    elif orient == "split":
        obj = {"columns": [str(c) for c in cols], "index": [_json_val(l, dp, date_format) for l in df.index._values],
               "data": [[_json_val(df._data[c][i], dp, date_format) for c in cols] for i in range(len(df))]}
        if index is False:
            del obj["index"]
    elif orient == "values":
        obj = [[_json_val(df._data[c][i], dp, date_format) for c in cols] for i in range(len(df))]
    elif orient == "table":
        obj = {"schema": {"fields": [{"name": str(c), "type": _schema_type(df._dtypes[c])} for c in cols]},
               "data": [{str(c): _json_val(df._data[c][i], dp, "iso") for c in cols} for i in range(len(df))]}
    else:
        raise ValueError("Invalid value '%s' for option 'orient'" % orient)
    return _write_out(_dump(obj, indent, force_ascii), path_or_buf)


def _schema_type(dt):
    return {"int64": "integer", "float64": "number", "bool": "boolean", "datetime64[ns]": "datetime"}.get(str(dt), "string")


def series_to_json(s, path_or_buf=None, orient="index", indent=None, date_format=None, double_precision=10, force_ascii=True, **kw):
    if orient == "records" or orient == "values":
        obj = [_json_val(v, double_precision, date_format) for v in s._values]
    elif orient == "split":
        obj = {"name": s.name, "index": list(s.index._values), "data": [_json_val(v, double_precision, date_format) for v in s._values]}
    else:
        obj = {str(_json_val(k, double_precision, date_format)): _json_val(v, double_precision, date_format) for k, v in zip(s.index._values, s._values)}
    return _write_out(_dump(obj, indent, force_ascii), path_or_buf)


def _esc(s):
    return str(s).replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;")


def to_html(df, index=True, classes=None, border=1, float_format=None, na_rep="NaN", **kw):
    from ._fmt import format_values
    cols = list(df._data.keys())
    cls = "dataframe" + ((" " + (classes if isinstance(classes, str) else " ".join(classes))) if classes else "")
    out = ['<table border="%d" class="%s">' % (border, cls), "  <thead>", '    <tr style="text-align: right;">']
    if index:
        out.append("      <th>%s</th>" % ("" if df.index.name is None else _esc(df.index.name)))
    for c in cols:
        out.append("      <th>%s</th>" % _esc(c))
    out += ["    </tr>", "  </thead>", "  <tbody>"]
    texts = {c: [t.strip() for t in format_values(df._data[c], df._dtypes[c], na_rep)] for c in cols}
    for i, lab in enumerate(df.index._values):
        out.append("    <tr>")
        if index:
            out.append("      <th>%s</th>" % _esc(lab))
        for c in cols:
            out.append("      <td>%s</td>" % _esc(texts[c][i]))
        out.append("    </tr>")
    out += ["  </tbody>", "</table>"]
    return "\n".join(out)
