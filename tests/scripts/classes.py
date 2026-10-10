class Animal:
    count = 0
    def __init__(self, name):
        self.name = name
        Animal.count += 1
    def speak(self):
        return f"{self.name} makes a sound"
    def __repr__(self):
        return f"Animal({self.name!r})"
    def __str__(self):
        return self.name

class Dog(Animal):
    def speak(self):
        return super().speak() + " (woof)"

class Vec:
    __slots__ = ("x", "y")
    def __init__(self, x, y):
        self.x, self.y = x, y
    def __add__(self, o):
        return Vec(self.x + o.x, self.y + o.y)
    def __mul__(self, k):
        return Vec(self.x * k, self.y * k)
    __rmul__ = __mul__
    def __eq__(self, o):
        return isinstance(o, Vec) and (self.x, self.y) == (o.x, o.y)
    def __hash__(self):
        return hash((self.x, self.y))
    def __len__(self):
        return 2
    def __getitem__(self, i):
        return (self.x, self.y)[i]
    def __iter__(self):
        yield self.x
        yield self.y
    def __lt__(self, o):
        return (self.x, self.y) < (o.x, o.y)
    def __repr__(self):
        return f"Vec({self.x}, {self.y})"

class Temp:
    def __init__(self):
        self._c = 0
    @property
    def c(self):
        return self._c
    @c.setter
    def c(self, v):
        if v < -273:
            raise ValueError("too cold")
        self._c = v
    @staticmethod
    def unit():
        return "C"
    @classmethod
    def make(cls, v):
        t = cls()
        t.c = v
        return t

a = Animal("cat")
d = Dog("rex")
print(a.speak(), d.speak(), repr(a), str(d), Animal.count, isinstance(d, Animal), issubclass(Dog, Animal))
v = Vec(1, 2) + Vec(3, 4)
print(v, v * 2, 3 * v, v == Vec(4, 6), len(v), v[1], list(v), sorted([Vec(2, 1), Vec(1, 5)]))
print({Vec(1, 1): "a"}[Vec(1, 1)], max(Vec(0, 9), Vec(1, 0)))
t = Temp.make(25)
print(t.c, Temp.unit())
try:
    t.c = -300
except ValueError as e:
    print("error:", e)
print(Dog.__mro__ if False else [c.__name__ for c in Dog.__mro__])

class Stack:
    def __init__(self):
        self.items = []
    def push(self, x):
        self.items.append(x)
        return self
    def __bool__(self):
        return bool(self.items)
    def __contains__(self, x):
        return x in self.items
s = Stack().push(1).push(2)
print(bool(s), 2 in s, 5 in s, bool(Stack()))
print(getattr(a, "name"), hasattr(a, "zzz"), getattr(a, "zzz", "dflt"))
setattr(a, "age", 3)
print(a.age, vars(a))

class Meta:
    def __call__(self, x):
        return x * 2
print(Meta()(21))
