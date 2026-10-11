# SPDX-License-Identifier: MIT
# merge, concat, pivot_table, crosstab, cut and get_dummies for rpy's pandas.

from ._util import NaN, NaT, isna_scalar, infer_dtype, is_numeric, DT_FLOAT, DT_INT
from ._index import Index, RangeIndex, default_index, _hkey, _sort_key
from ._series import Series, _na_for
from . import _agg


def _key_arrays(df, cols, use_index):
    if use_index:
        idx = df.index
        if idx.nlevels == 1:
            return [idx._values]
        return [[t[i] for t in idx._values] for i in range(idx.nlevels)]
    return [df._data[c] if c in df._data else df.index.get_level_values(c)._values for c in cols]


def merge(left, right, how="inner", on=None, left_on=None, right_on=None, left_index=False, right_index=False, sort=False, suffixes=("_x", "_y"), indicator=False):
    from ._frame import DataFrame
    if isinstance(right, Series):
        right = right.to_frame()
    if how == "cross":
        data = {}
        lc, rc = list(left._data), list(right._data)
        nl, nr = len(left), len(right)
        for c in lc:
            name = c + suffixes[0] if c in right._data else c
            data[name] = [left._data[c][i] for i in range(nl) for _ in range(nr)]
        for c in rc:
            name = c + suffixes[1] if c in left._data else c
            data[name] = [right._data[c][j] for _ in range(nl) for j in range(nr)]
        return DataFrame._new(data, {}, default_index(nl * nr))
    if on is None and left_on is None and right_on is None and not left_index and not right_index:
        on = [c for c in left._data if c in right._data]
        if not on:
            raise ValueError("No common columns to perform merge on.")
    if on is not None:
        left_on = right_on = on if isinstance(on, list) else [on]
    if left_on is not None and not isinstance(left_on, list):
        left_on = [left_on]
    if right_on is not None and not isinstance(right_on, list):
        right_on = [right_on]
    la = _key_arrays(left, left_on, left_index)
    ra = _key_arrays(right, right_on, right_index)
    nl, nr = len(left), len(right)
    lkeys = [tuple(_hkey(a[i]) for a in la) for i in range(nl)]
    rkeys = [tuple(_hkey(a[i]) for a in ra) for i in range(nr)]
    rmap = {}
    for j, k in enumerate(rkeys):
        rmap.setdefault(k, []).append(j)
    pairs = []
    if how in ("inner", "left"):
        for i, k in enumerate(lkeys):
            js = rmap.get(k) if not any(x == ("__nan__",) for x in k) or True else None
            if js:
                for j in js:
                    pairs.append((i, j))
            elif how == "left":
                pairs.append((i, None))
    elif how == "right":
        lmap = {}
        for i, k in enumerate(lkeys):
            lmap.setdefault(k, []).append(i)
        for j, k in enumerate(rkeys):
            is_ = lmap.get(k)
            if is_:
                for i in is_:
                    pairs.append((i, j))
            else:
                pairs.append((None, j))
    elif how == "outer":
        lmap = {}
        korder = []
        kval = {}
        for i, k in enumerate(lkeys):
            if k not in lmap:
                lmap[k] = []
                korder.append(k)
                kval[k] = tuple(a[i] for a in la)
            lmap[k].append(i)
        for j, k in enumerate(rkeys):
            if k not in lmap and k not in kval:
                korder.append(k)
                kval[k] = tuple(a[j] for a in ra)
        try:
            korder.sort(key=lambda k: _sort_key(kval[k]))
        except TypeError:
            pass
        for k in korder:
            is_ = lmap.get(k, [None])
            js = rmap.get(k, [None])
            for i in is_:
                for j in js:
                    pairs.append((i, j))
    else:
        raise ValueError("how must be one of inner, left, right, outer, cross")
    if sort and how in ("inner", "left"):
        pairs.sort(key=lambda p: _sort_key(tuple(a[p[0]] for a in la)) if p[0] is not None else (3,))
    data = {}
    dts = {}
    key_same = (left_on == right_on) and not left_index and not right_index
    if key_same:
        for li, c in enumerate(left_on):
            vals = []
            for i, j in pairs:
                vals.append(la[li][i] if i is not None else ra[li][j])
            data[c] = vals
    lcols = [c for c in left._data if not (key_same and c in left_on)]
    rcols = [c for c in right._data if not (key_same and c in right_on)]
    lset, rset = set(lcols), set(rcols)
    for c in lcols:
        name = (str(c) + suffixes[0]) if c in rset and suffixes[0] is not None and (left_on is None or c not in left_on or c not in (right_on or [])) else c
        if c in rset and suffixes[0] is not None and not (left_on and right_on and c in left_on and c in right_on and key_same):
            name = str(c) + suffixes[0]
        v = left._data[c]
        na = _na_for(left._dtypes[c])
        data[name] = [v[i] if i is not None else na for i, j in pairs]
    for c in rcols:
        name = c
        if c in lset and suffixes[1] is not None:
            name = str(c) + suffixes[1]
        v = right._data[c]
        na = _na_for(right._dtypes[c])
        data[name] = [v[j] if j is not None else na for i, j in pairs]
    if key_same:
        ordered = {}
        for c in left._data:
            if c in left_on:
                ordered[c] = data[c]
            elif c in rset:
                ordered[str(c) + suffixes[0]] = data[str(c) + suffixes[0]]
            else:
                ordered[c] = data[c]
        for k, v in data.items():
            if k not in ordered:
                ordered[k] = v
        data = ordered
    if indicator:
        data["_merge"] = ["both" if i is not None and j is not None else ("left_only" if j is None else "right_only") for i, j in pairs]
    if left_index and right_index:
        idx = Index._new([left.index._values[i] if i is not None else right.index._values[j] for i, j in pairs], left.index._names)
    elif left_index and not right_index:
        idx = Index._new([right.index._values[j] if j is not None else NaN for i, j in pairs], right.index._names)
    elif right_index and not left_index:
        idx = Index._new([left.index._values[i] if i is not None else NaN for i, j in pairs], left.index._names)
    else:
        idx = default_index(len(pairs))
    for k in list(data.keys()):
        src = None
        if k in left._dtypes:
            src = left._dtypes[k]
        if k in right._dtypes and src is None:
            src = right._dtypes[k]
        if src is not None and src == "int64" and not any(isna_scalar(v) for v in data[k]):
            dts[k] = DT_INT
        else:
            dts[k] = None
    return DataFrame._new(data, dts, idx)


def concat(objs, axis=0, join="outer", ignore_index=False, keys=None, sort=False, names=None):
    from ._frame import DataFrame
    if isinstance(objs, dict):
        keys = list(objs.keys())
        objs = list(objs.values())
    objs = [o for o in objs if o is not None]
    if not objs:
        raise ValueError("No objects to concatenate")
    if axis in (1, "columns"):
        frames = [o.to_frame() if isinstance(o, Series) else o for o in objs]
        idx = frames[0].index
        for f in frames[1:]:
            if not f.index.equals(idx):
                idx = idx.union(f.index) if join == "outer" else idx.intersection(f.index)
        data = {}
        for n, f in enumerate(frames):
            g = f.reindex(idx) if not f.index.equals(idx) else f
            for c in g._data:
                key = (keys[n], c) if keys is not None else c
                data[key] = g._data[c]
        out = DataFrame._new(data, {}, idx)
        if ignore_index:
            out.columns = list(range(len(data)))
        return out
    if all(isinstance(o, Series) for o in objs):
        vals = []
        labels = []
        for n, o in enumerate(objs):
            vals.extend(o._values)
            if keys is not None:
                labels.extend((keys[n], l) for l in o.index._values)
            else:
                labels.extend(o.index._values)
        idx = default_index(len(vals)) if ignore_index else Index._new(labels, ([None] + list(objs[0].index._names)) if keys is not None else objs[0].index._names)
        name = objs[0].name if all(o.name == objs[0].name for o in objs) else None
        return Series._new(vals, idx, name)
    frames = [o.to_frame() if isinstance(o, Series) else o for o in objs]
    cols = []
    seen = set()
    for f in frames:
        for c in f._data:
            if c not in seen:
                seen.add(c)
                cols.append(c)
    if join == "inner":
        cols = [c for c in cols if all(c in f._data for f in frames)]
    if sort:
        cols = sorted(cols, key=_sort_key)
    data = {c: [] for c in cols}
    dts = {}
    labels = []
    for n, f in enumerate(frames):
        m = len(f)
        for c in cols:
            if c in f._data:
                data[c].extend(f._data[c])
            else:
                data[c].extend([NaN] * m)
        if keys is not None:
            labels.extend((keys[n], l) for l in f.index._values)
        else:
            labels.extend(f.index._values)
    for c in cols:
        kinds = set(str(f._dtypes[c]) for f in frames if c in f._data)
        if all(c in f._data for f in frames) and len(kinds) == 1:
            dts[c] = frames[0]._dtypes[c] if c in frames[0]._dtypes else None
        else:
            dts[c] = None
    if ignore_index:
        idx = default_index(len(labels))
    else:
        nms = ([None] + list(frames[0].index._names)) if keys is not None else frames[0].index._names
        if all(isinstance(f.index, RangeIndex) for f in frames) and keys is None:
            idx = Index._new(labels, nms, DT_INT)
        else:
            idx = Index._new(labels, nms)
    return DataFrame._new(data, dts, idx, list(frames[0]._columns_names))


def pivot_table(df, values=None, index=None, columns=None, aggfunc="mean", fill_value=None, margins=False, dropna=True, margins_name="All", sort=True):
    from ._frame import DataFrame
    from ._groupby import _Grouper, _apply_how
    idx_keys = [] if index is None else (index if isinstance(index, list) else [index])
    col_keys = [] if columns is None else (columns if isinstance(columns, list) else [columns])
    if values is None:
        vals_cols = [c for c in df._data if c not in idx_keys and c not in col_keys and is_numeric(df._dtypes[c])]
        values_given_single = False
    elif isinstance(values, list):
        vals_cols = values
        values_given_single = False
    else:
        vals_cols = [values]
        values_given_single = True
    funcs = aggfunc if isinstance(aggfunc, (list, tuple)) else None
    func_map = aggfunc if isinstance(aggfunc, dict) else None
    row_g = _Grouper(df, idx_keys, sort=sort) if idx_keys else None
    col_g = _Grouper(df, col_keys, sort=sort) if col_keys else None
    n = len(df)
    rkeys = row_g.order if row_g else [None]
    ckeys = col_g.order if col_g else [None]
    rid = {}
    if row_g:
        for k in row_g.order:
            for i in row_g.groups[k]:
                rid[i] = k
    cid = {}
    if col_g:
        for k in col_g.order:
            for i in col_g.groups[k]:
                cid[i] = k
    cells = {}
    for i in range(n):
        if row_g and i not in rid:
            continue
        if col_g and i not in cid:
            continue
        cells.setdefault((rid.get(i), cid.get(i)), []).append(i)
    data = {}
    dts = {}
    flist = []
    for v in vals_cols:
        fs = func_map.get(v, "mean") if func_map else (funcs if funcs else [aggfunc])
        fs = fs if isinstance(fs, (list, tuple)) else [fs]
        for f in fs:
            flist.append((v, f))
    multi_func = funcs is not None or (func_map is not None and any(isinstance(x, (list, tuple)) for x in func_map.values()))
    for v, f in flist:
        how = _agg.resolve(f)
        col = df._data[v]
        dt = df._dtypes[v]
        for ck in ckeys:
            out = []
            for rk in rkeys:
                pos = cells.get((rk, ck))
                if not pos:
                    out.append(NaN if fill_value is None else fill_value)
                else:
                    out.append(_apply_how([col[i] for i in pos], how, dt, {}))
            if ckeys == [None]:
                key = (_agg.func_name(f), v) if multi_func else v
            else:
                cval = col_g.keys[ck]
                cv = cval if isinstance(cval, tuple) else (cval,)
                prefix = ((_agg.func_name(f),) if multi_func else ()) + ((v,) if not values_given_single or multi_func else ())
                key = prefix + cv
                if len(key) == 1:
                    key = key[0]
            if dropna and all(isna_scalar(x) for x in out) and ckeys != [None]:
                continue
            data[key] = out
            dts[key] = None
    ridx = row_g.result_index() if row_g else Index(["" if not vals_cols else vals_cols[0]])
    out = DataFrame._new(data, dts, ridx)
    if ckeys != [None]:
        nlev = len(next(iter(data))) if data and isinstance(next(iter(data)), tuple) else 1
        if nlev > 1:
            out._columns_names = [None] * (nlev - len(col_keys)) + col_keys
        else:
            out._columns_names = [col_keys[0]]
    if margins and row_g:
        totals = {}
        for key in data:
            totals[key] = NaN
        for v, f in flist:
            how = _agg.resolve(f)
            col = df._data[v]
            dt = df._dtypes[v]
            for ck in ckeys:
                pos = [i for (rk, c), ps in cells.items() if c == ck for i in ps]
                if ckeys == [None]:
                    key = (_agg.func_name(f), v) if multi_func else v
                else:
                    cval = col_g.keys[ck]
                    cv = cval if isinstance(cval, tuple) else (cval,)
                    prefix = ((_agg.func_name(f),) if multi_func else ()) + ((v,) if not values_given_single or multi_func else ())
                    key = prefix + cv
                    if len(key) == 1:
                        key = key[0]
                if key in data:
                    totals[key] = _apply_how([col[i] for i in pos], how, dt, {})
            if ckeys != [None]:
                allpos = [i for ps in cells.values() for i in ps]
                key = (((_agg.func_name(f),) if multi_func else ()) + ((v,) if not values_given_single or multi_func else ()) + (margins_name,))
                key = key[0] if len(key) == 1 else key
                rowvals = []
                for rk in rkeys:
                    pos = [i for (r, c), ps in cells.items() if r == rk for i in ps]
                    rowvals.append(_apply_how([col[i] for i in pos], how, dt, {}))
                out._data[key] = rowvals
                out._dtypes[key] = infer_dtype(rowvals)
                totals[key] = _apply_how([col[i] for i in allpos], how, dt, {})
        lab = margins_name if len(idx_keys) == 1 else (margins_name,) + ("",) * (len(idx_keys) - 1)
        object.__setattr__(out, "_index", Index._new(out._index._values + [lab], out._index._names))
        for key in out._data:
            out._data[key] = list(out._data[key]) + [totals.get(key, NaN)]
            out._dtypes[key] = None
        out = DataFrame._new(out._data, out._dtypes, out._index, out._columns_names)
    return out


def pivot(df, index=None, columns=None, values=None):
    from ._frame import DataFrame
    if index is None:
        rows = df.index._values
        iname = df.index.name
    else:
        rows = df._data[index]
        iname = index
    cols = df._data[columns]
    vcols = [values] if isinstance(values, str) else (values if values is not None else [c for c in df._data if c not in (index, columns)])
    rorder, corder = [], []
    rs, cs = {}, {}
    for r in rows:
        if _hkey(r) not in rs:
            rs[_hkey(r)] = len(rorder)
            rorder.append(r)
    for c in cols:
        if _hkey(c) not in cs:
            cs[_hkey(c)] = len(corder)
            corder.append(c)
    try:
        rorder.sort(key=_sort_key)
        corder.sort(key=_sort_key)
    except TypeError:
        pass
    rpos = {_hkey(r): i for i, r in enumerate(rorder)}
    data = {}
    for v in vcols:
        grid = {_hkey(c): [NaN] * len(rorder) for c in corder}
        for i in range(len(df)):
            k = (_hkey(rows[i]), _hkey(cols[i]))
            grid[k[1]][rpos[k[0]]] = df._data[v][i]
        for c in corder:
            key = c if isinstance(values, str) else (v, c)
            data[key] = grid[_hkey(c)]
    out = DataFrame._new(data, {}, Index._new(rorder, [iname]))
    out._columns_names = [columns] if isinstance(values, str) else [None, columns]
    return out


def crosstab(index, columns, values=None, aggfunc=None, normalize=False, margins=False, margins_name="All", rownames=None, colnames=None):
    from ._frame import DataFrame
    iname = getattr(index, "name", None) or "row_0"
    cname = getattr(columns, "name", None) or "col_0"
    data = {iname: list(index), cname: list(columns)}
    if values is not None:
        data["__v"] = list(values)
        out = pivot_table(DataFrame(data), values="__v", index=iname, columns=cname, aggfunc=aggfunc or "mean", margins=margins, margins_name=margins_name)
    else:
        data["__v"] = [1] * len(data[iname])
        out = pivot_table(DataFrame(data), values="__v", index=iname, columns=cname, aggfunc="count", fill_value=0, margins=margins, margins_name=margins_name)
    if normalize:
        if normalize is True or normalize == "all":
            tot = sum(v for c in out._data for v in out._data[c] if not isna_scalar(v))
            for c in out._data:
                out._data[c] = [v / tot for v in out._data[c]]
                out._dtypes[c] = DT_FLOAT
        elif normalize in ("index", 0):
            n = len(out)
            sums = [sum(out._data[c][i] for c in out._data) for i in range(n)]
            for c in out._data:
                out._data[c] = [v / s if s else NaN for v, s in zip(out._data[c], sums)]
                out._dtypes[c] = DT_FLOAT
        elif normalize in ("columns", 1):
            for c in out._data:
                s = sum(out._data[c])
                out._data[c] = [v / s if s else NaN for v in out._data[c]]
                out._dtypes[c] = DT_FLOAT
    return out


def cut(x, bins, labels=None, right=True, include_lowest=False, precision=3, retbins=False):
    vals = list(x._values) if isinstance(x, Series) else list(x)
    if isinstance(bins, int):
        xs = [v for v in vals if not isna_scalar(v)]
        lo, hi = min(xs), max(xs)
        if lo == hi:
            lo, hi = lo - 0.001 * abs(lo or 1), hi + 0.001 * abs(hi or 1)
        step = (hi - lo) / bins
        edges = [lo + step * i for i in range(bins + 1)]
        adj = (hi - lo) * 0.001
        if right:
            edges[0] = lo - adj
        else:
            edges[-1] = hi + adj
    else:
        edges = list(bins)
    if labels is None:
        names = []
        for i in range(len(edges) - 1):
            a, b = _fmt_edge(edges[i], precision), _fmt_edge(edges[i + 1], precision)
            if right:
                names.append("(%s, %s]" % (a, b) if not (include_lowest and i == 0) else "(%s, %s]" % (_fmt_edge(edges[i] - 0.001 if False else edges[i], precision), b))
            else:
                names.append("[%s, %s)" % (a, b))
    elif labels is False:
        names = list(range(len(edges) - 1))
    else:
        names = list(labels)
    out = []
    for v in vals:
        if isna_scalar(v):
            out.append(NaN)
            continue
        hit = NaN
        for i in range(len(edges) - 1):
            a, b = edges[i], edges[i + 1]
            if right:
                ok = (a < v <= b) or (include_lowest and i == 0 and v == a)
            else:
                ok = (a <= v < b) or (i == len(edges) - 2 and v == b and False)
            if ok:
                hit = names[i]
                break
        out.append(hit)
    if isinstance(x, Series):
        res = Series._new(out, x.index, x.name)
    else:
        res = Series._new(out, default_index(len(out)), None)
    if retbins:
        return res, edges
    return res


def qcut(x, q, labels=None, precision=3):
    vals = [v for v in (x._values if isinstance(x, Series) else x) if not isna_scalar(v)]
    qs = [i / q for i in range(q + 1)] if isinstance(q, int) else list(q)
    edges = [_agg.quantile(vals, p) for p in qs]
    return cut(x, edges, labels=labels, include_lowest=True, precision=precision)


def _fmt_edge(v, precision):
    if isinstance(v, int):
        return str(v)
    s = ("%." + str(precision) + "f") % v
    s = s.rstrip("0")
    if s.endswith("."):
        s += "0"
    return s


def get_dummies(data, columns=None, prefix=None, prefix_sep="_", drop_first=False, dtype=None):
    from ._frame import DataFrame
    if isinstance(data, Series):
        cats = sorted(set(v for v in data._values if not isna_scalar(v)), key=_sort_key)
        if drop_first:
            cats = cats[1:]
        pre = (prefix + prefix_sep) if prefix else ""
        return DataFrame({pre + str(c): [v == c for v in data._values] for c in cats}, index=data.index)
    cols = columns if columns is not None else [c for c in data._data if data._dtypes[c] == "object"]
    out = {}
    for c in data._data:
        if c not in cols:
            out[c] = data._data[c]
    for c in cols:
        cats = sorted(set(v for v in data._data[c] if not isna_scalar(v)), key=_sort_key)
        if drop_first:
            cats = cats[1:]
        pre = prefix.get(c, c) if isinstance(prefix, dict) else (prefix if prefix else c)
        for k in cats:
            out["%s%s%s" % (pre, prefix_sep, k)] = [v == k for v in data._data[c]]
    return DataFrame._new(out, {}, data.index)
