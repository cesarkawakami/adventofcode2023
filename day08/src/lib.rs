use num::Integer;

pub fn part1<R: std::io::BufRead>(reader: R) -> i64 {
    let mut lines = reader.lines();
    let instructions = lines.next().unwrap().unwrap();
    lines.next().unwrap().unwrap();

    let mut map = std::collections::HashMap::<String, (String, String)>::new();
    for line in lines {
        let line = line.unwrap();
        let (from, rest) = line.split_once(' ').unwrap();
        let (_, rest) = rest.split_once('(').unwrap();
        let (left, rest) = rest.split_once(',').unwrap();
        let (right, _) = rest.trim().split_once(')').unwrap();
        map.insert(from.to_string(), (left.to_string(), right.to_string()));
    }

    let mut current_node = "AAA";
    let mut step_count = 0;
    for instruction in instructions.chars().cycle() {
        if current_node == "ZZZ" {
            break;
        }

        current_node = match instruction {
            'L' => map.get(current_node).unwrap().0.as_str(),
            'R' => map.get(current_node).unwrap().1.as_str(),
            _ => panic!("Unknown instruction: {}", instruction),
        };
        step_count += 1;
    }

    step_count
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NamesTrajectory<'a> {
    head: Vec<&'a str>,
    cycle: Vec<&'a str>,
}

impl<'a> NamesTrajectory<'a> {
    fn calculate(
        map: &'a std::collections::HashMap<String, (String, String)>,
        instructions: &[char],
        start: &'a str,
    ) -> Self {
        let mut current_node = start;
        let mut current_ip: usize = 0;
        let mut head = Vec::<&str>::new();
        let mut seen = std::collections::HashMap::<(&str, usize), usize>::new();
        loop {
            if let Some(&loop_ip) = seen.get(&(current_node, current_ip)) {
                let cycle = head.split_off(loop_ip);
                return NamesTrajectory { head, cycle };
            }
            seen.insert((current_node, current_ip), head.len());
            head.push(current_node);
            match instructions[current_ip] {
                'L' => current_node = map.get(current_node).unwrap().0.as_str(),
                'R' => current_node = map.get(current_node).unwrap().1.as_str(),
                _ => panic!("Unknown instruction: {}", instructions[current_ip]),
            }
            current_ip = (current_ip + 1) % instructions.len();
        }
    }
}

type Index = u128;

#[derive(Debug, Clone, PartialEq, Eq)]
struct IndexesTrajectory {
    head: Vec<Index>,
    head_size: Index,
    cycle: Vec<Index>,
    cycle_size: Index,
}

impl IndexesTrajectory {
    fn from_names(names_trajectory: &NamesTrajectory) -> Self {
        let head: Vec<Index> = names_trajectory
            .head
            .iter()
            .enumerate()
            .filter(|(_, &v)| v.ends_with('Z'))
            .map(|(i, _)| i.try_into().unwrap())
            .collect();
        let cycle: Vec<Index> = names_trajectory
            .cycle
            .iter()
            .enumerate()
            .filter(|(_, &v)| v.ends_with('Z'))
            .map(|(i, _)| i.try_into().unwrap())
            .collect();

        IndexesTrajectory {
            head,
            head_size: names_trajectory.head.len().try_into().unwrap(),
            cycle,
            cycle_size: names_trajectory.cycle.len().try_into().unwrap(),
        }
    }

    fn lengthen_head(&self, delta: Index) -> IndexesTrajectory {
        let mut head = self.head.clone();
        let mut current_offset: Index = self.head.len().try_into().unwrap();
        let mut remaining_delta = delta;
        loop {
            head.extend(
                self.cycle
                    .iter()
                    .filter(|&&v| v < remaining_delta)
                    .map(|v| v + current_offset),
            );
            if remaining_delta <= self.cycle_size {
                break;
            } else {
                remaining_delta -= self.cycle_size;
                current_offset += self.cycle_size;
            }
        }
        let cycle: Vec<Index> = self
            .cycle
            .iter()
            .map(|v| (v + self.cycle_size - delta % self.cycle_size) % self.cycle_size)
            .collect();
        IndexesTrajectory {
            head,
            head_size: self.head_size + delta,
            cycle,
            cycle_size: self.cycle_size,
        }
    }

    fn merge(&self, other: &Self) -> Self {
        match self.head_size.cmp(&other.head_size) {
            std::cmp::Ordering::Less => {
                return self
                    .lengthen_head(other.head_size - self.head_size)
                    .merge(other);
            }
            std::cmp::Ordering::Greater => {
                return self.merge(&other.lengthen_head(self.head_size - other.head_size));
            }
            _ => {}
        }

        let self_head_members: std::collections::HashSet<Index> =
            self.head.iter().cloned().collect();
        let other_head_members: std::collections::HashSet<Index> =
            other.head.iter().cloned().collect();
        let mut head: Vec<Index> = self_head_members
            .intersection(&other_head_members)
            .cloned()
            .collect();
        head.sort();

        assert!(self.cycle.len() == 1);
        assert!(other.cycle.len() == 1);
        let a1: i128 = self.cycle[0].try_into().unwrap();
        let n1: i128 = self.cycle_size.try_into().unwrap();
        let a2: i128 = other.cycle[0].try_into().unwrap();
        let n2: i128 = other.cycle_size.try_into().unwrap();
        let num::integer::ExtendedGcd { gcd, x: m1, y: m2 } = n1.extended_gcd(&n2);
        assert!(a1 % gcd == a2 % gcd);
        assert!(m1 * n1 + m2 * n2 == gcd);
        let x = a1 * m2 * n2 + a2 * m1 * n1;
        assert!(x % gcd == 0);
        let x = x / gcd;
        let cycle_size = n1 * n2 / gcd;
        let x = x.rem_euclid(cycle_size);
        let x: Index = x.try_into().unwrap();
        let cycle_size: Index = cycle_size.try_into().unwrap();

        IndexesTrajectory {
            head,
            head_size: self.head_size,
            cycle: vec![x],
            cycle_size,
        }
    }

    fn merge_slow(&self, other: &Self) -> Self {
        match self.head_size.cmp(&other.head_size) {
            std::cmp::Ordering::Less => {
                return self
                    .lengthen_head(other.head_size - self.head_size)
                    .merge(other);
            }
            std::cmp::Ordering::Greater => {
                return self.merge(&other.lengthen_head(self.head_size - other.head_size));
            }
            _ => {}
        }

        let self_head_members: std::collections::HashSet<Index> =
            self.head.iter().cloned().collect();
        let other_head_members: std::collections::HashSet<Index> =
            other.head.iter().cloned().collect();
        let mut head: Vec<Index> = self_head_members
            .intersection(&other_head_members)
            .cloned()
            .collect();
        head.sort();

        let cycle_size = self.cycle_size.lcm(&other.cycle_size);
        let self_cycle_members: std::collections::HashSet<Index> = (0..(cycle_size
            / self.cycle_size))
            .flat_map(|i| self.cycle.iter().map(move |v| v + i * self.cycle_size))
            .collect();
        let other_cycle_members: std::collections::HashSet<Index> = (0..(cycle_size
            / other.cycle_size))
            .flat_map(|i| other.cycle.iter().map(move |v| v + i * other.cycle_size))
            .collect();
        let mut cycle: Vec<Index> = self_cycle_members
            .intersection(&other_cycle_members)
            .cloned()
            .collect();
        cycle.sort();

        IndexesTrajectory {
            head,
            head_size: self.head_size,
            cycle,
            cycle_size,
        }
    }

    fn iter(&self) -> impl Iterator<Item = Index> + '_ {
        let head_size = self.head_size;
        let cycle_size = self.cycle_size;
        self.head.iter().cloned().chain((0..).flat_map(move |i| {
            self.cycle
                .iter()
                .cloned()
                .map(move |v| head_size + v + i * cycle_size)
        }))
    }
}

pub fn part2_try1<R: std::io::BufRead>(reader: R) -> i128 {
    let mut lines = reader.lines();
    let instructions = lines.next().unwrap().unwrap();
    lines.next().unwrap().unwrap();

    let mut map = std::collections::HashMap::<String, (String, String)>::new();
    for line in lines {
        let line = line.unwrap();
        let (from, rest) = line.split_once(' ').unwrap();
        let (_, rest) = rest.split_once('(').unwrap();
        let (left, rest) = rest.split_once(',').unwrap();
        let (right, _) = rest.trim().split_once(')').unwrap();
        map.insert(from.to_string(), (left.to_string(), right.to_string()));
    }

    let mut current_node_set: std::collections::HashSet<String> = map
        .keys()
        .filter(|node| node.ends_with('A'))
        .cloned()
        .collect();
    let mut step_count = 0;
    for instruction in instructions.chars().cycle() {
        if current_node_set.iter().all(|node| node.ends_with('Z')) {
            break;
        }

        current_node_set = current_node_set
            .into_iter()
            .map(|node| match instruction {
                'L' => map.get(&node).unwrap().0.clone(),
                'R' => map.get(&node).unwrap().1.clone(),
                _ => panic!("Unknown instruction: {}", instruction),
            })
            .collect();
        step_count += 1;
    }

    step_count
}

pub fn part2_try2<R: std::io::BufRead>(reader: R) -> i128 {
    let mut lines = reader.lines();
    let instructions = lines.next().unwrap().unwrap();
    let instructions: Vec<char> = instructions.chars().collect();
    lines.next().unwrap().unwrap();

    let mut map = std::collections::HashMap::<String, (String, String)>::new();
    for line in lines {
        let line = line.unwrap();
        let (from, rest) = line.split_once(' ').unwrap();
        let (_, rest) = rest.split_once('(').unwrap();
        let (left, rest) = rest.split_once(',').unwrap();
        let (right, _) = rest.trim().split_once(')').unwrap();
        map.insert(from.to_string(), (left.to_string(), right.to_string()));
    }

    let trajectories: Vec<IndexesTrajectory> = map
        .keys()
        .filter(|v| v.ends_with('A'))
        .map(|v| NamesTrajectory::calculate(&map, instructions.as_slice(), v.as_str()))
        .map(|t| IndexesTrajectory::from_names(&t))
        .collect();

    eprintln!("Trajectories: {:#?}", trajectories);

    let merged = trajectories
        .iter()
        .skip(1)
        .fold(trajectories.first().unwrap().clone(), |a, b| a.merge(b));

    let rv = merged.iter().next().unwrap().try_into().unwrap();

    rv
}

#[cfg(test)]
mod tests {
    use crate::NamesTrajectory;

    #[test]
    fn test_names_trajectory() {
        let map = [
            ("A", ("B", "C")),
            ("B", ("A", "B")),
            ("C", ("C", "A")),
            ("Z", ("Z", "A")),
        ]
        .into_iter()
        .map(|(a, (b, c))| (a.to_string(), (b.to_string(), c.to_string())))
        .collect();
        let trajectory = NamesTrajectory::calculate(&map, &['L', 'R'], "Z");

        // LRLRLRLRLRL
        // ZZ[ABBACC]
        assert_eq!(
            trajectory,
            NamesTrajectory {
                head: vec!["Z", "Z"],
                cycle: vec!["A", "B", "B", "A", "C", "C"],
            }
        );
    }

    #[test]
    fn test_indexes_trajectory() {
        let trajectory1 = super::IndexesTrajectory {
            // 0123456 0123
            // 0010011[0011]
            head: vec![2, 5, 6],
            head_size: 7,
            cycle: vec![2, 3],
            cycle_size: 4,
        };
        let trajectory2 = super::IndexesTrajectory {
            // 01234567 012
            // 00011010[001]
            head: vec![3, 4, 6],
            head_size: 8,
            cycle: vec![2],
            cycle_size: 3,
        };
        let merged_trajectory = trajectory1.merge_slow(&trajectory2);
        let expected_trajectory = super::IndexesTrajectory {
            head: vec![6],
            head_size: 8,
            // 011001100110
            // 001001001001
            // 001001000000
            // 012345678901
            cycle: vec![2, 5],
            cycle_size: 12,
        };
        assert_eq!(merged_trajectory, expected_trajectory);
    }

    const EXAMPLE1: &str = "\
RL

AAA = (BBB, CCC)
BBB = (DDD, EEE)
CCC = (ZZZ, GGG)
DDD = (DDD, DDD)
EEE = (EEE, EEE)
GGG = (GGG, GGG)
ZZZ = (ZZZ, ZZZ)
";

    const EXAMPLE2: &str = "\
LLR

AAA = (BBB, BBB)
BBB = (AAA, ZZZ)
ZZZ = (ZZZ, ZZZ)
";

    #[test]
    fn test_part1_example1() {
        let reader = std::io::BufReader::new(EXAMPLE1.as_bytes());
        let answer = super::part1(reader);
        assert_eq!(answer, 2);
    }

    #[test]
    fn test_part1_example2() {
        let reader = std::io::BufReader::new(EXAMPLE2.as_bytes());
        let answer = super::part1(reader);
        assert_eq!(answer, 6);
    }

    #[test]
    fn test_part1_final() {
        let reader = std::io::BufReader::new(include_bytes!("final.txt").as_slice());
        let answer = super::part1(reader);
        assert_eq!(answer, 13771);
    }

    const EXAMPLE3: &str = "\
LR

11A = (11B, XXX)
11B = (XXX, 11Z)
11Z = (11B, XXX)
22A = (22B, XXX)
22B = (22C, 22C)
22C = (22Z, 22Z)
22Z = (22B, 22B)
XXX = (XXX, XXX)
";

    #[test]
    fn test_part2_example() {
        let reader = std::io::BufReader::new(EXAMPLE3.as_bytes());
        let answer = super::part2_try1(reader);
        assert_eq!(answer, 6);
    }

    #[test]
    fn test_part2_final() {
        let reader = std::io::BufReader::new(include_str!("final.txt").as_bytes());
        let answer = super::part2_try2(reader);
        assert_eq!(answer, 13129439557681);
    }
}
