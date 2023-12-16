use std::io::BufRead;

pub fn part1<R: std::io::Read>(reader: R) -> i64 {
    let reader = std::io::BufReader::new(reader);
    let mut line_iter = reader.lines().map(|x| x.unwrap());
    let time_line = line_iter.next().unwrap();
    let distance_line = line_iter.next().unwrap();
    let races = time_line
        .split_ascii_whitespace()
        .zip(distance_line.split_ascii_whitespace())
        .skip(1)
        .map(|(t, d)| (t.parse::<i64>().unwrap(), d.parse::<i64>().unwrap()))
        .collect::<Vec<_>>();
    let mut winner_counts = vec![];
    for (total_time, record_distance) in races {
        let mut winner_count = 0;
        for t in 1..total_time {
            let d = (total_time - t) * t;
            if d > record_distance {
                winner_count += 1;
            }
        }
        winner_counts.push(winner_count);
    }
    winner_counts.into_iter().product()
}

pub fn part2<R: std::io::Read>(reader: R) -> i64 {
    let reader = std::io::BufReader::new(reader);
    let mut line_iter = reader.lines().map(|x| x.unwrap());
    let time_line = line_iter.next().unwrap();
    let distance_line = line_iter.next().unwrap();
    let parse_line = |line: String| {
        line.split_ascii_whitespace()
            .skip(1)
            .collect::<String>()
            .parse::<i64>()
            .unwrap()
    };
    let total_time = parse_line(time_line);
    let record_distance = parse_line(distance_line);
    let mut winner_count = 0;
    for t in 1..total_time {
        let d = (total_time - t) * t;
        if d > record_distance {
            winner_count += 1;
        }
    }
    winner_count
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_part1_example() {
        let input = include_str!("example1.txt");
        let answer = super::part1(input.as_bytes());
        assert_eq!(answer, 288);
    }

    #[test]
    fn test_part1_final() {
        let input = include_str!("final.txt");
        let answer = super::part1(input.as_bytes());
        assert_eq!(answer, 1155175);
    }

    #[test]
    fn test_part2_example() {
        let input = include_str!("example1.txt");
        let answer = super::part2(input.as_bytes());
        assert_eq!(answer, 71503);
    }

    #[test]
    fn test_part2_final() {
        let input = include_str!("final.txt");
        let answer = super::part2(input.as_bytes());
        assert_eq!(answer, 35961505);
    }
}
