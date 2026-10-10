def a(i, j):
    return 1.0 / ((i + j) * (i + j + 1) // 2 + i + 1)

def mul_av(v):
    n = len(v)
    return [sum(a(i, j) * v[j] for j in range(n)) for i in range(n)]

def mul_atv(v):
    n = len(v)
    return [sum(a(j, i) * v[j] for j in range(n)) for i in range(n)]

n = 150
u = [1.0] * n
for _ in range(10):
    v = mul_atv(mul_av(u))
    u = mul_atv(mul_av(v))
vbv = sum(x * y for x, y in zip(u, v))
vv = sum(x * x for x in v)
print("%.9f" % (vbv / vv) ** 0.5)
