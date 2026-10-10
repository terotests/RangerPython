# SPDX-License-Identifier: MIT
# heapq for the Ranger Python runtime: a binary min-heap on a list

__all__ = ["heappush", "heappop", "heapify", "heapreplace", "merge", "nlargest", "nsmallest", "heappushpop"]


def heappush(heap, item):
    heap.append(item)
    _siftdown(heap, 0, len(heap) - 1)


def heappop(heap):
    last = heap.pop()
    if heap:
        item = heap[0]
        heap[0] = last
        _siftup(heap, 0)
        return item
    return last


def heapreplace(heap, item):
    returnitem = heap[0]
    heap[0] = item
    _siftup(heap, 0)
    return returnitem


def heappushpop(heap, item):
    if heap and heap[0] < item:
        item, heap[0] = heap[0], item
        _siftup(heap, 0)
    return item


def heapify(x):
    n = len(x)
    for i in reversed(range(n // 2)):
        _siftup(x, i)


def _siftdown(heap, startpos, pos):
    newitem = heap[pos]
    while pos > startpos:
        parentpos = (pos - 1) >> 1
        parent = heap[parentpos]
        if newitem < parent:
            heap[pos] = parent
            pos = parentpos
            continue
        break
    heap[pos] = newitem


def _siftup(heap, pos):
    endpos = len(heap)
    startpos = pos
    newitem = heap[pos]
    childpos = 2 * pos + 1
    while childpos < endpos:
        rightpos = childpos + 1
        if rightpos < endpos and not heap[childpos] < heap[rightpos]:
            childpos = rightpos
        heap[pos] = heap[childpos]
        pos = childpos
        childpos = 2 * pos + 1
    heap[pos] = newitem
    _siftdown(heap, startpos, pos)


def _heappop_max(heap):
    last = heap.pop()
    if heap:
        item = heap[0]
        heap[0] = last
        _siftup_max(heap, 0)
        return item
    return last


def _heapify_max(x):
    for i in reversed(range(len(x) // 2)):
        _siftup_max(x, i)


def _siftdown_max(heap, startpos, pos):
    newitem = heap[pos]
    while pos > startpos:
        parentpos = (pos - 1) >> 1
        parent = heap[parentpos]
        if parent < newitem:
            heap[pos] = parent
            pos = parentpos
            continue
        break
    heap[pos] = newitem


def _siftup_max(heap, pos):
    endpos = len(heap)
    startpos = pos
    newitem = heap[pos]
    childpos = 2 * pos + 1
    while childpos < endpos:
        rightpos = childpos + 1
        if rightpos < endpos and not heap[rightpos] < heap[childpos]:
            childpos = rightpos
        heap[pos] = heap[childpos]
        pos = childpos
        childpos = 2 * pos + 1
    heap[pos] = newitem
    _siftdown_max(heap, startpos, pos)


def merge(*iterables, key=None, reverse=False):
    items = []
    for it in iterables:
        items.extend(it)
    return iter(sorted(items, key=key, reverse=reverse))


def nsmallest(n, iterable, key=None):
    return sorted(iterable, key=key)[:n]


def nlargest(n, iterable, key=None):
    return sorted(iterable, key=key, reverse=True)[:n]
