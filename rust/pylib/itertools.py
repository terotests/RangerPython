# SPDX-License-Identifier: MIT
# itertools for the Rust runtime, following the CPython documentation's equivalents.


class count:
    def __init__(self, start=0, step=1):
        self._n = start
        self._step = step

    def __iter__(self):
        return self

    def __next__(self):
        n = self._n
        self._n = n + self._step
        return n

    def __repr__(self):
        if self._step == 1:
            return f"count({self._n!r})"
        return f"count({self._n!r}, {self._step!r})"


class repeat:
    def __init__(self, object, times=None):
        self._obj = object
        self._times = times

    def __iter__(self):
        return self

    def __next__(self):
        if self._times is not None:
            if self._times <= 0:
                raise StopIteration
            self._times -= 1
        return self._obj

    def __length_hint__(self):
        return 0 if self._times is None else max(self._times, 0)

    def __repr__(self):
        if self._times is None:
            return f"repeat({self._obj!r})"
        return f"repeat({self._obj!r}, {self._times})"


def cycle(iterable):
    saved = []
    for element in iterable:
        yield element
        saved.append(element)
    if not saved:
        return
    while True:
        for element in saved:
            yield element


class chain:
    def __init__(self, *iterables):
        self._its = iter(iterables)
        self._cur = None

    @classmethod
    def from_iterable(cls, iterables):
        c = cls()
        c._its = iter(iterables)
        return c

    def __iter__(self):
        return self

    def __next__(self):
        while True:
            if self._cur is None:
                self._cur = iter(next(self._its))
            try:
                return next(self._cur)
            except StopIteration:
                self._cur = None


def accumulate(iterable, func=None, *, initial=None):
    it = iter(iterable)
    total = initial
    if initial is None:
        try:
            total = next(it)
        except StopIteration:
            return
    yield total
    if func is None:
        for element in it:
            total = total + element
            yield total
    else:
        for element in it:
            total = func(total, element)
            yield total


def batched(iterable, n, *, strict=False):
    if n < 1:
        raise ValueError("n must be at least one")
    it = iter(iterable)
    while True:
        batch = tuple(islice(it, n))
        if not batch:
            return
        if strict and len(batch) != n:
            raise ValueError("batched(): incomplete batch")
        yield batch


def compress(data, selectors):
    return (d for d, s in zip(data, selectors) if s)


def dropwhile(predicate, iterable):
    it = iter(iterable)
    for x in it:
        if not predicate(x):
            yield x
            break
    for x in it:
        yield x


def filterfalse(predicate, iterable):
    if predicate is None:
        predicate = bool
    for x in iterable:
        if not predicate(x):
            yield x


class groupby:
    def __init__(self, iterable, key=None):
        self.keyfunc = (lambda x: x) if key is None else key
        self.it = iter(iterable)
        self.tgtkey = self.currkey = self.currvalue = object()
        self._id = object()

    def __iter__(self):
        return self

    def __next__(self):
        self._id = object()
        while self.currkey == self.tgtkey:
            self.currvalue = next(self.it)
            self.currkey = self.keyfunc(self.currvalue)
        self.tgtkey = self.currkey
        return (self.currkey, self._grouper(self.tgtkey, self._id))

    def _grouper(self, tgtkey, id):
        while self._id is id and self.currkey == tgtkey:
            yield self.currvalue
            try:
                self.currvalue = next(self.it)
            except StopIteration:
                return
            self.currkey = self.keyfunc(self.currvalue)


def islice(iterable, *args):
    s = slice(*args)
    start, stop, step = s.start or 0, s.stop, s.step or 1
    if start < 0 or (stop is not None and stop < 0):
        raise ValueError("Indices for islice() must be None or an integer: 0 <= x <= sys.maxsize.")
    if step < 1:
        raise ValueError("Step for islice() must be a positive integer or None.")
    it = iter(iterable)
    i = 0
    nexti = start
    while stop is None or i < stop:
        try:
            x = next(it)
        except StopIteration:
            return
        if i == nexti:
            yield x
            nexti += step
        i += 1


def pairwise(iterable):
    it = iter(iterable)
    try:
        a = next(it)
    except StopIteration:
        return
    for b in it:
        yield a, b
        a = b


def starmap(function, iterable):
    for args in iterable:
        yield function(*args)


def takewhile(predicate, iterable):
    for x in iterable:
        if predicate(x):
            yield x
        else:
            break


def tee(iterable, n=2):
    if n < 0:
        raise ValueError("n must be >= 0")
    it = iter(iterable)
    bufs = [[] for _ in range(n)]

    def gen(mine):
        while True:
            if not mine:
                try:
                    v = next(it)
                except StopIteration:
                    return
                for b in bufs:
                    b.append(v)
            yield mine.pop(0)
    return tuple(gen(b) for b in bufs)


class _Sentinel:
    pass


def zip_longest(*args, fillvalue=None):
    iterators = [iter(it) for it in args]
    num_active = len(iterators)
    if not num_active:
        return
    while True:
        values = []
        for i, it in enumerate(iterators):
            try:
                value = next(it)
            except StopIteration:
                num_active -= 1
                if not num_active:
                    return
                iterators[i] = repeat(fillvalue)
                value = fillvalue
            values.append(value)
        yield tuple(values)


def product(*iterables, repeat=1):
    if repeat < 0:
        raise ValueError("repeat argument cannot be negative")
    pools = [tuple(pool) for pool in iterables] * repeat
    result = [[]]
    for pool in pools:
        result = [x + [y] for x in result for y in pool]
    for prod in result:
        yield tuple(prod)


def permutations(iterable, r=None):
    pool = tuple(iterable)
    n = len(pool)
    r = n if r is None else r
    if r > n:
        return
    indices = list(range(n))
    cycles = list(range(n, n - r, -1))
    yield tuple(pool[i] for i in indices[:r])
    while n:
        for i in reversed(range(r)):
            cycles[i] -= 1
            if cycles[i] == 0:
                indices[i:] = indices[i + 1:] + indices[i:i + 1]
                cycles[i] = n - i
            else:
                j = cycles[i]
                indices[i], indices[-j] = indices[-j], indices[i]
                yield tuple(pool[i] for i in indices[:r])
                break
        else:
            return


def combinations(iterable, r):
    pool = tuple(iterable)
    n = len(pool)
    if r < 0:
        raise ValueError("r must be non-negative")
    if r > n:
        return
    indices = list(range(r))
    yield tuple(pool[i] for i in indices)
    while True:
        for i in reversed(range(r)):
            if indices[i] != i + n - r:
                break
        else:
            return
        indices[i] += 1
        for j in range(i + 1, r):
            indices[j] = indices[j - 1] + 1
        yield tuple(pool[i] for i in indices)


def combinations_with_replacement(iterable, r):
    pool = tuple(iterable)
    n = len(pool)
    if r < 0:
        raise ValueError("r must be non-negative")
    if not n and r:
        return
    indices = [0] * r
    yield tuple(pool[i] for i in indices)
    while True:
        for i in reversed(range(r)):
            if indices[i] != n - 1:
                break
        else:
            return
        indices[i:] = [indices[i] + 1] * (r - i)
        yield tuple(pool[i] for i in indices)
