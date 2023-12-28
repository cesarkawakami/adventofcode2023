use std::cmp::Reverse;

type Slr = i16;
type Dst = u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Dir {
    N,
    E,
    S,
    W,
}

impl Dir {
    const VALS: [Self; 4] = [Self::N, Self::E, Self::S, Self::W];

    fn apply(&self, (given_r, given_c): (Slr, Slr)) -> (Slr, Slr) {
        match self {
            Self::N => (given_r - 1, given_c),
            Self::E => (given_r, given_c + 1),
            Self::S => (given_r + 1, given_c),
            Self::W => (given_r, given_c - 1),
        }
    }

    fn opposite(&self) -> Self {
        match self {
            Self::N => Self::S,
            Self::E => Self::W,
            Self::S => Self::N,
            Self::W => Self::E,
        }
    }
}

trait NodeIdIsh:
    std::fmt::Debug + Clone + Copy + PartialEq + Eq + PartialOrd + Ord + std::hash::Hash
{
    fn new(r: Slr, c: Slr, recent_dir: Dir, recent_cnt: u8) -> Self;

    fn coords(&self) -> (Slr, Slr);

    fn apply(&self, map: &Map, dir: Dir) -> Option<Self>;

    fn neighs(&self, map: &Map) -> smallvec::SmallVec<[(u8, Self); 4]> {
        Dir::VALS
            .iter()
            .cloned()
            .flat_map(|dir| self.apply(map, dir))
            .map(|node| {
                let (r, c) = node.coords();
                (
                    map.mtx[(r.try_into().unwrap(), c.try_into().unwrap())],
                    node,
                )
            })
            .collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct NodeId {
    r: Slr,
    c: Slr,
    recent_dir: Dir,
    recent_cnt: u8,
}

impl NodeIdIsh for NodeId {
    fn new(r: Slr, c: Slr, recent_dir: Dir, recent_cnt: u8) -> Self {
        Self {
            r,
            c,
            recent_dir,
            recent_cnt,
        }
    }

    fn coords(&self) -> (Slr, Slr) {
        (self.r, self.c)
    }

    fn apply(&self, map: &Map, dir: Dir) -> Option<NodeId> {
        let NodeId {
            r,
            c,
            recent_dir,
            recent_cnt,
        } = *self;
        let (nr, nc) = dir.apply((r, c));
        if nr < 0
            || nr >= map.mtx.nrows().try_into().unwrap()
            || nc < 0
            || nc >= map.mtx.ncols().try_into().unwrap()
        {
            return None;
        }
        if dir == recent_dir {
            if recent_cnt < 3 {
                Some(NodeId::new(nr, nc, dir, recent_cnt + 1))
            } else {
                None
            }
        } else if dir == recent_dir.opposite() {
            None
        } else {
            Some(NodeId::new(nr, nc, dir, 1))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct NodeId2 {
    r: Slr,
    c: Slr,
    recent_dir: Dir,
    recent_cnt: u8,
}

impl NodeIdIsh for NodeId2 {
    fn new(r: Slr, c: Slr, recent_dir: Dir, recent_cnt: u8) -> Self {
        Self {
            r,
            c,
            recent_dir,
            recent_cnt,
        }
    }

    fn coords(&self) -> (Slr, Slr) {
        (self.r, self.c)
    }

    fn apply(&self, map: &Map, dir: Dir) -> Option<NodeId2> {
        let NodeId2 {
            r,
            c,
            recent_dir,
            recent_cnt,
        } = *self;
        let (nr, nc) = dir.apply((r, c));
        if nr < 0
            || nr >= map.mtx.nrows().try_into().unwrap()
            || nc < 0
            || nc >= map.mtx.ncols().try_into().unwrap()
        {
            return None;
        }
        if dir == recent_dir {
            if recent_cnt < 10 {
                Some(NodeId2::new(nr, nc, dir, recent_cnt + 1))
            } else {
                None
            }
        } else if dir == recent_dir.opposite() {
            None
        } else if recent_cnt >= 4 {
            Some(NodeId2::new(nr, nc, dir, 1))
        } else {
            None
        }
    }
}

struct Map {
    mtx: ndarray::Array2<u8>,
}

impl Map {
    fn read<R: std::io::BufRead>(reader: R) -> anyhow::Result<Self> {
        let mut vals = vec![];
        let (mut nrows, mut ncols) = (0, 0);
        for (r, line) in reader.lines().enumerate() {
            for (c, val) in line?.trim().chars().enumerate() {
                nrows = r + 1;
                ncols = c + 1;
                vals.push(u8::try_from(val).unwrap() - b'0');
            }
        }
        Ok(Map {
            mtx: ndarray::Array2::from_shape_vec((nrows, ncols), vals)?,
        })
    }

    fn solve<NI: NodeIdIsh>(&self, (sr, sc): (Slr, Slr)) -> std::collections::HashMap<NI, Dst> {
        let mut dists = std::collections::HashMap::<NI, Dst>::new();
        let mut pq = std::collections::BinaryHeap::<(Reverse<Dst>, NI)>::new();
        pq.push((Reverse(0), NI::new(sr, sc, Dir::E, 0)));
        pq.push((Reverse(0), NI::new(sr, sc, Dir::S, 0)));
        while let Some((Reverse(dist), node)) = pq.pop() {
            if dists.contains_key(&node) {
                continue;
            }
            dists.insert(node, dist);
            for (edge_dist, neigh) in node.neighs(self) {
                pq.push((Reverse(dist + Dst::from(edge_dist)), neigh));
            }
        }
        dists
    }
}

pub fn part1<R: std::io::BufRead>(reader: R) -> anyhow::Result<usize> {
    let map = Map::read(reader)?;
    let dists = map.solve::<NodeId>((0, 0));
    dists
        .iter()
        .filter(|(&NodeId { r, c, .. }, _)| {
            (r.try_into().unwrap(), c.try_into().unwrap())
                == (map.mtx.nrows() - 1, map.mtx.ncols() - 1)
        })
        .map(|(_, &dist)| dist.try_into().unwrap())
        .min()
        .ok_or(anyhow::anyhow!("no path found"))
}

pub fn part2<R: std::io::BufRead>(reader: R) -> anyhow::Result<usize> {
    let map = Map::read(reader)?;
    let dists = map.solve::<NodeId2>((0, 0));
    dists
        .iter()
        .filter(
            |(
                &NodeId2 {
                    r, c, recent_cnt, ..
                },
                _,
            )| {
                (r.try_into().unwrap(), c.try_into().unwrap())
                    == (map.mtx.nrows() - 1, map.mtx.ncols() - 1)
                    && recent_cnt >= 4
            },
        )
        .map(|(_, &dist)| dist.try_into().unwrap())
        .min()
        .ok_or(anyhow::anyhow!("no path found"))
}

#[cfg(test)]
mod tests {
    const EXAMPLE1: &str = "\
2413432311323
3215453535623
3255245654254
3446585845452
4546657867536
1438598798454
4457876987766
3637877979653
4654967986887
4564679986453
1224686865563
2546548887735
4322674655533
";

    #[test]
    fn part1_example() -> anyhow::Result<()> {
        let reader = std::io::BufReader::new(EXAMPLE1.as_bytes());
        let result = super::part1(reader)?;
        assert_eq!(result, 102);
        Ok(())
    }

    #[test]
    fn part1_big() -> anyhow::Result<()> {
        let reader = std::io::BufReader::new(include_str!("big.txt").as_bytes());
        let result = super::part1(reader)?;
        assert_eq!(result, 851);
        Ok(())
    }

    const EXAMPLE2: &str = "\
111111111111
999999999991
999999999991
999999999991
999999999991
";

    #[test]
    fn part2_example() -> anyhow::Result<()> {
        let reader = std::io::BufReader::new(EXAMPLE1.as_bytes());
        let result = super::part2(reader)?;
        assert_eq!(result, 94);
        Ok(())
    }

    #[test]
    fn part2_example2() -> anyhow::Result<()> {
        let reader = std::io::BufReader::new(EXAMPLE2.as_bytes());
        let result = super::part2(reader)?;
        assert_eq!(result, 71);
        Ok(())
    }

    #[test]
    fn part2_big() -> anyhow::Result<()> {
        let reader = std::io::BufReader::new(include_str!("big.txt").as_bytes());
        let result = super::part2(reader)?;
        assert_eq!(result, 982);
        Ok(())
    }
}
