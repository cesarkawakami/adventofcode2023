use ndarray::s;

fn parse<R: std::io::BufRead>(reader: R) -> ndarray::Array2<u8> {
    let mut map = ndarray::Array2::<u8>::zeros((0, 0));
    for line in reader.lines() {
        let line = line
            .unwrap()
            .trim()
            .chars()
            .map(|c| c as u8)
            .collect::<ndarray::Array1<_>>();
        if line.len() != map.ncols() {
            map = map.into_shape((0, line.len())).unwrap();
        }
        map.push_row(line.view()).unwrap();
    }
    map
}

fn tilt_up(map: &mut ndarray::Array2<u8>) {
    let (nrows, ncols) = map.dim();
    for r in 0..nrows {
        for c in 0..ncols {
            for nr in (1..=r).rev() {
                if map[(nr, c)] == b'O' && map[(nr - 1, c)] == b'.' {
                    map.swap((nr, c), (nr - 1, c));
                }
            }
        }
    }
}

fn load(map: &ndarray::Array2<u8>) -> usize {
    let (nrows, ncols) = map.dim();
    let mut result = 0;
    for r in 0..nrows {
        let row_score = nrows - r;
        for c in 0..ncols {
            if map[(r, c)] == b'O' {
                result += row_score;
            }
        }
    }
    result
}

pub fn part1<R: std::io::BufRead>(reader: R) -> usize {
    let mut map = parse(reader);
    tilt_up(&mut map);
    load(&map)
}

fn rotate_clockwise(map: &mut ndarray::Array2<u8>) {
    let map2 = map.t();
    let map2 = map2.slice(s![.., ..; -1]);
    *map = map2.into_owned();
}

pub fn part2<R: std::io::BufRead>(reader: R) -> usize {
    let mut map = parse(reader);

    let mut cycle_count: usize = 0;
    for _ in 0..1000 {
        for _ in 0..4 {
            tilt_up(&mut map);
            rotate_clockwise(&mut map);
        }
        cycle_count += 1;
    }

    let map_cycle_start = map.to_owned();
    let mut cycle_length = 0;
    loop {
        for _ in 0..4 {
            tilt_up(&mut map);
            rotate_clockwise(&mut map);
        }
        cycle_count += 1;
        cycle_length += 1;
        if map == map_cycle_start {
            break;
        }
    }

    cycle_count += (1_000_000_000usize - cycle_count) / cycle_length * cycle_length;

    for _ in cycle_count..1_000_000_000usize {
        for _ in 0..4 {
            tilt_up(&mut map);
            rotate_clockwise(&mut map);
        }
    }

    load(&map)
}

#[cfg(test)]
mod tests {
    const EXAMPLE1: &str = "\
O....#....
O.OO#....#
.....##...
OO.#O....O
.O.....O#.
O.#..O.#.#
..O..#O..O
.......O..
#....###..
#OO..#....
";

    #[test]
    fn part1_example() {
        let reader = std::io::BufReader::new(EXAMPLE1.as_bytes());
        let result = super::part1(reader);
        assert_eq!(result, 136);
    }

    #[test]
    fn part1_big() {
        let reader = std::io::BufReader::new(include_str!("big.txt").as_bytes());
        let result = super::part1(reader);
        assert_eq!(result, 113424);
    }

    #[test]
    fn part2_example() {
        let reader = std::io::BufReader::new(EXAMPLE1.as_bytes());
        let result = super::part2(reader);
        assert_eq!(result, 64);
    }

    #[test]
    fn part2_big() {
        let reader = std::io::BufReader::new(include_str!("big.txt").as_bytes());
        let result = super::part2(reader);
        assert_eq!(result, 96003);
    }
}
