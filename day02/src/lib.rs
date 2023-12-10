use std::io::BufRead;

pub fn part1<R: std::io::Read>(reader: R) -> i64 {
    let mut answer = 0;
    for line in std::io::BufReader::new(reader).lines() {
        let line = line.unwrap();
        let (left, right) = line.split_once(':').unwrap();
        let (_, game_id) = left.split_once(' ').unwrap();
        let game_id = game_id.parse::<i64>().unwrap();
        let mut valid = true;
        for reveal_description in right.split(';') {
            for set_description in reveal_description.split(',') {
                let (count, type_) = set_description.trim().split_once(' ').unwrap();
                let count = count.trim().parse::<i64>().unwrap();
                let type_ = type_.trim();
                if (type_ == "red" && count > 12)
                    || (type_ == "green" && count > 13)
                    || (type_ == "blue" && count > 14)
                {
                    valid = false;
                }
            }
        }
        // println!("game {game_id}: {valid}");
        if valid {
            answer += game_id;
        }
    }
    answer
}

pub fn part2<R: std::io::Read>(reader: R) -> i64 {
    let mut answer = 0;
    for line in std::io::BufReader::new(reader).lines() {
        let line = line.unwrap();
        let (_, right) = line.split_once(':').unwrap();
        let mut map = vec![("red", 0), ("blue", 0), ("green", 0)]
            .into_iter()
            .collect::<std::collections::HashMap<&str, i64>>();
        for reveal_description in right.split(';') {
            for set_description in reveal_description.split(',') {
                let (count, type_) = set_description.trim().split_once(' ').unwrap();
                let count = count.trim().parse::<i64>().unwrap();
                let type_ = type_.trim();
                *map.get_mut(type_).unwrap() = std::cmp::max(map[type_], count);
            }
        }
        let power = map.values().product::<i64>();
        // println!("game {game_id}: {power}");
        answer += power;
    }
    answer
}

#[cfg(test)]
mod tests {
    #[test]
    fn part1_example() {
        let input = std::io::Cursor::new(include_str!("example1.txt"));
        assert_eq!(super::part1(input), 8);
    }
    #[test]
    fn part1() {
        let input = std::io::Cursor::new(include_str!("big.txt"));
        assert_eq!(super::part1(input), 2771);
    }
    #[test]
    fn part2_example() {
        let input = std::io::Cursor::new(include_str!("example1.txt"));
        assert_eq!(super::part2(input), 2286);
    }
    #[test]
    fn part2() {
        let input = std::io::Cursor::new(include_str!("big.txt"));
        assert_eq!(super::part2(input), 70924);
    }
}
