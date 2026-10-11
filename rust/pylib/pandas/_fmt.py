# SPDX-License-Identifier: MIT
# Text formatting for rpy's pandas, following pandas' own rules: every
# value gets a leading space for the sign, floats share one precision per
# column with trailing zeros trimmed, and columns are joined by one space.

import re
import math

from ._util import (NaN, NaT, NA, isna_scalar, get_option, fmt_datetime,
                    fmt_timedelta)

_NUM_DEC = re.compile(r"^\s*[\+-]?[0-9]+\.[0-9]*$")


def _trim_zeros(strs):
    def numbers(vals):
        return [x for x in vals if _NUM_DEC.match(x)]
    while True:
        nums = numbers(strs)
        if not nums or not all(x.endswith("0") for x in nums):
            break
        strs = [x[:-1] if _NUM_DEC.match(x) else x for x in strs]
    return [x + "0" if _NUM_DEC.match(x) and x.endswith(".") else x for x in strs]


def _float_str(v, digits, sci):
    if v != v:
        return "NaN"
    if v == math.inf:
        return " inf"
    if v == -math.inf:
        return "-inf"
    if sci:
        return "{: .{}e}".format(v, digits)
    return "{: .{}f}".format(v, digits)


def format_floats(vals, na_rep="NaN"):
    ff = get_option("display.float_format")
    if ff is not None:
        return [na_rep if v != v else " " + ff(v) for v in vals]
    digits = get_option("display.precision")
    out = _trim_zeros([_float_str(v, digits, False) for v in vals])
    maxlen = max((len(x) for x in out), default=0)
    too_long = maxlen > digits + 6
    absv = [abs(v) for v in vals if v == v and v not in (math.inf, -math.inf)]
    has_large = any(a > 1e6 for a in absv)
    has_small = any(0 < a < 10 ** (-digits) for a in absv)
    if has_small or (too_long and has_large):
        out = [_float_str(v, digits, True) for v in vals]
    if na_rep != "NaN":
        out = [na_rep if x == "NaN" else x for x in out]
    return out


def _obj_str(v):
    if v is None:
        return "None"
    if isinstance(v, float):
        if v != v:
            return "NaN"
        return repr(v)
    if v is NaT:
        return "NaT"
    if v is NA:
        return "<NA>"
    if isinstance(v, (list, tuple)):
        inner = ", ".join(_obj_str(x) if not isinstance(x, str) else x for x in v)
        return ("[%s]" if isinstance(v, list) else "(%s)") % inner
    if isinstance(v, dict):
        return repr(v)
    s = str(v)
    mcw = get_option("display.max_colwidth")
    if mcw and len(s) > mcw:
        s = s[:mcw - 3] + "..."
    return s.replace("\n", "\\n")


def date_only(vals):
    for v in vals:
        if v is NaT or v is None:
            continue
        if v.hour or v.minute or v.second or v.microsecond:
            return False
    return True


def format_values(vals, dt, na_rep="NaN"):
    """Each value as text with pandas' leading space."""
    if dt == "float64":
        return format_floats(vals, na_rep)
    if dt == "int64":
        return ["{: d}".format(v) for v in vals]
    if dt == "bool":
        return [" " + str(v) for v in vals]
    if dt == "datetime64[ns]":
        do = date_only(vals)
        return [fmt_datetime(v, do) for v in vals]
    if dt == "timedelta64[ns]":
        return [" " + fmt_timedelta(v) for v in vals]
    out = []
    for v in vals:
        if isinstance(v, float) and v == v:
            out.append(" " + _obj_str(v))
        else:
            out.append(" " + (na_rep if (na_rep != "NaN" and isna_scalar(v)) else _obj_str(v)))
    return out


def _trim_front(strs):
    while strs and all(x.startswith(" ") for x in strs):
        strs = [x[1:] for x in strs]
    return strs


def format_index_level(vals, dt):
    return _trim_front(format_values(vals, dt))


def sparsify(levels):
    """Blank repeated outer labels in multi-level index text (all but last level)."""
    n = len(levels[0]) if levels else 0
    out = [list(l) for l in levels]
    for li in range(len(levels) - 1):
        prev = None
        for i in range(n):
            key = tuple(levels[k][i] for k in range(li + 1))
            if key == prev:
                out[li][i] = ""
            prev = key
    return out


def _index_texts(index):
    """Per level, the label strings (sparsified)."""
    if index.nlevels == 1:
        return [format_index_level(index._values, index.dtype)]
    levels = []
    for li in range(index.nlevels):
        vals = [t[li] for t in index._values]
        from ._util import infer_dtype
        levels.append(format_index_level(vals, infer_dtype(vals)))
    return sparsify(levels)


def _rows_to_show(n):
    max_rows = get_option("display.max_rows")
    if max_rows and n > max_rows:
        mr = get_option("display.min_rows") or max_rows
        mr = min(mr, max_rows)
        head = (mr + 1) // 2
        tail = mr // 2
        return list(range(head)) + list(range(n - tail, n)), head
    return list(range(n)), None


def _adjoin(strcols, space=1):
    widths = [max((len(x) for x in col), default=0) for col in strcols]
    nrows = max((len(c) for c in strcols), default=0)
    lines = []
    for r in range(nrows):
        parts = []
        for ci, col in enumerate(strcols):
            cell = col[r] if r < len(col) else ""
            parts.append(cell.ljust(widths[ci]))
        lines.append((" " * space).join(parts))
    return lines


def frame_to_string(df, max_rows=None, show_dims=None):
    from ._util import infer_dtype
    n = len(df.index)
    cols = list(df._data.keys())
    if not cols or n == 0:
        if not cols and n == 0:
            return "Empty DataFrame\nColumns: []\nIndex: []"
        if not cols:
            return "Empty DataFrame\nColumns: []\nIndex: [%s]" % ", ".join(str(x) for x in df.index._values)
        return "Empty DataFrame\nColumns: [%s]\nIndex: []" % ", ".join(_col_label(c) for c in cols)
    rows, trunc_at = _rows_to_show(n)
    if max_rows is not None:
        rows, trunc_at = list(range(n)), None

    col_levels = df.columns.nlevels
    col_names = df.columns.names
    idx = df.index.take(rows)
    idx_texts = _index_texts(idx)
    has_idx_names = any(nm is not None for nm in df.index.names)
    has_col_names = any(nm is not None for nm in col_names)

    # header labels per column level
    if col_levels == 1:
        header_rows = [[(" " if df._dtypes[c] in ("int64", "float64", "bool") else "") + _col_label(c) for c in cols]]
    else:
        header_rows = [[str(c[li]) if not _isna_label(c[li]) else "" for c in cols] for li in range(col_levels)]
        header_rows = sparsify(header_rows) if col_levels > 1 else header_rows
    n_header = col_levels + (1 if has_idx_names else 0)

    def data_col(ci, c):
        vals = df._data[c]
        sub = [vals[i] for i in rows]
        texts = format_values(sub, df._dtypes[c])
        if trunc_at is not None:
            texts = texts[:trunc_at] + [".."] + texts[trunc_at:]
        head = [header_rows[li][ci] for li in range(col_levels)]
        if has_idx_names:
            head.append("")
        w = max(len(x) for x in head + texts)
        return [h.rjust(w) for h in head] + [t.rjust(w) for t in texts]

    idx_cols = []
    nlev = len(idx_texts)
    for li in range(nlev):
        texts = list(idx_texts[li])
        if trunc_at is not None:
            texts = texts[:trunc_at] + [".." if li == 0 else ""] + texts[trunc_at:]
        head = []
        for cl in range(col_levels):
            if li == nlev - 1 and has_col_names and col_names[cl] is not None:
                head.append(str(col_names[cl]))
            else:
                head.append("")
        if has_idx_names:
            nm = df.index.names[li]
            head.append("" if nm is None else str(nm))
        idx_cols.append(head + texts)

    strcols_all = [data_col(ci, c) for ci, c in enumerate(cols)]

    # columns that fit the terminal width (pandas' terminal behaviour)
    max_cols = get_option("display.max_columns")
    width = get_option("display.width")
    shown = list(range(len(cols)))
    trunc_cols = False
    if max_cols and len(cols) > max_cols:
        k = max_cols // 2
        shown = list(range(k)) + list(range(len(cols) - (max_cols - k), len(cols)))
        trunc_cols = True
    elif not max_cols and width:
        lines = _adjoin(idx_cols + strcols_all)
        max_len = max(len(l) for l in lines)
        if max_len > width:
            lens = [max(len(x) for x in c) for c in idx_cols + strcols_all]
            ids = list(range(len(lens)))
            adj = max_len - width + 1
            while adj > 0 and len(ids) > 1:
                mid = _round_half_even(len(ids) / 2)
                adj -= lens[ids[mid]] + 1
                del ids[mid]
            fitted = max(len(ids) - len(idx_cols), 2)
            if fitted < len(cols):
                k = fitted // 2
                shown = list(range(k)) + list(range(len(cols) - k, len(cols)))
                trunc_cols = True
    strcols = []
    for j, ci in enumerate(shown):
        if trunc_cols and j == len(shown) // 2 and j > 0:
            dots = [""] * n_header + [" ..."] * (len(strcols_all[0]) - n_header)
            dots[n_header - 1 if not has_idx_names else n_header - 2] = " ..."
            strcols.append(dots)
        strcols.append(strcols_all[ci])
    lines = _adjoin(idx_cols + strcols)
    out = "\n".join(lines)
    if trunc_at is not None or trunc_cols or show_dims:
        out += "\n\n[%d rows x %d columns]" % (n, len(cols))
    return out


def _round_half_even(x):
    return int(round(x))


def _isna_label(v):
    try:
        return isna_scalar(v)
    except Exception:
        return False


def _col_label(c):
    if isinstance(c, tuple):
        return str(c)
    return str(c)


def series_to_string(s, max_rows=None):
    n = len(s._values)
    dt = s.dtype
    footer_parts = []
    if s.name is not None:
        footer_parts.append("Name: %s" % (s.name,))
    if n == 0:
        nm = ("Name: %s, " % (s.name,)) if s.name is not None else ""
        return "Series([], %sdtype: %s)" % (nm, dt)
    rows, trunc_at = _rows_to_show(n)
    if max_rows is not None:
        rows, trunc_at = list(range(n)), None
    if trunc_at is not None:
        footer_parts.append("Length: %d" % n)
    footer_parts.append("dtype: %s" % dt)
    sub = [s._values[i] for i in rows]
    vals = format_values(sub, dt)
    idx = s.index.take(rows)
    idx_texts = _index_texts(idx)
    if trunc_at is not None:
        vals = vals[:trunc_at] + [".."] + vals[trunc_at:]
        idx_texts = [t[:trunc_at] + [""] + t[trunc_at:] for t in idx_texts]
    names = s.index.names
    has_names = any(nm is not None for nm in names)
    head = None
    if len(idx_texts) == 1:
        it = idx_texts[0]
        if has_names:
            head = str(names[0]) if names[0] is not None else ""
    else:
        widths = [max(len(x) for x in lv) for lv in idx_texts]
        if has_names:
            widths = [max(w, len(str(nm)) if nm is not None else 0) for w, nm in zip(widths, names)]
            head = "  ".join((str(nm) if nm is not None else "").ljust(widths[k]) for k, nm in enumerate(names))
        it = ["  ".join(lv[i].ljust(widths[k]) for k, lv in enumerate(idx_texts)) for i in range(len(vals))]
    iw = max(len(x) for x in it)
    if head is not None and len(idx_texts) > 1:
        iw = max(iw, len(head))
    vw = max(len(x) for x in vals)
    lines = []
    if head is not None:
        lines.append(head)
    for i in range(len(vals)):
        lines.append(it[i].ljust(iw) + "   " + vals[i].rjust(vw))
    freq = getattr(s.index, "freq", None)
    if freq:
        footer_parts.insert(0, "Freq: %s" % freq)
    lines.append(", ".join(footer_parts))
    return "\n".join(lines)


def index_repr(index):
    from ._core import RangeIndex
    if isinstance(index, RangeIndex):
        return "RangeIndex(start=%d, stop=%d, step=%d)" % (index._start, index._stop, index._step)
    if index.nlevels > 1:
        items = [repr(t) for t in index._values]
        if not items:
            return "MultiIndex([], names=%r)" % (list(index.names),)
        body = (",\n" + " " * 12).join(items)
        return "MultiIndex([%s],\n           names=%r)" % (body, list(index.names))
    vals = index._values
    dt = index.dtype
    if dt == "datetime64[ns]":
        do = date_only(vals)
        items = ["'%s'" % fmt_datetime(v, do) for v in vals]
        freq = getattr(index, "freq", None)
        tail = "dtype='datetime64[ns]'%s, freq=%s)" % ("" if index.name is None else ", name=%r" % (index.name,), "None" if not freq else repr(freq))
        one = "DatetimeIndex([%s], %s" % (", ".join(items), tail)
        if len("[%s]" % ", ".join(items)) + 14 <= 80 or len(items) < 2:
            return one
        lines = []
        cur = ""
        for it in items:
            piece = it + ", "
            if len(cur) + len(piece) > 66 and cur:
                lines.append(cur.rstrip())
                cur = ""
            cur += piece
        lines.append(cur.rstrip().rstrip(","))
        return "DatetimeIndex([" + ("\n" + " " * 15).join(lines) + "],\n              " + tail
    elif dt == "float64":
        items = [repr(v) if v == v else "nan" for v in vals]
    else:
        items = [repr(v) for v in vals]
    name = "" if index.name is None else ", name=%r" % (index.name,)
    s = "Index([%s], dtype='%s'%s)" % (", ".join(items), dt, name)
    if len("[%s]" % ", ".join(items)) + 6 > 80 and len(items) > 1:
        lines = []
        cur = ""
        for it in items:
            piece = it + ", "
            if len(cur) + len(piece) > 74 and cur:
                lines.append(cur.rstrip())
                cur = ""
            cur += piece
        lines.append(cur.rstrip().rstrip(","))
        s = "Index([" + ("\n       ").join(lines) + "],\n      dtype='%s'%s)" % (dt, name)
    return s


def to_markdown(df, index=True):
    from ._series import Series
    if isinstance(df, Series):
        df = df.to_frame()
    cols = list(df._data.keys())
    headers = []
    columns = []
    aligns = []
    if index:
        idx_vals = [str(v) for v in df.index._values]
        nm = df.index.name
        headers.append("" if nm is None else str(nm))
        columns.append(idx_vals)
        aligns.append("right" if df.index.dtype in ("int64", "float64") else "left")
    for c in cols:
        dt = df._dtypes[c]
        vals = df._data[c]
        if dt == "float64":
            texts = _decimal_align([_md_float(v) for v in vals])
        elif dt == "datetime64[ns]":
            do = date_only(vals)
            texts = [fmt_datetime(v, do) for v in vals]
        else:
            texts = [_obj_str(v) if not isinstance(v, str) else v for v in vals]
        headers.append(_col_label(c))
        columns.append(texts)
        aligns.append("right" if dt in ("int64", "float64") else "left")
    widths = [max([len(h) + 2] + [len(t) for t in col]) for h, col in zip(headers, columns)]
    lines = []

    def cell(t, w, a):
        return " " + (t.rjust(w) if a == "right" else t.ljust(w)) + " "

    lines.append("|" + "|".join(cell(h, w, a) for h, w, a in zip(headers, widths, aligns)) + "|")
    lines.append("|" + "|".join((":" + "-" * (w + 1)) if a == "left" else ("-" * (w + 1) + ":") for w, a in zip(widths, aligns)) + "|")
    for r in range(len(df.index)):
        lines.append("|" + "|".join(cell(col[r], w, a) for col, w, a in zip(columns, widths, aligns)) + "|")
    return "\n".join(lines)


def _decimal_align(texts):
    fr = [len(t) - t.index(".") if "." in t else 0 for t in texts]
    m = max(fr, default=0)
    return [t + " " * (m - f) for t, f in zip(texts, fr)]


def _md_float(v):
    if v != v:
        return "nan"
    return format(v, "g")
