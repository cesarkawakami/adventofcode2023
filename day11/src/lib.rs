use std::collections::BTreeSet;

pub fn solve<R: std::io::BufRead>(reader: R, factor: usize) -> i64 {
    let map = reader
        .lines()
        .map(|l| l.unwrap().chars().collect::<Vec<_>>())
        .collect::<Vec<_>>();
    let nrows = map.len();
    let ncols = map[0].len();
    let mut galaxies = vec![];
    let mut rows_with_galaxies = BTreeSet::new();
    let mut cols_with_galaxies = BTreeSet::new();
    for (r, row) in map.iter().enumerate() {
        for (c, &val) in row.iter().enumerate() {
            if val == '#' {
                galaxies.push((r, c));
                rows_with_galaxies.insert(r);
                cols_with_galaxies.insert(c);
            }
        }
    }
    let rows_without_galaxies = (0..nrows)
        .collect::<BTreeSet<_>>()
        .difference(&rows_with_galaxies)
        .cloned()
        .collect::<BTreeSet<_>>();
    let cols_without_galaxies = (0..ncols)
        .collect::<BTreeSet<_>>()
        .difference(&cols_with_galaxies)
        .cloned()
        .collect::<BTreeSet<_>>();

    let mut answer = 0;
    for (i1, &galaxy1) in galaxies.iter().enumerate() {
        for &galaxy2 in galaxies.iter().skip(i1 + 1) {
            let (r1, c1) = galaxy1;
            let (r2, c2) = galaxy2;
            let (r1, r2) = (r1.min(r2), r1.max(r2));
            let (c1, c2) = (c1.min(c2), c1.max(c2));
            let dr = r1.abs_diff(r2);
            let dc = c1.abs_diff(c2);
            let expdr = dr + (factor - 1) * rows_without_galaxies.range(r1..r2).count();
            let expdc = dc + (factor - 1) * cols_without_galaxies.range(c1..c2).count();
            answer += expdr + expdc;
        }
    }

    answer as i64
}

pub fn part1<R: std::io::BufRead>(reader: R) -> i64 {
    solve(reader, 2)
}

pub fn part2<R: std::io::BufRead>(reader: R) -> i64 {
    solve(reader, 1_000_000)
}

#[cfg(test)]
mod tests {
    const EXAMPLE1: &str = "\
...#......
.......#..
#.........
..........
......#...
.#........
.........#
..........
.......#..
#...#.....
";

    #[test]
    fn part1_example() {
        let reader = std::io::BufReader::new(EXAMPLE1.as_bytes());
        let result = super::part1(reader);
        assert_eq!(result, 374);
    }

    #[test]
    fn part1_big() {
        let reader = std::io::BufReader::new(include_str!("big.txt").as_bytes());
        let result = super::part1(reader);
        assert_eq!(result, 9536038);
    }

    #[test]
    fn part2_example() {
        let reader = std::io::BufReader::new(EXAMPLE1.as_bytes());
        let result = super::solve(reader, 10);
        assert_eq!(result, 1030);
    }

    #[test]
    fn part2_example2() {
        let reader = std::io::BufReader::new(EXAMPLE1.as_bytes());
        let result = super::solve(reader, 100);
        assert_eq!(result, 8410);
    }

    #[test]
    fn part2_big() {
        let reader = std::io::BufReader::new(include_str!("big.txt").as_bytes());
        let result = super::part2(reader);
        assert_eq!(result, 447744640566);
    }
}
