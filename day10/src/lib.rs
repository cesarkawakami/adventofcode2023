const MAX_N: usize = 200;

#[derive(Debug, Clone)]
struct Map {
    n_rows: usize,
    n_cols: usize,
    start: (usize, usize),
    mtx: [[char; MAX_N]; MAX_N],
    dists: [[i32; MAX_N]; MAX_N],
}

impl Map {
    fn from_lines(lines: impl Iterator<Item = String>) -> Self {
        let mut n_cols = 0;
        let mut n_rows = 0;
        let mut mtx: [[char; MAX_N]; MAX_N] = [['.'; MAX_N]; MAX_N];
        for (r, line) in lines.enumerate() {
            let line = line.trim();
            n_cols = line.len();
            n_rows = r + 1;
            for (c, ch) in line.chars().enumerate() {
                mtx[r][c] = ch;
            }
        }
        let mut map = Map {
            n_rows,
            n_cols,
            start: (usize::MAX, usize::MAX),
            mtx,
            dists: [[-1; MAX_N]; MAX_N],
        };
        map.adjust_start();
        map
    }

    fn adjust_start(&mut self) {
        for r in 0..self.n_rows {
            for c in 0..self.n_cols {
                if self.mtx[r][c] == 'S' {
                    self.start = (r, c);
                    let mut actual_neighbors: Vec<(i32, i32)> = self
                        .neighbors((r, c))
                        .filter_map(|(cr, cc)| {
                            if self.neighbors((cr, cc)).any(|p| p == (r, c)) {
                                Some((cr as i32 - r as i32, cc as i32 - c as i32))
                            } else {
                                None
                            }
                        })
                        .collect();
                    actual_neighbors.sort();
                    self.mtx[r][c] = match actual_neighbors.as_slice() {
                        [(-1, 0), (1, 0)] => '|',
                        [(0, -1), (0, 1)] => '-',
                        [(-1, 0), (0, 1)] => 'L',
                        [(-1, 0), (0, -1)] => 'J',
                        [(0, -1), (1, 0)] => '7',
                        [(0, 1), (1, 0)] => 'F',
                        _ => panic!("Invalid map char: {:?}", self.mtx[r][c]),
                    };
                    return;
                }
            }
        }
    }

    fn neighbors(&self, (r, c): (usize, usize)) -> impl Iterator<Item = (usize, usize)> {
        let deltas: Vec<(i32, i32)> = match self.mtx[r][c] {
            '|' => vec![(-1, 0), (1, 0)],
            '-' => vec![(0, -1), (0, 1)],
            'L' => vec![(-1, 0), (0, 1)],
            'J' => vec![(-1, 0), (0, -1)],
            '7' => vec![(0, -1), (1, 0)],
            'F' => vec![(0, 1), (1, 0)],
            'S' => vec![(-1, 0), (0, -1), (0, 1), (1, 0)],
            _ => vec![],
        };
        let (n_rows, n_cols) = (self.n_rows, self.n_cols);
        deltas.into_iter().filter_map(move |(dr, dc)| {
            let nr = r as i32 + dr;
            let nc = c as i32 + dc;
            if 0 <= nr && nr < n_rows as i32 && 0 <= nc && nc < n_cols as i32 {
                Some((nr as usize, nc as usize))
            } else {
                None
            }
        })
    }

    fn compute_dists(&mut self) {
        let mut q = std::collections::VecDeque::new();
        q.push_back(self.start);
        self.dists[self.start.0][self.start.1] = 0;
        while let Some((r, c)) = q.pop_front() {
            let dist = self.dists[r][c];
            for (nr, nc) in self.neighbors((r, c)) {
                if self.dists[nr][nc] == -1 {
                    self.dists[nr][nc] = dist + 1;
                    q.push_back((nr, nc));
                }
            }
        }
    }

    fn remove_non_wall(&mut self) {
        for r in 0..self.n_rows {
            for c in 0..self.n_cols {
                if self.dists[r][c] == -1 {
                    self.mtx[r][c] = '.';
                }
            }
        }
    }

    fn inside(&self, (r, c): (usize, usize)) -> bool {
        if self.mtx[r][c] != '.' {
            return false;
        }
        let mut intersections: usize = 0;
        for nc in c..self.n_cols {
            let delta: usize = match self.mtx[r][nc] {
                // Pretend we're crossing a bit lower than midpoint
                '|' => 1,
                '-' => 0,
                'L' => 0,
                'J' => 0,
                '7' => 1,
                'F' => 1,
                _ => 0,
            };
            intersections += delta;
        }
        intersections % 2 == 1
    }
}

pub fn part1<R: std::io::BufRead>(reader: R) -> i64 {
    let lines = reader.lines().map(|l| l.unwrap());
    let mut map = Map::from_lines(lines);
    map.compute_dists();
    *map.dists
        .iter()
        .map(|row| row.iter().max().unwrap())
        .max()
        .unwrap() as i64
}

pub fn part2<R: std::io::BufRead>(reader: R) -> i64 {
    let lines = reader.lines().map(|l| l.unwrap());
    let mut map = Map::from_lines(lines);
    map.compute_dists();
    map.remove_non_wall();
    let mut inside_count: i64 = 0;
    for r in 0..map.n_rows {
        for c in 0..map.n_cols {
            if map.inside((r, c)) {
                // println!("inside: {r} {c}");
                inside_count += 1;
            }
        }
    }
    inside_count
}

#[cfg(test)]
mod tests {
    const EXAMPLE1: &str = "\
-L|F7
7S-7|
L|7||
-L-J|
L|-JF
";

    const EXAMPLE2: &str = "\
7-F7-
.FJ|7
SJLL7
|F--J
LJ.LJ
";

    #[test]
    fn test_part1_example1() {
        let reader = std::io::BufReader::new(EXAMPLE1.as_bytes());
        let answer = super::part1(reader);
        assert_eq!(answer, 4);
    }

    #[test]
    fn test_part1_example2() {
        let reader = std::io::BufReader::new(EXAMPLE2.as_bytes());
        let answer = super::part1(reader);
        assert_eq!(answer, 8);
    }

    #[test]
    fn test_part1_big() {
        let reader = std::io::BufReader::new(include_str!("big.txt").as_bytes());
        let answer = super::part1(reader);
        assert_eq!(answer, 6640);
    }

    const PART2_EXAMPLES: [(i64, &str); 4] = [
        (
            4,
            "\
...........
.S-------7.
.|F-----7|.
.||OOOOO||.
.||OOOOO||.
.|L-7OF-J|.
.|II|O|II|.
.L--JOL--J.
.....O.....
",
        ),
        (
            4,
            "\
..........
.S------7.
.|F----7|.
.||OOOO||.
.||OOOO||.
.|L-7F-J|.
.|II||II|.
.L--JL--J.
..........
",
        ),
        (
            8,
            "\
.F----7F7F7F7F-7....
.|F--7||||||||FJ....
.||.FJ||||||||L7....
FJL7L7LJLJ||LJ.L-7..
L--J.L7...LJS7F-7L7.
....F-J..F7FJ|L7L7L7
....L7.F7||L7|.L7L7|
.....|FJLJ|FJ|F7|.LJ
....FJL-7.||.||||...
....L---J.LJ.LJLJ...
",
        ),
        (
            10,
            "\
FF7FSF7F7F7F7F7F---7
L|LJ||||||||||||F--J
FL-7LJLJ||||||LJL-77
F--JF--7||LJLJ7F7FJ-
L---JF-JLJ.||-FJLJJ7
|F|F-JF---7F7-L7L|7|
|FFJF7L7F-JF7|JL---7
7-L-JL7||F7|L7F-7F7|
L.L7LFJ|||||FJL7||LJ
L7JLJL-JLJLJL--JLJ.L
",
        ),
    ];

    #[test]
    fn test_part2_examples() {
        for (expected, input) in PART2_EXAMPLES.iter() {
            let reader = std::io::BufReader::new(input.as_bytes());
            let answer = super::part2(reader);
            assert_eq!(answer, *expected, "\nexpected: {expected}\ninput:\n{input}");
        }
    }

    #[test]
    fn test_part2_big() {
        let reader = std::io::BufReader::new(include_str!("big.txt").as_bytes());
        let answer = super::part2(reader);
        assert_eq!(answer, 411);
    }
}
