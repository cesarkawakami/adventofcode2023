fn compatible(states: &[char], group_size: usize, position: usize) -> bool {
    if position + group_size > states.len() {
        return false;
    }
    if states[..position].iter().any(|&c| c == '#') {
        return false;
    }
    if states[position..position + group_size]
        .iter()
        .any(|&c| c == '.')
    {
        return false;
    }
    if position + group_size < states.len() && states[position + group_size] == '#' {
        return false;
    }

    true
}

struct ArrangeCounter<'a> {
    memo: std::collections::HashMap<(&'a [char], &'a [usize]), usize>,
}

impl<'a> ArrangeCounter<'a> {
    fn new() -> Self {
        ArrangeCounter {
            memo: std::collections::HashMap::new(),
        }
    }

    fn _count_arrangements(&mut self, states: &'a [char], groups: &'a [usize]) -> usize {
        if groups.is_empty() {
            return if states.iter().all(|&c| c == '.' || c == '?') {
                1
            } else {
                0
            };
        }

        let group_size = groups[0];
        let mut count = 0;
        for position in 0..states.len() {
            if compatible(states, group_size, position) {
                count += self.count_arrangements(
                    &states[(position + group_size + 1).min(states.len())..],
                    &groups[1..],
                );
            }
        }

        count
    }

    fn count_arrangements(&mut self, states: &'a [char], groups: &'a [usize]) -> usize {
        if let Some(result) = self.memo.get(&(states, groups)) {
            *result
        } else {
            let result = self._count_arrangements(states, groups);
            self.memo.insert((states, groups), result);
            result
        }
    }
}

pub fn part1<R: std::io::BufRead>(reader: R) -> usize {
    let mut total = 0;
    for line in reader.lines() {
        let line = line.unwrap();
        let (states, groups) = line.trim().split_once(' ').unwrap();
        let states = states.chars().collect::<Vec<_>>();
        let groups = groups
            .split(',')
            .map(|s| s.parse::<usize>().unwrap())
            .collect::<Vec<_>>();

        let count = ArrangeCounter::new().count_arrangements(&states[..], &groups[..]);
        total += count;

        // println!("{states:?} {groups:?} => {count}");
    }
    total
}

pub fn part2<R: std::io::BufRead>(reader: R) -> usize {
    let mut total = 0;
    for line in reader.lines() {
        let line = line.unwrap();
        let (states, groups) = line.trim().split_once(' ').unwrap();
        let states = states.chars().collect::<Vec<_>>();
        let groups = groups
            .split(',')
            .map(|s| s.parse::<usize>().unwrap())
            .collect::<Vec<_>>();

        let states = (0..5)
            .flat_map(|i| {
                std::iter::repeat('?')
                    .take(if i == 0 { 0 } else { 1 })
                    .chain(states.iter().cloned())
            })
            .collect::<Vec<_>>();
        let groups = (0..5)
            .flat_map(|_| groups.iter().cloned())
            .collect::<Vec<_>>();

        let count = ArrangeCounter::new().count_arrangements(&states[..], &groups[..]);
        total += count;

        // println!("{states:?} {groups:?} => {count}");
    }
    total
}

#[cfg(test)]
mod tests {
    const EXAMPLE1: &str = "\
???.### 1,1,3
.??..??...?##. 1,1,3
?#?#?#?#?#?#?#? 1,3,1,6
????.#...#... 4,1,1
????.######..#####. 1,6,5
?###???????? 3,2,1
";

    #[test]
    fn part1_example() {
        let reader = std::io::BufReader::new(EXAMPLE1.as_bytes());
        let result = super::part1(reader);
        assert_eq!(result, 21);
    }

    #[test]
    fn part1_big() {
        let reader = std::io::BufReader::new(include_str!("big.txt").as_bytes());
        let result = super::part1(reader);
        assert_eq!(result, 7110);
    }

    #[test]
    fn part2_example() {
        let reader = std::io::BufReader::new(EXAMPLE1.as_bytes());
        let result = super::part2(reader);
        assert_eq!(result, 525152);
    }

    #[test]
    fn part2_big() {
        let reader = std::io::BufReader::new(include_str!("big.txt").as_bytes());
        let result = super::part2(reader);
        assert_eq!(result, 1566786613613);
    }
}
