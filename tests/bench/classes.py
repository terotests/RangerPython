class Vec:
    __slots__ = ("x", "y")
    def __init__(self, x, y):
        self.x = x
        self.y = y
    def __add__(self, o):
        return Vec(self.x + o.x, self.y + o.y)
    def dot(self, o):
        return self.x * o.x + self.y * o.y
acc = Vec(0, 0)
t = 0
for i in range(200000):
    v = Vec(i, -i)
    acc = acc + v
    t += v.dot(acc) % 7
print(acc.x, acc.y, t)
