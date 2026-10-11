// SPDX-License-Identifier: MIT
// The scripts in the Examples menu: { name: { "main.py": …, other files } }
export const examples = {
  "Hello": {
    "main.py": `import sys

print("Hello from Python", sys.version.split()[0])

for n in range(1, 16):
    print("FizzBuzz" if n % 15 == 0 else "Fizz" if n % 3 == 0 else "Buzz" if n % 5 == 0 else n)
`,
  },

  "CSV to an SVG chart": {
    "main.py": `import csv
from statistics import mean

with open("data/sales.csv") as f:
    rows = [(r["month"], float(r["sales"])) for r in csv.DictReader(f)]

print(f"{len(rows)} months, average {mean(v for _, v in rows):.1f}")

w, h, pad = 480, 240, 30
top = max(v for _, v in rows)
bw = (w - 2 * pad) / len(rows)
bars = []
for i, (month, v) in enumerate(rows):
    bh = (h - 2 * pad) * v / top
    x, y = pad + i * bw, h - pad - bh
    bars.append(f'<rect x="{x + 4:.0f}" y="{y:.0f}" width="{bw - 8:.0f}" height="{bh:.0f}" fill="#4f7cff"/>'
                f'<text x="{x + bw / 2:.0f}" y="{h - 10}" font-size="12" text-anchor="middle">{month}</text>')

svg = f'<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" font-family="sans-serif">{"".join(bars)}</svg>'
with open("/tmp/chart.svg", "w") as f:
    f.write(svg)
print("wrote /tmp/chart.svg")
`,
    "data/sales.csv": `month,sales
Jan,120
Feb,98
Mar,143
Apr,170
May,155
Jun,190
`,
  },

  "Classes and dataclasses": {
    "main.py": `from dataclasses import dataclass, field
from functools import total_ordering

@total_ordering
@dataclass
class Card:
    rank: int
    suit: str

    def __lt__(self, other):
        return self.rank < other.rank

@dataclass
class Hand:
    cards: list = field(default_factory=list)

    def add(self, card):
        self.cards.append(card)
        return self

    @property
    def best(self):
        return max(self.cards)

hand = Hand().add(Card(10, "hearts")).add(Card(14, "spades")).add(Card(3, "clubs"))
print(hand.best)
print(sorted(hand.cards))
`,
  },

  "Generators and itertools": {
    "main.py": `from itertools import islice, takewhile, accumulate

def primes():
    found = []
    n = 2
    while True:
        if all(n % p for p in takewhile(lambda p: p * p <= n, found)):
            found.append(n)
            yield n
        n += 1

print(list(islice(primes(), 20)))
print(list(accumulate(range(1, 11))))

def averager():
    total = count = 0
    avg = None
    while True:
        x = yield avg
        total += x
        count += 1
        avg = total / count

a = averager(); next(a)
print([a.send(x) for x in (10, 20, 60)])
`,
  },

  "Big integers and fractions": {
    "main.py": `from fractions import Fraction
from decimal import Decimal, getcontext
import math

print(2 ** 200)
print(math.factorial(40))
print(sum(Fraction(1, n) for n in range(1, 11)))

getcontext().prec = 40
print(Decimal(2).sqrt())
`,
  },

  "match and JSON": {
    "main.py": `import json

events = json.loads(open("events.json").read())

def describe(e):
    match e:
        case {"type": "click", "x": x, "y": y}:
            return f"click at {x},{y}"
        case {"type": "key", "key": str(k)} if len(k) == 1:
            return f"typed {k!r}"
        case {"type": t}:
            return f"other event: {t}"

for e in events:
    print(describe(e))

summary = {"count": len(events), "types": sorted({e["type"] for e in events})}
with open("/tmp/summary.json", "w") as f:
    json.dump(summary, f, indent=2)
print(json.dumps(summary))
`,
    "events.json": `[
  {"type": "click", "x": 10, "y": 20},
  {"type": "key", "key": "a"},
  {"type": "scroll", "dy": -3}
]
`,
  },

  "Modules of your own": {
    "main.py": `from geometry import Vec

a, b = Vec(1, 2), Vec(3, 4)
print(a + b, abs(b), a * 3)
`,
    "geometry.py": `import math

class Vec:
    def __init__(self, x, y):
        self.x, self.y = x, y

    def __add__(self, o):
        return Vec(self.x + o.x, self.y + o.y)

    def __mul__(self, k):
        return Vec(self.x * k, self.y * k)

    def __abs__(self):
        return math.hypot(self.x, self.y)

    def __repr__(self):
        return f"Vec({self.x}, {self.y})"
`,
  },

  "Exceptions": {
    "main.py": `class InsufficientFunds(Exception):
    pass

def withdraw(balance, amount):
    if amount > balance:
        raise InsufficientFunds(f"need {amount - balance} more")
    return balance - amount

try:
    withdraw(10, 25)
except InsufficientFunds as e:
    print("caught:", e)

try:
    raise ExceptionGroup("several", [ValueError(1), TypeError(2)])
except* ValueError as g:
    print("values:", g.exceptions)
except* TypeError as g:
    print("types:", g.exceptions)

# an uncaught one ends the script with a traceback
withdraw(5, 7)
`,
  },
};
