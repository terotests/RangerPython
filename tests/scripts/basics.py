# numbers, strings, containers
print(1 + 2 * 3, 7 // 2, -7 // 2, 7 % 3, -7 % 3, 2 ** 10, 2 ** -1, 10 / 4)
print(0.1 + 0.2, 1e16, 1.5e-5, 3.0, -0.0, float('inf'), round(2.675, 2), round(0.5), round(1.5))
print(int("42"), int("ff", 16), int(3.9), float("1.25"), str(12), repr("a'b"), repr('a"b'))
print(divmod(17, 5), abs(-3), max(3, 9, 2), min([4, 1, 7]), sum([1, 2, 3]), sum([0.1] * 10))
print(hex(255), oct(8), bin(5), chr(65), ord("é"), len("héllo"))
s = "Hello, World"
print(s.lower(), s.upper(), s.split(", "), s[::-1], s[2:5], s.find("o"), s.count("l"))
print(s.replace("l", "L", 2), s.startswith("He"), s.strip("Hd"), "-".join(["a", "b", "c"]))
print("  pad  ".strip() + "|", "x".center(7, "*"), "7".zfill(3), "a,b,,c".split(","))
print("%5.2f|%-4d|%s|%r|%x" % (3.14159, 42, "hi", "hi", 255))
print("{} {:>6} {:<6}| {:^7} {:08.3f} {:,} {:.2%} {:e}".format(1, "r", "l", "c", 3.14159, 1234567, 0.256, 12345.678))
x = 255
print(f"{x:#x} {x:b} {x!r} {x=} {'nested ' + str(x)} {x:>{6}}")
lst = [5, 3, 8, 1]
lst.append(9); lst.extend([2, 2]); lst.insert(0, 0); lst.remove(2)
print(lst, sorted(lst), sorted(lst, reverse=True), lst.index(8), lst.pop(), lst)
print(lst[1:4], lst[-2:], lst[::2], [i * i for i in range(6) if i % 2 == 0])
d = {"b": 2, "a": 1}
d["c"] = 3
print(d, list(d.keys()), list(d.values()), list(d.items()), d.get("z", 0), "a" in d)
print({k: v * 10 for k, v in d.items()}, dict(zip("xyz", range(3))), d.pop("b"), d)
st = {3, 1, 2}
print(sorted(st | {5}), sorted(st & {1, 9}), sorted(st - {1}), 2 in st, len(st))
t = (1, "two", 3.0)
a, b, c = t
first, *rest = [1, 2, 3, 4]
print(a, b, c, first, rest, t[1], t + (4,), t * 2 if False else t)
print(list(enumerate("ab", 1)), list(zip([1, 2], [3, 4])), list(map(str, [1, 2])), list(filter(None, [0, 1, 2])))
print(any([0, 0, 1]), all([1, 1, 0]), list(reversed([1, 2, 3])), sorted("banana"), sorted([("b", 2), ("a", 2), ("c", 1)], key=lambda p: p[1]))
print(isinstance(3, int), isinstance(True, int), isinstance("s", (int, str)), type(3).__name__, type([]))
n = 0
while n < 3:
    n += 1
else:
    print("while-else", n)
for i in range(3):
    if i == 5:
        break
else:
    print("for-else")
print((y := 10) + 1, y)
print(1 < 2 < 3, 1 < 2 > 3, None is None, [] == [], "a" * 3, [0] * 3)
print(str(None), bool(""), bool([1]), 3 if True else 4, not 0)
big = 2 ** 40 + 1
print(big, big * 3, big // 7, big % 1000)
print(round(-0.4), -0.0 * 1, 5 * -0.0)
