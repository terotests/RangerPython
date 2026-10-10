import sys
# strings and unicode
s = "Ääkkönen 😀 ok"
print(len(s), s.upper(), s[9], s.encode is not None, s.title(), s.swapcase())
print("tab\there", repr("line\nbreak"), "a\\b", r"raw\n", b"bytes" if False else "no-bytes")
print("x".join(sorted(set("hello"))), "Hello".ljust(8, ".") + "|", "abc".rjust(5), "  x ".lstrip(), "x  ".rstrip() + "|")
print("a b  c".split(), "a,b".partition(","), "k=v=w".rsplit("=", 1), "ABC".lower().capitalize(), "3".isdigit(), "a1".isalnum())
print("{0}-{1}-{0}".format("a", "b"), "{name}!".format(name="hi"), "%(x)s/%(y)d" % {"x": "p", "y": 3}, "%c%%" % 65)
print(f"{3.14159:.3}", f"{1234.5:,.1f}", f"{0.000123:g}", f"{123456789:g}", f"{-5:+d}", f"{42:*^9}", f"{'s':>3}")
print(str(1/3), 2/3, 1e300 * 10, -1e-310, 123456789.0, 1e22, 0.1 * 3, 100.0, 1.0e-4)
print(int("-0x1f", 16) if False else int("-1f", 16), int("0b101", 0), int(" 42 "), float(" 1.5 "), float("-inf"))
# sorting stability and keys
people = [("bob", 25), ("al", 30), ("cy", 25), ("di", 30)]
print(sorted(people, key=lambda p: p[1]), sorted(people, key=lambda p: (-p[1], p[0])))
people.sort(key=lambda p: p[0], reverse=True)
print(people)
# slicing assignment and deletion
a = list(range(10))
a[2:5] = ["x", "y"]
del a[0]
del a[-2:]
print(a, a[::-3])
d = {"a": 1, "b": 2, "c": 3}
del d["b"]
d.update({"z": 26}, q=1)
print(d, d.setdefault("a", 9), d.setdefault("n", 0), d)
# generators and iteration protocol
def squares(n):
    for i in range(n):
        yield i * i
g = squares(5)
print(next(g), next(g), list(g))
class Countdown:
    def __init__(self, n):
        self.n = n
    def __iter__(self):
        return self
    def __next__(self):
        if self.n <= 0:
            raise StopIteration
        self.n -= 1
        return self.n + 1
print(list(Countdown(4)), sum(Countdown(3)))
# with statement and context managers
class CM:
    def __enter__(self):
        print("enter")
        return "res"
    def __exit__(self, et, ev, tb):
        print("exit", et.__name__ if et else None)
        return True
with CM() as r:
    print("body", r)
    raise ValueError("swallowed")
print("after with")
# dataclass-like manual and __eq__ default identity
class P:
    pass
p1, p2 = P(), P()
print(p1 == p1, p1 == p2, p1 is not p2)
# chained comparison, ternary, augmented ops
x = 5
x += 2; x *= 3; x -= 1; x //= 2; x **= 2; x %= 7
print(x, 0 < x <= 7, "even" if x % 2 == 0 else "odd")
lst = [1]
lst += [2]
lst *= 2
print(lst)
# global builtins
print(abs(-2.5), round(3.14159, 3), pow(2, 10, 1000), sum([], 10), max([], default="none"), min("bca"))
print(list(range(10, 0, -3)), len(range(0, 100, 7)), 49 in range(0, 100, 7), range(5)[-1])
print(sys.argv[0] != "", sys.maxsize > 2**31)
print(sorted({3: "c", 1: "a"}.items()), {**{"a": 1}, "b": 2}, [*range(3), *"ab"])
def outer():
    v = []
    def inner(x):
        v.append(x)
        return len(v)
    return inner
f = outer()
print(f(1), f(2))
print(type(None).__name__, type(1.0).__name__, type(f).__name__, callable(f), callable(3))
print(hash("a") == hash("a"), id(f) == id(f), isinstance(f, object))
print(eval("1+2") if False else "no-eval")
print(chr(0x1F600), ord("😀"), "😀".encode is not None, ascii("é"))
