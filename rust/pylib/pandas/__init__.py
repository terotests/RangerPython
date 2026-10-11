# SPDX-License-Identifier: MIT
# pandas for rpy: a pure-Python DataFrame and Series with the part of the
# pandas API that analysis scripts use most (read_csv, selection, groupby,
# merge, pivot_table, describe, value_counts...). Columns are Python lists;
# a missing value is float("nan") (NaT in a datetime column).

from ._util import NA, NaN, nan, NaT, get_option, set_option, option_context, OPTIONS as options, dtype as _dtype
from ._index import Index, RangeIndex, MultiIndex
from ._series import Series
from ._frame import DataFrame
from ._core import (isna, isnull, notna, notnull, to_numeric, to_datetime,
                    Timestamp, Timedelta, date_range, period_range, unique,
                    factorize, melt, pivot, NamedAgg, Grouper, read_excel,
                    read_parquet, api)
from ._reshape import concat, merge, cut, qcut, crosstab, get_dummies, pivot_table
from ._io import read_csv, read_json, read_table, json_normalize

__version__ = "2.2.0+rpy"

CategoricalDtype = None


def show_versions():
    print("pandas: %s (rpy pure-Python subset)" % __version__)
