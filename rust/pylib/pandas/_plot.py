# SPDX-License-Identifier: MIT
# df.plot / s.plot for rpy's pandas. There is no matplotlib here; the
# accessor says what to use instead so a script fails with a clear message.


class PlotAccessor:
    def __init__(self, obj):
        self._obj = obj

    def _fail(self, *a, **k):
        raise NotImplementedError(
            "plotting is not available in this runtime (no matplotlib); "
            "write the chart as SVG, or save the data with to_csv/to_json for the slide's chart")

    __call__ = line = bar = barh = pie = scatter = hist = area = box = kde = _fail
