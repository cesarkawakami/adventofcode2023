#![feature(iter_array_chunks)]

use std::io::BufRead;

#[derive(Debug, Clone, Copy, Hash)]
struct Range(i64, i64);

#[derive(Debug, Clone, Copy)]
struct ShuffleOp {
    source_start: i64,
    source_end: i64,
    destination_start: i64,
}

impl ShuffleOp {
    fn map(&self, value: i64) -> Option<i64> {
        // println!("mapping {value} with {self:?}");
        if value >= self.source_start && value < self.source_end {
            Some(value - self.source_start + self.destination_start)
        } else {
            None
        }
    }

    fn map_range(
        &self,
        Range(range_start, range_end): Range,
    ) -> (Option<Range>, Option<Range>, Option<Range>) {
        let source_start = self.source_start;
        let source_end = self.source_end;

        // println!("input: {range_start}-{range_end}");
        // println!("self: {self:?}");

        let (left_remnant, range_start) = if range_start < source_start {
            (
                Some(Range(range_start, std::cmp::min(range_end, source_start))),
                source_start,
            )
        } else {
            (None, range_start)
        };

        let (right_remnant, range_end) = if range_end > source_end {
            (
                Some(Range(std::cmp::max(range_start, source_end), range_end)),
                source_end,
            )
        } else {
            (None, range_end)
        };

        let moved_center = if range_start < range_end {
            Some(Range(
                range_start - source_start + self.destination_start,
                range_end - source_start + self.destination_start,
            ))
        } else {
            None
        };

        // println!("left remnant: {left_remnant:?}");
        // println!("right remnant: {right_remnant:?}");
        // println!("moved_center: {moved_center:?}");

        // println!("result: {result:?}");

        (moved_center, left_remnant, right_remnant)
    }
}

#[derive(Debug)]
struct Map {
    _source: String,
    destination: String,
    shuffle_ops: Vec<ShuffleOp>,
}

impl Map {
    fn map(&self, value: i64) -> i64 {
        for shuffle_op in &self.shuffle_ops {
            if let Some(mapped_value) = shuffle_op.map(value) {
                return mapped_value;
            }
        }
        value
    }

    fn map_range(&self, range: Range) -> impl Iterator<Item = Range> {
        let mut untouched = vec![range];
        let mut touched = vec![];
        for shuffle_op in &self.shuffle_ops {
            untouched = untouched
                .into_iter()
                .flat_map(|r| {
                    let (a, b, c) = shuffle_op.map_range(r);
                    touched.extend(a);
                    b.into_iter().chain(c)
                })
                .collect();
        }
        touched.into_iter().chain(untouched)
    }

    fn map_ranges(&self, ranges: Vec<Range>) -> Vec<Range> {
        ranges.into_iter().flat_map(|r| self.map_range(r)).collect()
    }
}

fn parse_input<R: std::io::Read>(reader: R) -> (Vec<i64>, std::collections::HashMap<String, Map>) {
    let reader = std::io::BufReader::new(reader);

    let mut seeds = vec![];
    let mut maps = std::collections::HashMap::<String, Map>::new();
    let mut current_map: Option<&mut Map> = None;

    for line in reader.lines() {
        let line = line.unwrap().trim().to_string();

        if line.is_empty() {
            continue;
        } else if line.starts_with("seeds:") {
            seeds = line
                .split_once(':')
                .unwrap()
                .1
                .split_ascii_whitespace()
                .map(|x| x.parse::<i64>().unwrap())
                .collect();
        } else if line.contains(" map:") {
            let (source, destination) = line.split_once(' ').unwrap().0.split_once("-to-").unwrap();
            maps.insert(
                source.into(),
                Map {
                    _source: source.into(),
                    destination: destination.into(),
                    shuffle_ops: vec![],
                },
            );
            current_map = maps.get_mut(source);
        } else {
            let (destination_start, source_start, length) = {
                let mut iter = line.split_ascii_whitespace();
                (
                    iter.next().unwrap().parse::<i64>().unwrap(),
                    iter.next().unwrap().parse::<i64>().unwrap(),
                    iter.next().unwrap().parse::<i64>().unwrap(),
                )
            };
            let Some(ref mut v) = current_map else {
                panic!("no current map");
            };
            v.shuffle_ops.push(ShuffleOp {
                source_start,
                source_end: source_start + length,
                destination_start,
            });
        }
    }
    (seeds, maps)
}

pub fn part1<R: std::io::Read>(reader: R) -> i64 {
    let (seeds, maps) = parse_input(reader);

    let mut current_category = "seed";
    let mut current_values = seeds.clone();
    while current_category != "location" {
        println!("current category: {current_category}");
        println!("current values: {current_values:?}");
        let map = &maps[current_category];
        let next_category = map.destination.as_str();
        let next_values = current_values
            .iter()
            .map(|&x| map.map(x))
            .collect::<Vec<_>>();
        current_category = next_category;
        current_values = next_values;
    }

    println!("final locations: {current_values:?}");

    *current_values.iter().min().unwrap()
}

pub fn part2<R: std::io::Read>(reader: R) -> i64 {
    let (seeds, maps) = parse_input(reader);
    let mut current_ranges = seeds
        .into_iter()
        .array_chunks::<2>()
        .map(|[a, b]| Range(a, a + b))
        .collect::<Vec<_>>();

    let mut current_category = "seed";
    while current_category != "location" {
        println!("current category: {current_category}");
        println!("current values: {current_ranges:?}");
        let map = &maps[current_category];
        current_category = map.destination.as_str();
        current_ranges = map.map_ranges(current_ranges);
    }

    println!("final locations: {current_ranges:?}");

    current_ranges
        .into_iter()
        .flat_map(|Range(a, b)| if a < b { Some(a) } else { None })
        .min()
        .unwrap()
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_part1_example() {
        let input = include_str!("example1.txt");
        let answer = super::part1(input.as_bytes());
        assert_eq!(answer, 35);
    }
    #[test]
    fn test_part1_final() {
        let input = include_str!("big.txt");
        let answer = super::part1(input.as_bytes());
        assert_eq!(answer, 825516882);
    }
    #[test]
    fn test_part2_example() {
        let input = include_str!("example1.txt");
        let answer = super::part2(input.as_bytes());
        assert_eq!(answer, 46);
    }
    #[test]
    fn test_part2_final() {
        let input = include_str!("big.txt");
        let answer = super::part2(input.as_bytes());
        assert_eq!(answer, 136096660);
    }
}
