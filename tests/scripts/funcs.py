def greet(name, greeting="Hello", *args, sep=", ", **kw):
    return greeting + sep + name + str(args) + str(sorted(kw.items()))
print(greet("A"), greet("B", "Hi", 1, 2, sep="; ", z=1, y=2))
def counter():
    n = 0
    def inc():
        nonlocal n
        n += 1
        return n
    return inc
c = counter()
c(); c()
print(c())
total = 0
def add(x):
    global total
    total += x
add(5); add(6)
print(total)
def deco(fn):
    def wrapper(*a, **k):
        return "<" + str(fn(*a, **k)) + ">"
    return wrapper
@deco
def sq(x):
    return x * x
print(sq(7))
def fib():
    a, b = 0, 1
    while True:
        yield a
        a, b = b, a + b
        if a > 100:
            return
print(list(fib()))
def gen2():
    yield from range(3)
    yield from "ab"
print(list(gen2()), sum(x * x for x in range(10)))
fact = lambda n: 1 if n <= 1 else n * fact(n - 1)
print(fact(10), (lambda *a: a)(1, 2))
print(sorted(["bb", "a", "ccc"], key=len), max(["bb", "a", "ccc"], key=len))
def kwonly(a, *, b=2):
    return a + b
print(kwonly(1, b=5), kwonly(1))
matrix = [[1, 2, 3], [4, 5, 6]]
print([list(r) for r in zip(*matrix)], [[x * 2 for x in row] for row in matrix])
print({x % 3 for x in range(10)}, [(i, j) for i in range(3) for j in range(i)])
def rec(n):
    return 0 if n == 0 else 1 + rec(n - 1)
print(rec(150))
def f(*, x): return x
print(f(x=3))
