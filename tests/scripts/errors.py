def div(a, b):
    return a / b
for args in [(1, 2), (1, 0), ("a", 1)]:
    try:
        print(div(*args))
    except ZeroDivisionError as e:
        print("zero:", e)
    except TypeError as e:
        print("type:", e)
    finally:
        print("finally", args)

class MyError(Exception):
    def __init__(self, code):
        super().__init__(f"code {code}")
        self.code = code
try:
    raise MyError(7)
except Exception as e:
    print(type(e).__name__, e, e.code, e.args)

def f():
    try:
        return "try"
    finally:
        print("cleanup")
print(f())
try:
    [1, 2][5]
except IndexError as e:
    print("IndexError:", e)
try:
    {}["k"]
except KeyError as e:
    print("KeyError:", e, repr(e))
try:
    int("x")
except ValueError as e:
    print(e)
try:
    undefined_name
except NameError as e:
    print(e)
try:
    None.foo
except AttributeError as e:
    print(e)
try:
    try:
        raise ValueError("inner")
    except ValueError:
        raise RuntimeError("outer")
except RuntimeError as e:
    print(e, type(e.__context__).__name__ if hasattr(e, "__context__") and e.__context__ else "no-context")
try:
    assert 1 == 2, "nope"
except AssertionError as e:
    print("assert:", e)
def gen():
    yield 1
    raise StopIteration
it = iter([1, 2])
print(next(it), next(it), next(it, "done"))
try:
    raise KeyError("a", "b")
except (KeyError, IndexError) as e:
    print(e.args)
else:
    print("no")
try:
    pass
except Exception:
    pass
else:
    print("else branch")
