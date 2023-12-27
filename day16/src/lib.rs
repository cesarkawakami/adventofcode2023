use smallvec::smallvec;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Dir {
    N,
    S,
    W,
    E,
}

impl Dir {
    fn opposite(self) -> Self {
        match self {
            Self::N => Self::S,
            Self::S => Self::N,
            Self::W => Self::E,
            Self::E => Self::W,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct NodeId {
    row: u8,
    col: u8,
    dir: Dir,
}

impl NodeId {
    fn new<T: TryInto<u8>>(row: T, col: T, dir: Dir) -> Self
    where
        <T as TryInto<u8>>::Error: std::fmt::Debug,
    {
        Self {
            row: row.try_into().unwrap(),
            col: col.try_into().unwrap(),
            dir,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
enum Tile {
    #[default]
    Space,
    FwdMirror,
    BwdMirror,
    VBeam,
    HBeam,
}

#[derive(Debug, Clone)]
struct Map {
    tiles: ndarray::Array2<Tile>,
}

impl Map {
    fn read<R: std::io::BufRead>(reader: R) -> anyhow::Result<Self> {
        let (mut nrows, mut ncols) = (0, 0);
        let mut vals = vec![];
        for (r, line) in reader.lines().enumerate() {
            nrows = r + 1;
            for (col, c) in line?.trim().chars().enumerate() {
                ncols = col + 1;
                let tile = match c {
                    '.' => Tile::Space,
                    '/' => Tile::FwdMirror,
                    '\\' => Tile::BwdMirror,
                    '|' => Tile::VBeam,
                    '-' => Tile::HBeam,
                    _ => anyhow::bail!("invalid char: {}", c),
                };
                vals.push(tile);
            }
        }
        // println!("nrows: {}, ncols: {}", nrows, ncols);
        // println!("len: {}", vals.len());
        let tiles = ndarray::Array2::from_shape_vec((nrows, ncols), vals)?;
        Ok(Self { tiles })
    }

    fn coord_neighbor(&self, (r, c): (u8, u8), dir: Dir) -> Option<(u8, u8)> {
        let (nr, nc) = match dir {
            Dir::N => (r.wrapping_sub(1), c),
            Dir::S => (r + 1, c),
            Dir::W => (r, c.wrapping_sub(1)),
            Dir::E => (r, c + 1),
        };
        if nr < self.tiles.nrows() as u8 && nc < self.tiles.ncols() as u8 {
            Some((nr, nc))
        } else {
            None
        }
    }

    fn neighbors(&self, NodeId { row, col, dir }: NodeId) -> smallvec::SmallVec<[NodeId; 4]> {
        let mut result = smallvec![];
        if let Some((row, col)) = self.coord_neighbor((row, col), dir) {
            result.push(NodeId::new(row, col, dir.opposite()));
        }
        let internal_neigh_dirs: smallvec::SmallVec<[Dir; 2]> =
            match (self.tiles[(row as usize, col as usize)], dir) {
                (Tile::Space, _)
                | (Tile::VBeam, Dir::N)
                | (Tile::VBeam, Dir::S)
                | (Tile::HBeam, Dir::W)
                | (Tile::HBeam, Dir::E) => smallvec![dir.opposite()],
                (Tile::FwdMirror, Dir::N) => smallvec![Dir::W],
                (Tile::FwdMirror, Dir::W) => smallvec![Dir::N],
                (Tile::FwdMirror, Dir::S) => smallvec![Dir::E],
                (Tile::FwdMirror, Dir::E) => smallvec![Dir::S],
                (Tile::BwdMirror, Dir::N) => smallvec![Dir::E],
                (Tile::BwdMirror, Dir::E) => smallvec![Dir::N],
                (Tile::BwdMirror, Dir::S) => smallvec![Dir::W],
                (Tile::BwdMirror, Dir::W) => smallvec![Dir::S],
                (Tile::VBeam, Dir::W) | (Tile::VBeam, Dir::E) => smallvec![Dir::N, Dir::S],
                (Tile::HBeam, Dir::N) | (Tile::HBeam, Dir::S) => smallvec![Dir::E, Dir::W],
            };
        internal_neigh_dirs
            .into_iter()
            .for_each(|dir| result.push(NodeId::new(row, col, dir)));
        result
    }

    fn visit(
        &self,
        mut seen: std::collections::HashSet<NodeId>,
        node: NodeId,
    ) -> std::collections::HashSet<NodeId> {
        if seen.insert(node) {
            for neighbor in self.neighbors(node) {
                seen = self.visit(seen, neighbor)
            }
        }
        seen
    }
}

pub fn part1<R: std::io::BufRead>(reader: R) -> anyhow::Result<usize> {
    let map = Map::read(reader)?;
    let seen = map.visit(std::collections::HashSet::new(), NodeId::new(0, 0, Dir::W));
    Ok(seen
        .iter()
        .map(|node| (node.row, node.col))
        .collect::<std::collections::HashSet<_>>()
        .len())
}

pub fn part2<R: std::io::BufRead>(reader: R) -> anyhow::Result<usize> {
    let map = Map::read(reader)?;
    let result = std::iter::empty()
        .chain((0..map.tiles.ncols()).map(|c| NodeId::new(0, c, Dir::N)))
        .chain((0..map.tiles.ncols()).map(|c| NodeId::new(map.tiles.nrows() - 1, c, Dir::S)))
        .chain((0..map.tiles.nrows()).map(|r| NodeId::new(r, 0, Dir::W)))
        .chain((0..map.tiles.nrows()).map(|r| NodeId::new(r, map.tiles.ncols() - 1, Dir::E)))
        .map(|node| {
            let seen = map.visit(std::collections::HashSet::new(), node);
            seen.iter()
                .map(|node| (node.row, node.col))
                .collect::<std::collections::HashSet<_>>()
                .len()
        })
        .max()
        .ok_or(anyhow::anyhow!("huh"))?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    const EXAMPLE1: &str = r".|...\....
|.-.\.....
.....|-...
........|.
..........
.........\
..../.\\..
.-.-/..|..
.|....-|.\
..//.|....
";

    #[test]
    fn part1_example() -> anyhow::Result<()> {
        let reader = std::io::BufReader::new(EXAMPLE1.as_bytes());
        let result = super::part1(reader)?;
        assert_eq!(result, 46);
        Ok(())
    }

    #[test]
    fn part1_final() -> anyhow::Result<()> {
        let reader = std::io::BufReader::new(include_str!("big.txt").as_bytes());
        let result = stacker::grow(32 * 1024 * 1024, || super::part1(reader))?;
        assert_eq!(result, 6740);
        Ok(())
    }

    #[test]
    fn part2_example() -> anyhow::Result<()> {
        let reader = std::io::BufReader::new(EXAMPLE1.as_bytes());
        let result = super::part2(reader)?;
        assert_eq!(result, 51);
        Ok(())
    }

    #[test]
    fn part2_final() -> anyhow::Result<()> {
        let reader = std::io::BufReader::new(include_str!("big.txt").as_bytes());
        let result = stacker::grow(32 * 1024 * 1024, || super::part2(reader))?;
        assert_eq!(result, 7041);
        Ok(())
    }
}
