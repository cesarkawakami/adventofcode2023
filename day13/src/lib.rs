use ndarray::s;

fn parse<R: std::io::BufRead>(mut reader: R) -> Vec<ndarray::Array2<u8>> {
    let mut maps = vec![];
    'outer: loop {
        let mut map = ndarray::Array2::zeros((0, 0));
        loop {
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            // println!("line: {line:#?}");
            if line.is_empty() {
                maps.push(map);
                break 'outer;
            }
            if line.trim().is_empty() {
                maps.push(map);
                break;
            }
            let line = line
                .trim()
                .chars()
                .map(|c| (c == '#') as u8)
                .collect::<ndarray::Array1<u8>>();
            if line.len() != map.ncols() {
                map = map.into_shape((0, line.len())).unwrap();
            }
            map.push_row(line.view()).unwrap();
        }
    }
    maps
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Refl {
    Hori(usize),
    Vert(usize),
}

impl Refl {
    fn score(&self) -> usize {
        match self {
            Refl::Hori(r) => 100 * r,
            Refl::Vert(c) => *c,
        }
    }
}

fn find_reflections(map: ndarray::ArrayView2<u8>) -> impl Iterator<Item = Refl> + '_ {
    std::iter::empty()
        .chain((1..map.nrows()).flat_map(move |r| {
            let (r1, r2) = if r > map.nrows() - r {
                (2 * r - map.nrows(), map.nrows())
            } else {
                (0, 2 * r)
            };
            if map.slice(s![r1..r, ..]) == map.slice(s![r..r2; -1, ..]) {
                Some(Refl::Hori(r))
            } else {
                None
            }
        }))
        .chain((1..map.ncols()).flat_map(move |c| {
            let (c1, c2) = if c > map.ncols() - c {
                (2 * c - map.ncols(), map.ncols())
            } else {
                (0, 2 * c)
            };
            if map.slice(s![.., c1..c]) == map.slice(s![.., c..c2; -1]) {
                Some(Refl::Vert(c))
            } else {
                None
            }
        }))
}

pub fn part1<R: std::io::BufRead>(reader: R) -> i64 {
    let maps = parse(reader);
    let mut result = 0;

    for map in maps {
        // println!("doing:\n{map:?}");

        let acc: usize = find_reflections(map.view()).map(|refl| refl.score()).sum();

        result += acc;
    }

    result as i64
}

pub fn part2<R: std::io::BufRead>(reader: R) -> i64 {
    let maps = parse(reader);
    let mut result = 0;

    for map in maps {
        // println!("doing:\n{map:?}");

        let old_refls = find_reflections(map.view()).collect::<Vec<_>>();

        for ((r, c), _) in map.indexed_iter() {
            let mut map = map.clone();
            map[(r, c)] = 1 - map[(r, c)];
            if let Some(new_refl) =
                find_reflections(map.view()).find(|refl| !old_refls.contains(refl))
            {
                result += new_refl.score();
                break;
            };
        }
    }

    result as i64
}

#[cfg(test)]
mod tests {
    const EXAMPLE1: &str = "\
#.##..##.
..#.##.#.
##......#
##......#
..#.##.#.
..##..##.
#.#.##.#.

#...##..#
#....#..#
..##..###
#####.##.
#####.##.
..##..###
#....#..#
";

    #[test]
    fn part1_example() {
        let reader = std::io::BufReader::new(EXAMPLE1.as_bytes());
        let result = super::part1(reader);
        assert_eq!(result, 405);
    }

    #[test]
    fn part1_big() {
        let reader = std::io::BufReader::new(include_str!("big.txt").as_bytes());
        let result = super::part1(reader);
        assert_eq!(result, 34772);
    }

    #[test]
    fn part2_example() {
        let reader = std::io::BufReader::new(EXAMPLE1.as_bytes());
        let result = super::part2(reader);
        assert_eq!(result, 400);
    }

    #[test]
    fn part2_big() {
        let reader = std::io::BufReader::new(include_str!("big.txt").as_bytes());
        let result = super::part2(reader);
        assert_eq!(result, 35554);
    }
}
