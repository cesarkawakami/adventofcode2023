use std::io::BufRead;

fn symbol_in_region(board: &[Vec<char>], ra: i64, rb: i64, ca: i64, cb: i64) -> bool {
    let ra = ra.clamp(0, (board.len() - 1).try_into().unwrap());
    let rb = rb.clamp(0, (board.len() - 1).try_into().unwrap());
    let ca = ca.clamp(0, (board[0].len() - 1).try_into().unwrap());
    let cb = cb.clamp(0, (board[0].len() - 1).try_into().unwrap());
    for r in ra..=rb {
        for c in ca..=cb {
            let val = board[usize::try_from(r).unwrap()][usize::try_from(c).unwrap()];
            if !val.is_ascii_digit() && val != '.' && !val.is_whitespace() {
                return true;
            }
        }
    }
    false
}

fn gears_in_region(board: &[Vec<char>], ra: i64, rb: i64, ca: i64, cb: i64) -> Vec<(i64, i64)> {
    let ra = ra.clamp(0, (board.len() - 1).try_into().unwrap());
    let rb = rb.clamp(0, (board.len() - 1).try_into().unwrap());
    let ca = ca.clamp(0, (board[0].len() - 1).try_into().unwrap());
    let cb = cb.clamp(0, (board[0].len() - 1).try_into().unwrap());
    let mut rv = vec![];
    for r in ra..=rb {
        for c in ca..=cb {
            let val = board[usize::try_from(r).unwrap()][usize::try_from(c).unwrap()];
            if val == '*' {
                rv.push((r, c));
            }
        }
    }
    rv
}

pub fn part1<R: std::io::Read>(reader: R) -> i64 {
    let board: Vec<Vec<char>> = std::io::BufReader::new(reader)
        .lines()
        .map(|line| line.unwrap().trim().chars().collect())
        .collect();
    let mut answer = 0;
    for (r, line) in board.iter().enumerate() {
        let r: i64 = r.try_into().unwrap();
        let line: String = line.iter().collect();
        for match_ in regex::Regex::new(r"\d+").unwrap().find_iter(&line) {
            let ca: i64 = match_.start().try_into().unwrap();
            let cb: i64 = match_.end().try_into().unwrap();
            let cb = cb - 1;
            let number: i64 = match_.as_str().parse().unwrap();
            let has_symbol = symbol_in_region(&board, r - 1, r + 1, ca - 1, cb + 1);
            println!("number={number} has_symbol={has_symbol}");
            if has_symbol {
                answer += number;
            }
        }
    }
    answer
}

pub fn part2<R: std::io::Read>(reader: R) -> i64 {
    let board: Vec<Vec<char>> = std::io::BufReader::new(reader)
        .lines()
        .map(|line| line.unwrap().trim().chars().collect())
        .collect();
    let mut gear_candidates = std::collections::HashMap::<(i64, i64), Vec<i64>>::new();
    for (r, line) in board.iter().enumerate() {
        let r: i64 = r.try_into().unwrap();
        let line: String = line.iter().collect();
        for match_ in regex::Regex::new(r"\d+").unwrap().find_iter(&line) {
            let ca: i64 = match_.start().try_into().unwrap();
            let cb: i64 = match_.end().try_into().unwrap();
            let cb = cb - 1;
            let number: i64 = match_.as_str().parse().unwrap();
            for gear_position in gears_in_region(&board, r - 1, r + 1, ca - 1, cb + 1) {
                gear_candidates
                    .entry(gear_position)
                    .or_default()
                    .push(number);
            }
        }
    }
    let mut answer = 0;
    for (_, candidates) in gear_candidates {
        if candidates.len() == 2 {
            answer += candidates[0] * candidates[1];
        }
    }
    answer
}

#[cfg(test)]
mod tests {
    #[test]
    fn part1_example() {
        let input = std::io::Cursor::new(include_str!("example1.txt"));
        assert_eq!(super::part1(input), 4361);
    }
    #[test]
    fn part1() {
        let input = std::io::Cursor::new(include_str!("big.txt"));
        assert_eq!(super::part1(input), 550064);
    }
    #[test]
    fn part2_example() {
        let input = std::io::Cursor::new(include_str!("example1.txt"));
        assert_eq!(super::part2(input), 467835);
    }
    #[test]
    fn part2() {
        let input = std::io::Cursor::new(include_str!("big.txt"));
        assert_eq!(super::part2(input), 85010461);
    }
}
