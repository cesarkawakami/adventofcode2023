use ndarray::s;

type Scalar = u16;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Brick {
    id: u16,
    x: (Scalar, Scalar),
    y: (Scalar, Scalar),
    z: (Scalar, Scalar),
}
impl Ord for Brick {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.z, self.x, self.y).cmp(&(other.z, other.x, other.y))
    }
}
impl PartialOrd for Brick {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Brick {
    fn from_str(s: &str, id: u16) -> Self {
        let (min, max) = s.trim().split_once('~').unwrap();
        let (minx, rest) = min.split_once(',').unwrap();
        let (miny, minz) = rest.split_once(',').unwrap();
        let (maxx, rest) = max.split_once(',').unwrap();
        let (maxy, maxz) = rest.split_once(',').unwrap();
        let minx = minx.parse::<Scalar>().unwrap();
        let miny = miny.parse::<Scalar>().unwrap();
        let minz = minz.parse::<Scalar>().unwrap();
        let maxx = maxx.parse::<Scalar>().unwrap();
        let maxy = maxy.parse::<Scalar>().unwrap();
        let maxz = maxz.parse::<Scalar>().unwrap();
        let (minx, maxx) = (minx.min(maxx), minx.max(maxx));
        let (miny, maxy) = (miny.min(maxy), miny.max(maxy));
        let (minz, maxz) = (minz.min(maxz), minz.max(maxz));
        Self {
            id,
            x: (minx, maxx),
            y: (miny, maxy),
            z: (minz, maxz),
        }
    }
}

struct Tower {
    bricks: Vec<Brick>,
}
impl Tower {
    fn from_str(s: &str) -> Self {
        let mut current_id = 0;
        let bricks = s
            .lines()
            .map(|line| {
                let brick = Brick::from_str(line, current_id);
                current_id += 1;
                brick
            })
            .collect::<Vec<Brick>>();
        Self { bricks }
    }

    fn fall(&mut self) -> SupportChain {
        self.bricks.sort();
        let maxx = self.bricks.iter().map(|b| b.x.1).max().unwrap();
        let maxy = self.bricks.iter().map(|b| b.y.1).max().unwrap();
        let mut birds_eye = ndarray::Array2::<(Scalar, Option<u16>)>::from_elem(
            ((maxx + 1).into(), (maxy + 1).into()),
            (0, None),
        );
        let max_id: usize = self.bricks.iter().map(|b| b.id).max().unwrap().into();
        let mut support_chain = SupportChain {
            at_ground: std::collections::HashSet::new(),
            supports: ndarray::Array2::zeros((max_id + 1, max_id + 1)),
        };
        for cur_brick in self.bricks.iter_mut() {
            let foot_x: (usize, usize) = (cur_brick.x.0.into(), cur_brick.x.1.into());
            let foot_y: (usize, usize) = (cur_brick.y.0.into(), cur_brick.y.1.into());
            let mut birds_eye_slice =
                birds_eye.slice_mut(s![foot_x.0..=foot_x.1, foot_y.0..=foot_y.1]);
            let found_z = birds_eye_slice.iter().map(|&(z, _)| z).max().unwrap();
            if found_z == 0 {
                support_chain.at_ground.insert(cur_brick.id);
            }
            birds_eye_slice
                .iter()
                .filter(|(z, _)| *z == found_z)
                .flat_map(|&(_, id)| id)
                .for_each(|id| {
                    support_chain.supports[(id.into(), cur_brick.id.into())] = 1;
                });
            assert!(cur_brick.z.0 > found_z);
            let delta_z = cur_brick.z.0 - found_z - 1;
            cur_brick.z.0 -= delta_z;
            cur_brick.z.1 -= delta_z;
            birds_eye_slice.fill((cur_brick.z.1, Some(cur_brick.id)));
        }
        support_chain
    }
}

struct SupportChain {
    at_ground: std::collections::HashSet<u16>,
    supports: ndarray::Array2<u16>,
}
impl SupportChain {
    fn chain(&self, id: u16) -> usize {
        // println!("at_ground: {:?}", self.at_ground);
        let mut supports = self.supports.clone();
        let mut removed = ndarray::Array1::<bool>::from_elem(supports.nrows(), false);
        supports
            .index_axis_mut(ndarray::Axis(0), usize::from(id))
            .fill(0);
        removed[usize::from(id)] = true;
        let mut in_degs = supports.sum_axis(ndarray::Axis(0));
        let mut count = 0;
        loop {
            let Some((id, _)) = in_degs.iter().zip(removed.iter()).enumerate().find(
                |&(id, (&in_deg, &removed))| {
                    in_deg == 0 && !removed && !self.at_ground.contains(&id.try_into().unwrap())
                },
            ) else {
                break;
            };
            // println!("id {id} has in_deg={}", in_degs[id]);
            count += 1;
            removed[id] = true;
            supports
                .index_axis_mut(ndarray::Axis(0), id)
                .indexed_iter_mut()
                .for_each(|(to_id, val)| {
                    if *val > 0 {
                        in_degs[to_id] -= 1;
                        *val = 0;
                    }
                });
        }
        // println!("chain {id}: {count}");
        count
    }
}

pub fn part1(s: &str) -> usize {
    let mut tower = Tower::from_str(s);
    let support_chain = tower.fall();
    (0..support_chain.supports.nrows())
        .filter(|&id| support_chain.chain(id.try_into().unwrap()) == 0)
        .count()
}

pub fn part2_single(s: &str, id: u16) -> usize {
    let mut tower = Tower::from_str(s);
    let support_chain = tower.fall();
    support_chain.chain(id)
}

pub fn part2(s: &str) -> usize {
    let mut tower = Tower::from_str(s);
    let support_chain = tower.fall();
    (0..support_chain.supports.nrows())
        .map(|id| support_chain.chain(id.try_into().unwrap()))
        .sum()
}

#[cfg(test)]
mod tests {
    const EXAMPLE1: &str = "\
1,0,1~1,2,1
0,0,2~2,0,2
0,2,3~2,2,3
0,0,4~0,2,4
2,0,5~2,2,5
0,1,6~2,1,6
1,1,8~1,1,9
";

    #[test]
    fn part1_example() {
        assert_eq!(super::part1(EXAMPLE1), 5);
    }

    #[test]
    fn part1_big() {
        assert_eq!(super::part1(include_str!("big.txt")), 395);
    }

    #[test]
    fn part2_single() {
        assert_eq!(super::part2_single(EXAMPLE1, 0), 6);
        assert_eq!(super::part2_single(EXAMPLE1, 5), 1);
    }

    #[test]
    fn part2_example() {
        assert_eq!(super::part2(EXAMPLE1), 7);
    }

    #[test]
    fn part2_final() {
        assert_eq!(super::part2(include_str!("big.txt")), 64714);
    }
}
