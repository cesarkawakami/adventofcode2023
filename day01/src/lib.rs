use std::io::BufRead;

pub fn part1<R: std::io::Read>(reader: R) -> i64 {
    let answer = std::io::BufReader::new(reader)
        .lines()
        .map(|line| {
            let number_str: String = line
                .unwrap()
                .chars()
                .filter(|c| c.is_ascii_digit())
                .collect();
            let number_str: String = number_str[..1]
                .chars()
                .chain(number_str[number_str.len() - 1..].chars())
                .collect();
            number_str.parse::<i64>().unwrap()
        })
        .sum::<i64>();
    answer
}

pub fn part2<R: std::io::Read>(reader: R) -> i64 {
    std::io::BufReader::new(reader)
        .lines()
        .map(|line| {
            let line = line.unwrap();
            let mut min_match: (i64, i64) = (999999999, -1);
            let mut max_match: (i64, i64) = (-1, -1);
            for (i, pats) in vec![
                // (0, vec!["0"]),
                (1, vec!["1", "one"]),
                (2, vec!["2", "two"]),
                (3, vec!["3", "three"]),
                (4, vec!["4", "four"]),
                (5, vec!["5", "five"]),
                (6, vec!["6", "six"]),
                (7, vec!["7", "seven"]),
                (8, vec!["8", "eight"]),
                (9, vec!["9", "nine"]),
            ] {
                for pat in pats {
                    if let Some(pos) = line.find(pat) {
                        min_match = std::cmp::min(min_match, (pos.try_into().unwrap(), i.into()));
                    };
                    if let Some(pos) = line.rfind(pat) {
                        max_match = std::cmp::max(max_match, (pos.try_into().unwrap(), i.into()));
                    }
                }
            }
            let number_str = format!("{}{}", min_match.1, max_match.1);
            // eprintln!("{}", number_str);
            number_str.parse::<i64>().unwrap()
        })
        .sum::<i64>()
}

#[cfg(test)]
mod tests {

    const EXAMPLE1: &str = "\
1abc2
pqr3stu8vwx
a1b2c3d4e5f
treb7uchet
";

    const EXAMPLE2: &str = "\
two1nine
eightwothree
abcone2threexyz
xtwone3four
4nineeightseven2
zoneight234
7pqrstsixteen
";

    #[test]
    fn part1_example() {
        let answer = super::part1(std::io::Cursor::new(EXAMPLE1));
        assert_eq!(answer, 142);
    }

    #[test]
    fn part1() {
        let input = include_str!("big_input.txt");
        let answer = super::part1(std::io::Cursor::new(input));
        assert_eq!(answer, 54601);
    }

    #[test]
    fn part2_example() {
        let answer = super::part2(std::io::Cursor::new(EXAMPLE2));
        assert_eq!(answer, 281);
    }

    #[test]
    fn part2() {
        let input = include_str!("big_input.txt");
        let answer = super::part2(std::io::Cursor::new(input));
        assert_eq!(answer, 54078);
    }
}
