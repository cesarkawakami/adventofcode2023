use std::io::Write;

pub fn part1<R: std::io::BufRead>(reader: R) -> i64 {
    let mut answer = 0;
    for line in reader.lines() {
        let line = line.unwrap();
        let mut vals: Vec<Vec<i64>> = vec![line
            .split_ascii_whitespace()
            .map(|s| s.parse::<i64>().unwrap())
            .collect()];
        while vals
            .last()
            .unwrap()
            .iter()
            .any(|v| v != vals.last().unwrap().first().unwrap())
        {
            vals.push(
                vals.last()
                    .unwrap()
                    .iter()
                    .zip(vals.last().unwrap().iter().skip(1))
                    .map(|(a, b)| b - a)
                    .collect(),
            )
        }
        let v = *vals.last().unwrap().last().unwrap();
        vals.last_mut().unwrap().push(v);
        for i in (0..(vals.len() - 1)).rev() {
            let v = vals[i].last().unwrap() + vals[i + 1].last().unwrap();
            vals[i].push(v);
        }
        answer += vals[0].last().unwrap();
    }
    answer
}

pub fn part2<R: std::io::BufRead>(reader: R) -> i64 {
    let mut writer = std::io::Cursor::new(Vec::<u8>::new());
    for line in reader.lines() {
        let rev_line: String = line
            .unwrap()
            .trim()
            .split_ascii_whitespace()
            .rev()
            .collect::<Vec<_>>()
            .join(" ");
        writeln!(writer, "{}", rev_line).unwrap();
    }
    writer.set_position(0);
    part1(writer)
}

#[cfg(test)]
mod tests {
    const EXAMPLE1: &str = "\
0 3 6 9 12 15
1 3 6 10 15 21
10 13 16 21 30 45
";

    #[test]
    fn test_part1_example() {
        let reader = std::io::BufReader::new(EXAMPLE1.as_bytes());
        let answer = super::part1(reader);
        assert_eq!(answer, 114);
    }

    #[test]
    fn test_part1_big() {
        let reader = std::io::BufReader::new(include_str!("big.txt").as_bytes());
        let answer = super::part1(reader);
        assert_eq!(answer, 1916822650);
    }

    #[test]
    fn test_part2_example() {
        let reader = std::io::BufReader::new(EXAMPLE1.as_bytes());
        let answer = super::part2(reader);
        assert_eq!(answer, 2);
    }

    #[test]
    fn test_part2_big() {
        let reader = std::io::BufReader::new(include_str!("big.txt").as_bytes());
        let answer = super::part2(reader);
        assert_eq!(answer, 966);
    }
}
