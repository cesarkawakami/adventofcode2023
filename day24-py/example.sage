
def cross(a1, a2, a3, b1, b2, b3):
    return (
        a2 * b3 - a3 * b2,
        a3 * b1 - a1 * b3,
        a1 * b2 - a2 * b1,
    )

prx,pry,prz,vrx,vry,vrz = var("prx,pry,prz,vrx,vry,vrz")
eqs = []
(cx, cy, cz) = cross(19 - prx, 13 - pry, 30 - prz, -2 - vrx, 1 - vry, -2 - vrz)
eqs.append(cx == 0)
eqs.append(cy == 0)
eqs.append(cz == 0)
(cx, cy, cz) = cross(18 - prx, 19 - pry, 22 - prz, -1 - vrx, -1 - vry, -2 - vrz)
eqs.append(cx == 0)
eqs.append(cy == 0)
eqs.append(cz == 0)
(cx, cy, cz) = cross(20 - prx, 25 - pry, 34 - prz, -2 - vrx, -2 - vry, -4 - vrz)
eqs.append(cx == 0)
eqs.append(cy == 0)
eqs.append(cz == 0)
(cx, cy, cz) = cross(12 - prx, 31 - pry, 28 - prz, -1 - vrx, -2 - vry, -1 - vrz)
eqs.append(cx == 0)
eqs.append(cy == 0)
eqs.append(cz == 0)
(cx, cy, cz) = cross(20 - prx, 19 - pry, 15 - prz, 1 - vrx, -5 - vry, -3 - vrz)
eqs.append(cx == 0)
eqs.append(cy == 0)
eqs.append(cz == 0)
print(solve(eqs, prx,pry,prz,vrx,vry,vrz))
