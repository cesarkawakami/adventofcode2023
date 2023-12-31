#[derive(Debug, Clone)]
struct PosVec(ndarray::Array1<u8>);

#[derive(Debug, Clone)]
struct AdjMtx(ndarray::Array2<u8>);

impl AdjMtx {
    fn apply(&self, PosVec(vec): &PosVec) -> PosVec {
        let mut result = ndarray::Array1::<u8>::zeros(vec.len());
        for i in 0..self.0.nrows() {
            for j in 0..self.0.ncols() {
                result[i] = result[i].max(self.0[(i, j)] * vec[j]);
            }
        }
        PosVec(result)
    }

    #[allow(dead_code)]
    fn compose_dense(&self, other: &Self) -> Self {
        let mut result = ndarray::Array2::<u8>::zeros(self.0.dim());
        for i in 0..self.0.nrows() {
            for j in 0..other.0.ncols() {
                for k in 0..self.0.ncols() {
                    result[(i, j)] = result[(i, j)].max(self.0[(i, k)] * other.0[(k, j)]);
                }
            }
        }
        Self(result)
    }

    #[allow(dead_code)]
    fn compose_sparse(&self, other: &Self) -> Self {
        let mut result = ndarray::Array2::<u8>::zeros(self.0.dim());
        let mut other_edges: Vec<(u16, u16)> = other
            .0
            .indexed_iter()
            .filter(|&(_, &v)| v == 1)
            .map(|((i, j), _)| (u16::try_from(i).unwrap(), u16::try_from(j).unwrap()))
            .collect();
        other_edges.sort();

        for i in 0..self.0.nrows() {
            for k in 0..self.0.ncols() {
                if self.0[(i, k)] == 1 {
                    let k_u16 = u16::try_from(k).unwrap();
                    let range_start = other_edges.partition_point(|&(kk, _)| kk < k_u16);
                    let range_end = other_edges.partition_point(|&(kk, _)| kk <= k_u16);
                    for &(kk, j) in &other_edges[range_start..range_end] {
                        assert!(kk == k_u16);
                        result[(i, usize::from(j))] = 1;
                    }
                }
            }
        }

        Self(result)
    }

    fn compose(&self, other: &Self) -> Self {
        self.compose_sparse(other)
    }

    fn pow(&self, n: usize) -> Self {
        if n == 0 {
            Self(ndarray::Array2::<u8>::from_diag_elem(self.0.nrows(), 1))
        } else if n == 1 {
            self.clone()
        } else if n % 2 == 0 {
            let half = self.pow(n / 2);
            half.compose(&half)
        } else {
            let half = self.pow(n / 2);
            half.compose(&half).compose(self)
        }
    }
}

#[derive(Debug, Clone)]
struct Map {
    map: ndarray::Array2<u8>,
}

const DELTAS_I16: [(i16, i16); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
const DELTAS_I32: [(i32, i32); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];

impl Map {
    fn from_str(s: &str) -> Self {
        let mut map = ndarray::Array2::zeros((0, 0));
        for line in s.lines() {
            let line = line.trim();
            let row = line
                .chars()
                .map(|c| u8::try_from(c).unwrap())
                .collect::<ndarray::Array1<u8>>();
            if row.len() != map.ncols() {
                map = map.into_shape((0, row.len())).unwrap();
            }
            map.push_row(row.view()).unwrap();
        }
        Self { map }
    }

    fn coord_to_idx(&self, r: usize, c: usize) -> usize {
        r * self.map.ncols() + c
    }

    fn vtx_cnt(&self) -> usize {
        self.map.nrows() * self.map.ncols()
    }

    fn initial_pos_vec(&self) -> PosVec {
        let mut vec = ndarray::Array1::<u8>::zeros(self.vtx_cnt());
        self.map.indexed_iter().for_each(|((r, c), &v)| {
            if v == b'S' {
                vec[self.coord_to_idx(r, c)] = 1;
            }
        });
        PosVec(vec)
    }

    fn adj_mtx(&self) -> AdjMtx {
        let mut mtx = ndarray::Array2::<u8>::zeros((self.vtx_cnt(), self.vtx_cnt()));
        let nrows: i16 = self.map.nrows().try_into().unwrap();
        let ncols: i16 = self.map.ncols().try_into().unwrap();
        for r in 0..nrows {
            for c in 0..ncols {
                if self.map[(r.try_into().unwrap(), c.try_into().unwrap())] != b'#' {
                    for (dr, dc) in DELTAS_I16 {
                        let (nr, nc) = (r + dr, c + dc);
                        if 0 <= nr
                            && nr < nrows
                            && 0 <= nc
                            && nc < ncols
                            && self.map[(nr.try_into().unwrap(), nc.try_into().unwrap())] != b'#'
                        {
                            let from_idx =
                                self.coord_to_idx(r.try_into().unwrap(), c.try_into().unwrap());
                            let to_idx =
                                self.coord_to_idx(nr.try_into().unwrap(), nc.try_into().unwrap());
                            mtx[(from_idx, to_idx)] = 1;
                        }
                    }
                }
            }
        }
        AdjMtx(mtx)
    }

    fn is_rock_wrapped(&self, r: i32, c: i32) -> bool {
        let r: usize = r
            .rem_euclid(self.map.nrows().try_into().unwrap())
            .try_into()
            .unwrap();
        let c: usize = c
            .rem_euclid(self.map.ncols().try_into().unwrap())
            .try_into()
            .unwrap();
        self.map[(r, c)] == b'#'
    }

    fn iter_set_wrapped(
        &self,
        set: &std::collections::HashSet<(i32, i32)>,
    ) -> std::collections::HashSet<(i32, i32)> {
        let mut result = std::collections::HashSet::new();
        for (r, c) in set.iter() {
            for (dr, dc) in DELTAS_I32 {
                let (r, c) = (r + dr, c + dc);
                if !self.is_rock_wrapped(r, c) {
                    result.insert((r, c));
                }
            }
        }
        result
    }

    fn exact_dist_set_wrapped(&self, count: usize) -> std::collections::HashSet<(i32, i32)> {
        let mut current = std::collections::HashSet::new();
        for ((r, c), &v) in self.map.indexed_iter() {
            if v == b'S' {
                current.insert((r.try_into().unwrap(), c.try_into().unwrap()));
            }
        }
        for _ in 0..count {
            current = self.iter_set_wrapped(&current);
        }
        current
    }
}

pub fn part1(input: &str, steps: usize) -> usize {
    let map = Map::from_str(input);
    let initial_vec = map.initial_pos_vec();
    let adj_mtx = map.adj_mtx();
    println!("adj_mtx dim: {:?}", adj_mtx.0.dim());
    let final_vec = adj_mtx.pow(steps).apply(&initial_vec);
    final_vec.0.iter().filter(|&&v| v == 1).count()
}

pub fn part2(input: &str, steps: usize) -> usize {
    let map = Map::from_str(input);
    let final_set = map.exact_dist_set_wrapped(steps);
    final_set.len()
}

pub fn part2_solved(steps: usize) -> usize {
    // solved by fitting to quadratic /shrug
    let a: u128 = 15236;
    let b: u128 = 33969;
    let c: u128 = 159044;
    let div: u128 = 17161;
    let steps: u128 = steps.try_into().unwrap();
    let num = a * steps * steps + b * steps + c;
    if num % div != 0 {
        panic!("{} % {} != 0", num, div);
    }
    (num / div).try_into().unwrap()
}

#[cfg(test)]
mod tests {
    const EXAMPLE1: &str = "\
...........
.....###.#.
.###.##..#.
..#.#...#..
....#.#....
.##..S####.
.##..#...#.
.......##..
.##.#.####.
.##..##.##.
...........
";

    #[test]
    fn part1_example() {
        assert_eq!(super::part1(EXAMPLE1, 6), 16);
    }

    #[test]
    fn part1_big() {
        assert_eq!(super::part1(include_str!("big.txt"), 64), 3746);
    }

    #[test]
    fn part2_example() {
        assert_eq!(super::part2(EXAMPLE1, 6), 16);
        assert_eq!(super::part2(EXAMPLE1, 10), 50);
        assert_eq!(super::part2(EXAMPLE1, 50), 1594);
        assert_eq!(super::part2(EXAMPLE1, 100), 6536);
        // assert_eq!(super::part2(EXAMPLE1, 500), 167004);
        // assert_eq!(super::part2(EXAMPLE1, 1000), 668697);
        // assert_eq!(super::part2(EXAMPLE1, 5000), 16733044);
    }

    #[test]
    fn part2_final_fragments() {
        for cnt in 0..=3 {
            let steps = cnt * 131 + 65;
            let answer = super::part2(include_str!("big.txt"), steps);
            let steps2 = steps * steps;
            println!("a * {steps2} + b * {steps} + c = {answer}");
        }
        // assert_eq!(super::part2(include_str!("big.txt"), 65), 0);
    }

    #[test]
    fn part2_final_solved() {
        assert_eq!(super::part2_solved(65), 3889);
        assert_eq!(super::part2_solved(196), 34504);
        assert_eq!(super::part2_solved(327), 95591);
        assert_eq!(super::part2_solved(458), 187150);
        assert_eq!(super::part2_solved(26501365), 623540829615589);
    }
}
