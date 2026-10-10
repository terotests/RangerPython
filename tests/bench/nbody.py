import math
PI = 3.14159265358979323
SOLAR_MASS = 4 * PI * PI
DAYS_PER_YEAR = 365.24
BODIES = [
    [[0.0, 0.0, 0.0], [0.0, 0.0, 0.0], SOLAR_MASS],
    [[4.84143144246472090e+00, -1.16032004402742839e+00, -1.03622044471123109e-01],
     [1.66007664274403694e-03 * DAYS_PER_YEAR, 7.69901118419740425e-03 * DAYS_PER_YEAR, -6.90460016972063023e-05 * DAYS_PER_YEAR],
     9.54791938424326609e-04 * SOLAR_MASS],
    [[8.34336671824457987e+00, 4.12479856412430479e+00, -4.03523417114321381e-01],
     [-2.76742510726862411e-03 * DAYS_PER_YEAR, 4.99852801234917238e-03 * DAYS_PER_YEAR, 2.30417297573763929e-05 * DAYS_PER_YEAR],
     2.85885980666130812e-04 * SOLAR_MASS],
    [[1.28943695621391310e+01, -1.51111514016986312e+01, -2.23307578892655734e-01],
     [2.96460137564761618e-03 * DAYS_PER_YEAR, 2.37847173959480950e-03 * DAYS_PER_YEAR, -2.96589568540237556e-05 * DAYS_PER_YEAR],
     4.36624404335156298e-05 * SOLAR_MASS],
    [[1.53796971148509165e+01, -2.59193146099879641e+01, 1.79258772950371181e-01],
     [2.68067772490389322e-03 * DAYS_PER_YEAR, 1.62824170038242295e-03 * DAYS_PER_YEAR, -9.51592254519715870e-05 * DAYS_PER_YEAR],
     5.15138902046611451e-05 * SOLAR_MASS],
]

def energy(bodies):
    e = 0.0
    for i, (p1, v1, m1) in enumerate(bodies):
        e += 0.5 * m1 * (v1[0] ** 2 + v1[1] ** 2 + v1[2] ** 2)
        for p2, v2, m2 in bodies[i + 1:]:
            dx, dy, dz = p1[0] - p2[0], p1[1] - p2[1], p1[2] - p2[2]
            e -= m1 * m2 / math.sqrt(dx * dx + dy * dy + dz * dz)
    return e

def advance(bodies, dt, n):
    pairs = [(bodies[i], bodies[j]) for i in range(len(bodies)) for j in range(i + 1, len(bodies))]
    for _ in range(n):
        for (p1, v1, m1), (p2, v2, m2) in pairs:
            dx, dy, dz = p1[0] - p2[0], p1[1] - p2[1], p1[2] - p2[2]
            mag = dt * ((dx * dx + dy * dy + dz * dz) ** -1.5)
            b1, b2 = m1 * mag, m2 * mag
            v1[0] -= dx * b2; v1[1] -= dy * b2; v1[2] -= dz * b2
            v2[0] += dx * b1; v2[1] += dy * b1; v2[2] += dz * b1
        for p, v, m in bodies:
            p[0] += dt * v[0]; p[1] += dt * v[1]; p[2] += dt * v[2]

print("%.9f" % energy(BODIES))
advance(BODIES, 0.01, 50000)
print("%.9f" % energy(BODIES))
