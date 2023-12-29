use ndarray::s;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Dir {
    U,
    D,
    L,
    R,
}

impl Dir {
    const VALS: [Self; 4] = [Self::U, Self::D, Self::L, Self::R];

    fn apply(self, (r, c): (i64, i64), cnt: i64) -> (i64, i64) {
        match self {
            Self::U => (r - cnt, c),
            Self::R => (r, c + cnt),
            Self::D => (r + cnt, c),
            Self::L => (r, c - cnt),
        }
    }
}

impl std::str::FromStr for Dir {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "U" => Ok(Self::U),
            "D" => Ok(Self::D),
            "L" => Ok(Self::L),
            "R" => Ok(Self::R),
            _ => anyhow::bail!("invalid direction: {s}"),
        }
    }
}

#[derive(Debug, Clone)]
struct Inst {
    dir: Dir,
    cnt: i64,
}

impl std::str::FromStr for Inst {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (dir, rest) = s.split_once(' ').ok_or(anyhow::anyhow!("no ' '"))?;
        let dir = dir.parse()?;
        let (cnt, _) = rest.split_once(' ').ok_or(anyhow::anyhow!("no ' '"))?;
        let cnt = cnt.parse()?;
        Ok(Self { dir, cnt })
    }
}

impl Inst {
    fn from_color_str(s: &str) -> Self {
        let (_, rest) = s.split_once('#').unwrap();
        let (color, _) = rest.split_once(')').unwrap();
        let cnt = i64::from_str_radix(&color[..5], 16).unwrap();
        let dir_ch = color.chars().nth(5).unwrap();
        let dir = match dir_ch {
            '0' => Dir::R,
            '1' => Dir::D,
            '2' => Dir::L,
            '3' => Dir::U,
            _ => panic!("invalid dir hex: {dir_ch}"),
        };
        Self { dir, cnt }
    }
}

#[derive(Debug, Clone)]
struct InstList {
    insts: Vec<Inst>,
}

impl std::str::FromStr for InstList {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> anyhow::Result<Self> {
        Ok(Self {
            insts: s
                .lines()
                .map(|l| l.parse::<Inst>())
                .collect::<Result<_, _>>()?,
        })
    }
}

impl InstList {
    fn from_color_str(s: &str) -> Self {
        Self {
            insts: s.lines().map(Inst::from_color_str).collect(),
        }
    }
}

struct Map {
    mtx: ndarray::Array2<u8>,
    row_weights: ndarray::Array1<u64>,
    col_weights: ndarray::Array1<u64>,
}

impl Map {
    fn from_inst_list(inst_list: &InstList) -> Self {
        let mut seen_rows = std::collections::BTreeSet::new();
        let mut seen_cols = std::collections::BTreeSet::new();
        let (mut r, mut c): (i64, i64) = (0, 0);
        seen_rows.insert(r);
        seen_cols.insert(c);
        for &Inst { dir, cnt, .. } in inst_list.insts.iter() {
            (r, c) = dir.apply((r, c), cnt);
            seen_rows.insert(r);
            seen_cols.insert(c);
        }
        let compress_row: std::collections::HashMap<i64, usize> = seen_rows
            .iter()
            .cloned()
            .enumerate()
            .map(|(i, r)| (r, 2 * i))
            .collect();
        let compress_col: std::collections::HashMap<i64, usize> = seen_cols
            .iter()
            .cloned()
            .enumerate()
            .map(|(i, c)| (c, 2 * i))
            .collect();
        let decomp_row: std::collections::HashMap<usize, i64> =
            compress_row.iter().map(|(&r, &i)| (i, r)).collect();
        let decomp_col: std::collections::HashMap<usize, i64> =
            compress_col.iter().map(|(&c, &i)| (i, c)).collect();
        let mut row_weights = ndarray::Array1::<u64>::from_elem(2 * compress_row.len() - 1, 1);
        let mut col_weights = ndarray::Array1::<u64>::from_elem(2 * compress_col.len() - 1, 1);
        let mut mtx = ndarray::Array2::<u8>::zeros((row_weights.len(), col_weights.len()));
        for r in (1..row_weights.len()).step_by(2) {
            row_weights[r] = (decomp_row[&(r + 1)] - decomp_row[&(r - 1)] - 1)
                .try_into()
                .unwrap();
        }
        for c in (1..col_weights.len()).step_by(2) {
            col_weights[c] = (decomp_col[&(c + 1)] - decomp_col[&(c - 1)] - 1)
                .try_into()
                .unwrap();
        }

        let (mut r, mut c): (i64, i64) = (0, 0);
        mtx[(compress_row[&r], compress_col[&c])] = 1;
        for &Inst { dir, cnt, .. } in inst_list.insts.iter() {
            let (nr, nc) = dir.apply((r, c), cnt);
            let (comp_r0, comp_r1, comp_c0, comp_c1) = (
                compress_row[&r],
                compress_row[&nr],
                compress_col[&c],
                compress_col[&nc],
            );
            let (comp_r0, comp_r1, comp_c0, comp_c1) = (
                comp_r0.min(comp_r1),
                comp_r0.max(comp_r1),
                comp_c0.min(comp_c1),
                comp_c0.max(comp_c1),
            );
            mtx.slice_mut(s![comp_r0..=comp_r1, comp_c0..=comp_c1])
                .fill(1);
            (r, c) = (nr, nc);
        }

        Self {
            mtx,
            row_weights,
            col_weights,
        }
    }

    fn inside_area(&self) -> u64 {
        fn visit(mtx: &ndarray::Array2<u8>, visited: &mut ndarray::Array2<u8>, (r, c): (i64, i64)) {
            let (nrows, ncols): (i64, i64) = (
                mtx.nrows().try_into().unwrap(),
                mtx.ncols().try_into().unwrap(),
            );
            if r < 0 || r >= nrows || c < 0 || c >= ncols {
                return;
            }
            let (ur, uc): (usize, usize) = (r.try_into().unwrap(), c.try_into().unwrap());
            if mtx[(ur, uc)] == 1 || visited[(ur, uc)] == 1 {
                return;
            }
            visited[(ur, uc)] = 1;
            for dir in Dir::VALS {
                visit(mtx, visited, dir.apply((r, c), 1));
            }
        }

        let mut visited = ndarray::Array2::<u8>::zeros(self.mtx.dim());
        let (nrows, ncols) = visited.dim();
        let (nrows, ncols): (i64, i64) = (nrows.try_into().unwrap(), ncols.try_into().unwrap());
        for r in 0..nrows {
            visit(&self.mtx, &mut visited, (r, 0));
            visit(&self.mtx, &mut visited, (r, ncols - 1));
        }
        for c in 0..ncols {
            visit(&self.mtx, &mut visited, (0, c));
            visit(&self.mtx, &mut visited, (nrows - 1, c));
        }
        let total_area = self.row_weights.sum() * self.col_weights.sum();
        let outside_area = visited
            .indexed_iter()
            .map(|((r, c), &x)| u64::from(x) * self.row_weights[r] * self.col_weights[c])
            .sum::<u64>();
        total_area - outside_area
    }
}

impl std::fmt::Display for Map {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in self.mtx.outer_iter() {
            let line: String = row
                .iter()
                .cloned()
                .map(|x| match x {
                    0 => '.',
                    1 => '#',
                    _ => '?',
                })
                .collect();
            writeln!(f, "{line}")?;
        }
        Ok(())
    }
}

pub fn part1<R: std::io::BufRead>(mut reader: R) -> u64 {
    let mut input = String::new();
    reader.read_to_string(&mut input).unwrap();
    let inst_list: InstList = input.parse().unwrap();
    let map = Map::from_inst_list(&inst_list);
    // println!("{map}");
    map.inside_area()
}

pub fn part2<R: std::io::BufRead>(mut reader: R) -> u64 {
    let mut input = String::new();
    reader.read_to_string(&mut input).unwrap();
    let inst_list = InstList::from_color_str(input.as_str());
    let map = Map::from_inst_list(&inst_list);
    // println!("{map}");
    map.inside_area()
}

#[cfg(test)]
mod tests {
    const EXAMPLE1: &str = "\
R 6 (#70c710)
D 5 (#0dc571)
L 2 (#5713f0)
D 2 (#d2c081)
R 2 (#59c680)
D 2 (#411b91)
L 5 (#8ceee2)
U 2 (#caa173)
L 1 (#1b58a2)
U 2 (#caa171)
R 2 (#7807d2)
U 3 (#a77fa3)
L 2 (#015232)
U 2 (#7a21e3)
";

    #[test]
    fn part1_example() {
        let reader = std::io::BufReader::new(EXAMPLE1.as_bytes());
        let result = super::part1(reader);
        assert_eq!(result, 62);
    }

    #[test]
    fn part1_big() {
        let reader = std::io::BufReader::new(include_str!("big.txt").as_bytes());
        let result = stacker::grow(32 * 1024 * 1024, || super::part1(reader));
        assert_eq!(result, 48795);
    }

    #[test]
    fn part2_example() {
        let reader = std::io::BufReader::new(EXAMPLE1.as_bytes());
        let result = super::part2(reader);
        assert_eq!(result, 952408144115);
    }

    #[test]
    fn part2_big() {
        let reader = std::io::BufReader::new(include_str!("big.txt").as_bytes());
        let result = stacker::grow(256 * 1024 * 1024, || super::part2(reader));
        assert_eq!(result, 40654918441248);
    }
}
