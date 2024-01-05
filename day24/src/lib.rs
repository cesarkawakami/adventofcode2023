use approx::relative_eq;
use std::io::Write;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
struct Pt2(f64, f64);

impl std::ops::Add for Pt2 {
    type Output = Pt2;

    fn add(self, other: Pt2) -> Pt2 {
        Pt2(self.0 + other.0, self.1 + other.1)
    }
}

impl std::ops::Sub for Pt2 {
    type Output = Pt2;

    fn sub(self, other: Pt2) -> Pt2 {
        Pt2(self.0 - other.0, self.1 - other.1)
    }
}

impl std::ops::Mul<Pt2> for f64 {
    type Output = Pt2;

    fn mul(self, other: Pt2) -> Pt2 {
        Pt2(self * other.0, self * other.1)
    }
}

impl std::ops::Div<f64> for Pt2 {
    type Output = Pt2;

    fn div(self, other: f64) -> Pt2 {
        Pt2(self.0 / other, self.1 / other)
    }
}

impl std::ops::Neg for Pt2 {
    type Output = Pt2;

    fn neg(self) -> Pt2 {
        Pt2(-self.0, -self.1)
    }
}

impl Pt2 {
    fn dot(self, other: Pt2) -> f64 {
        let Pt2(vx, vy) = self;
        let Pt2(wx, wy) = other;
        vx * wx + vy * wy
    }

    fn cross(self, other: Pt2) -> f64 {
        let Pt2(vx, vy) = self;
        let Pt2(wx, wy) = other;
        vx * wy - vy * wx
    }

    fn norm(self) -> f64 {
        self.dot(self).sqrt()
    }
}

impl approx::AbsDiffEq for Pt2 {
    type Epsilon = <f64 as approx::AbsDiffEq>::Epsilon;
    fn default_epsilon() -> Self::Epsilon {
        f64::default_epsilon()
    }
    fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
        f64::abs_diff_eq(&self.0, &other.0, epsilon) && f64::abs_diff_eq(&self.1, &other.1, epsilon)
    }
}

impl approx::RelativeEq for Pt2 {
    fn default_max_relative() -> Self::Epsilon {
        f64::default_max_relative()
    }

    fn relative_eq(
        &self,
        other: &Self,
        epsilon: Self::Epsilon,
        max_relative: Self::Epsilon,
    ) -> bool {
        f64::relative_eq(&self.0, &other.0, epsilon, max_relative)
            && f64::relative_eq(&self.1, &other.1, epsilon, max_relative)
    }
}

#[derive(Debug, Clone, Copy)]
struct Ray2 {
    pos: Pt2,
    vel: Pt2,
}

impl Ray2 {
    fn flip(self) -> Ray2 {
        let Ray2 { pos, vel } = self;
        Ray2 {
            pos: pos + vel,
            vel: -vel,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Segment2(Pt2, Pt2);

impl From<Segment2> for Ray2 {
    fn from(seg: Segment2) -> Ray2 {
        let Segment2(a, b) = seg;
        Ray2 { pos: a, vel: b - a }
    }
}

#[derive(Debug, Clone, Copy)]
enum RayIntersectResult {
    None,
    Single(Pt2),
    Ray(Ray2),
    Segment(Segment2),
}

impl Ray2 {
    fn intersect(self, other: Ray2) -> RayIntersectResult {
        let Ray2 { pos: p, vel: r } = self;
        let Ray2 { pos: q, vel: s } = other;

        if relative_eq!(r.norm(), 0.0) && relative_eq!(s.norm(), 0.0) {
            if relative_eq!(p, q) {
                RayIntersectResult::Single(p)
            } else {
                RayIntersectResult::None
            }
        } else if relative_eq!(r.norm(), 0.0) {
            if relative_eq!((p - q).cross(s), 0.0) && (p - q).dot(s) > 0.0 {
                RayIntersectResult::Single(p)
            } else {
                RayIntersectResult::None
            }
        } else if relative_eq!(s.norm(), 0.0) {
            other.intersect(self)
        } else {
            let denom = r.cross(s);
            let num_u = (q - p).cross(r);

            if relative_eq!(denom, 0.0) {
                if relative_eq!(num_u, 0.0) {
                    // Collinear
                    let t0 = (q - p).dot(r) / r.dot(r);
                    let t1 = t0 + s.dot(r) / r.dot(r);
                    if t0 < 0.0 {
                        if t1 < t0 {
                            RayIntersectResult::None
                        } else {
                            RayIntersectResult::Ray(self)
                        }
                    } else if t1 > t0 {
                        RayIntersectResult::Ray(other)
                    } else {
                        RayIntersectResult::Segment(Segment2(p, q))
                    }
                } else {
                    // Parallel, but non-intersecting
                    RayIntersectResult::None
                }
            } else {
                let t = (q - p).cross(s) / denom;
                let u = num_u / denom;
                if t > 0.0 && u > 0.0 {
                    RayIntersectResult::Single(p + t * r)
                } else {
                    RayIntersectResult::None
                }
            }
        }
    }

    fn has_intersection(self, other: Ray2) -> bool {
        match self.intersect(other) {
            RayIntersectResult::None => false,
            RayIntersectResult::Single(_) => true,
            RayIntersectResult::Ray(_) => true,
            RayIntersectResult::Segment(_) => true,
        }
    }

    fn has_intersection_with_segment(self, seg: Segment2) -> bool {
        let seg_ray1: Ray2 = seg.into();
        let seg_ray2 = seg_ray1.flip();
        self.has_intersection(seg_ray1) && self.has_intersection(seg_ray2)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AAB2 {
    min: Pt2,
    max: Pt2,
}

impl AAB2 {
    fn intersects_ir(self, ir: RayIntersectResult) -> bool {
        match ir {
            RayIntersectResult::None => false,
            RayIntersectResult::Single(p) => self.contains(p),
            RayIntersectResult::Ray(r) => self.intersects_ray(r),
            RayIntersectResult::Segment(s) => self.intersects_segment(s),
        }
    }

    fn contains(self, p: Pt2) -> bool {
        let AAB2 { min, max } = self;
        min.0 <= p.0 && p.0 <= max.0 && min.1 <= p.1 && p.1 <= max.1
    }

    fn intersects_segment(self, seg: Segment2) -> bool {
        let ray: Ray2 = seg.into();
        self.intersects_ray(ray) && self.intersects_ray(ray.flip())
    }

    fn intersects_ray(self, ray: Ray2) -> bool {
        if self.contains(ray.pos) {
            true
        } else {
            let AAB2 { min, max } = self;
            let (p0, p1, p2, p3) = (min, Pt2(min.0, max.1), max, Pt2(max.0, min.1));
            let (s0, s1, s2, s3) = (
                Segment2(p0, p1),
                Segment2(p1, p2),
                Segment2(p2, p3),
                Segment2(p3, p0),
            );
            ray.has_intersection_with_segment(s0)
                || ray.has_intersection_with_segment(s1)
                || ray.has_intersection_with_segment(s2)
                || ray.has_intersection_with_segment(s3)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
struct Pt3(f64, f64, f64);

impl Pt3 {
    fn from_str(s: &str) -> Pt3 {
        let (x, rest) = s.split_once(',').unwrap();
        let (y, z) = rest.split_once(',').unwrap();
        Pt3(
            x.trim().parse().unwrap(),
            y.trim().parse().unwrap(),
            z.trim().parse().unwrap(),
        )
    }

    fn discard_z(self) -> Pt2 {
        let Pt3(x, y, _) = self;
        Pt2(x, y)
    }
}

#[derive(Debug, Clone, Copy)]
struct Ray3 {
    pos: Pt3,
    vel: Pt3,
}

impl Ray3 {
    fn from_str(s: &str) -> Ray3 {
        let (pos, vel) = s.split_once('@').unwrap();
        Ray3 {
            pos: Pt3::from_str(pos.trim()),
            vel: Pt3::from_str(vel.trim()),
        }
    }

    fn discard_z(self) -> Ray2 {
        let Ray3 { pos, vel } = self;
        Ray2 {
            pos: pos.discard_z(),
            vel: vel.discard_z(),
        }
    }
}

pub fn part1(s: &str, aab: AAB2) -> usize {
    let rays = s
        .lines()
        .map(Ray3::from_str)
        .map(Ray3::discard_z)
        .collect::<Vec<_>>();
    let mut count = 0;
    for (i1, ray1) in rays.iter().cloned().enumerate() {
        for ray2 in rays.iter().cloned().skip(i1 + 1) {
            let intersection = ray1.intersect(ray2);
            // println!("{:?} and {:?} intersect at {:?}", ray1, ray2, intersection);
            if aab.intersects_ir(intersection) {
                count += 1;
            }
        }
    }
    count
}

pub fn part2(s: &str, subpath: &str) -> i64 {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .canonicalize()
        .unwrap()
        .parent()
        .unwrap()
        .join("day24-py")
        .join(subpath);
    let mut file = std::fs::File::create(path).unwrap();

    let rays = s.lines().map(Ray3::from_str).collect::<Vec<_>>();

    writeln!(
        file,
        r#"
def cross(a1, a2, a3, b1, b2, b3):
    return (
        a2 * b3 - a3 * b2,
        a3 * b1 - a1 * b3,
        a1 * b2 - a2 * b1,
    )
"#
    )
    .unwrap();

    let all_vars = ["prx", "pry", "prz", "vrx", "vry", "vrz"]
        .into_iter()
        .map(|s| s.to_string())
        // .chain((0..rays.len()).map(|i| format!("t{i}")))
        .collect::<Vec<_>>()
        .join(",");

    writeln!(file, "{all_vars} = var(\"{all_vars}\")").unwrap();

    writeln!(file, "eqs = []").unwrap();
    for ray in rays.iter().cloned() {
        let Ray3 {
            pos: Pt3(pix, piy, piz),
            vel: Pt3(vix, viy, viz),
        } = ray;

        writeln!(file, "(cx, cy, cz) = cross({pix} - prx, {piy} - pry, {piz} - prz, {vix} - vrx, {viy} - vry, {viz} - vrz)").unwrap();
        writeln!(file, "eqs.append(cx == 0)").unwrap();
        writeln!(file, "eqs.append(cy == 0)").unwrap();
        writeln!(file, "eqs.append(cz == 0)").unwrap();
    }

    // 12 eqs are enough for Sage to solve this
    writeln!(file, "print(solve(eqs[:12], {all_vars}))").unwrap();

    0
}

#[cfg(test)]
mod tests {
    const EXAMPLE1: &str = "\
19, 13, 30 @ -2,  1, -2
18, 19, 22 @ -1, -1, -2
20, 25, 34 @ -2, -2, -4
12, 31, 28 @ -1, -2, -1
20, 19, 15 @  1, -5, -3
";

    #[test]
    fn part1_example() {
        assert_eq!(
            super::part1(
                EXAMPLE1,
                super::AAB2 {
                    min: super::Pt2(7.0, 7.0),
                    max: super::Pt2(27.0, 27.0),
                }
            ),
            2
        );
    }

    #[test]
    fn part1_big() {
        assert_eq!(
            super::part1(
                include_str!("big.txt"),
                super::AAB2 {
                    min: super::Pt2(200000000000000.0, 200000000000000.0),
                    max: super::Pt2(400000000000000.0, 400000000000000.0),
                }
            ),
            15889
        );
    }

    #[test]
    fn part2_example() {
        super::part2(EXAMPLE1, "example.sage");
    }

    #[test]
    fn part2_big() {
        // Final answer 801386475216902
        super::part2(include_str!("big.txt"), "big.sage");
    }
}
