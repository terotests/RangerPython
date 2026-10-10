d = {}
for i in range(300000):
    k = "k%d" % (i % 997)
    d[k] = d.get(k, 0) + i
total = 0
for k, v in sorted(d.items()):
    total += v % 1000
s = set()
for i in range(300000):
    s.add(i * 7 % 1013)
print(len(d), total, len(s))
