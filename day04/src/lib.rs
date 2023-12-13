use std::io::BufRead;

pub fn part1<R: std::io::Read>(reader: R) -> i64 {
    let reader = std::io::BufReader::new(reader);
    let mut total_points = 0;
    for line in reader.lines() {
        let line = line.unwrap();
        let (_, card_contents) = line.split_once(':').unwrap();
        let (winning_numbers, your_hand) = card_contents.split_once("|").unwrap();
        let winning_numbers = winning_numbers
            .trim()
            .split_ascii_whitespace()
            .map(|s| s.parse::<i64>().unwrap())
            .collect::<std::collections::HashSet<_>>();
        let your_hand = your_hand
            .trim()
            .split_ascii_whitespace()
            .map(|s| s.parse::<i64>().unwrap())
            .collect::<std::collections::HashSet<_>>();
        let exponent: i64 = winning_numbers
            .intersection(&your_hand)
            .count()
            .try_into()
            .unwrap();
        let score = if exponent > 0 {
            2i64.pow((exponent - 1).try_into().unwrap())
        } else {
            0
        };
        total_points += score;
    }
    total_points
}

pub fn part2<R: std::io::Read>(reader: R) -> i64 {
    let reader = std::io::BufReader::new(reader);
    let mut total_points = 0;
    let mut copy_count = std::collections::HashMap::<i64, i64>::new();
    for (card_number, line) in reader.lines().enumerate() {
        let card_number: i64 = (card_number + 1).try_into().unwrap();
        let card_count = 1 + copy_count.get(&card_number).unwrap_or(&0i64);
        total_points += card_count;
        let line = line.unwrap();
        let (_, card_contents) = line.split_once(':').unwrap();
        let (winning_numbers, your_hand) = card_contents.split_once("|").unwrap();
        let winning_numbers = winning_numbers
            .trim()
            .split_ascii_whitespace()
            .map(|s| s.parse::<i64>().unwrap())
            .collect::<std::collections::HashSet<_>>();
        let your_hand = your_hand
            .trim()
            .split_ascii_whitespace()
            .map(|s| s.parse::<i64>().unwrap())
            .collect::<std::collections::HashSet<_>>();
        let match_count: i64 = winning_numbers
            .intersection(&your_hand)
            .count()
            .try_into()
            .unwrap();
        for index in (card_number + 1)..(card_number + 1 + match_count) {
            *copy_count.entry(index).or_insert(0) += card_count;
        }
    }
    total_points
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_part1_example() {
        let input = include_str!("example1.txt");
        let result = super::part1(std::io::Cursor::new(input));
        assert_eq!(result, 13);
    }
    #[test]
    fn test_part1_final() {
        let input = include_str!("big.txt");
        let result = super::part1(std::io::Cursor::new(input));
        assert_eq!(result, 23941);
    }
    #[test]
    fn test_part2_example() {
        let input = include_str!("example1.txt");
        let result = super::part2(std::io::Cursor::new(input));
        assert_eq!(result, 30);
    }
    #[test]
    fn test_part2_final() {
        let input = include_str!("big.txt");
        let result = super::part2(std::io::Cursor::new(input));
        assert_eq!(result, 5571760);
    }
}
