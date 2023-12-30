use anyhow::{anyhow, bail};

use std::cmp::Ordering;

#[derive(Debug, Clone)]
struct Part {
    x: i16,
    m: i16,
    a: i16,
    s: i16,
}

#[derive(Debug, Clone, Copy)]
struct PartRange {
    x: (i16, i16),
    m: (i16, i16),
    a: (i16, i16),
    s: (i16, i16),
}

impl PartRange {
    fn new_empty() -> Self {
        Self {
            x: (0, -1),
            m: (0, -1),
            a: (0, -1),
            s: (0, -1),
        }
    }

    fn new_full() -> Self {
        Self {
            x: (1, 4000),
            m: (1, 4000),
            a: (1, 4000),
            s: (1, 4000),
        }
    }

    fn card(self) -> u64 {
        [self.x, self.m, self.a, self.s]
            .iter()
            .map(|(min, max)| u64::try_from((max - min + 1).max(0)).unwrap())
            .product()
    }
}

impl std::str::FromStr for Part {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (_, rest) = s.split_once('{').ok_or(anyhow!("no '{{' in {s:?}"))?;
        let (vals, _) = rest.split_once('}').ok_or(anyhow!("no '}}' in {s:?}"))?;
        let mut part = Self {
            x: i16::MIN,
            m: i16::MIN,
            a: i16::MIN,
            s: i16::MIN,
        };
        for val in vals.split(',') {
            let (attr, val) = val.split_once('=').ok_or(anyhow!("no '-' in {val:?}"))?;
            let attr: Attr = attr.parse()?;
            let val: i16 = val.parse()?;
            attr.set(&mut part, val);
        }
        Ok(part)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Attr {
    X,
    M,
    A,
    S,
}

impl Attr {
    fn get(self, part: &Part) -> i16 {
        match self {
            Self::X => part.x,
            Self::M => part.m,
            Self::A => part.a,
            Self::S => part.s,
        }
    }
    fn set(self, part: &mut Part, val: i16) {
        match self {
            Self::X => part.x = val,
            Self::M => part.m = val,
            Self::A => part.a = val,
            Self::S => part.s = val,
        }
    }
}

impl std::str::FromStr for Attr {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "x" => Ok(Self::X),
            "m" => Ok(Self::M),
            "a" => Ok(Self::A),
            "s" => Ok(Self::S),
            _ => bail!("invalid attr: {s}"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct WfId([u8; 3]);

impl std::str::FromStr for WfId {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() > 3 {
            bail!("too long: {s}");
        }
        let mut bytes = [0; 3];
        bytes[..s.len()].copy_from_slice(s.as_bytes());
        Ok(Self(bytes))
    }
}

impl std::fmt::Display for WfId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s: String = self
            .0
            .iter()
            .cloned()
            .take_while(|&c| c != 0)
            .map(char::from)
            .collect();
        write!(f, "{s}")
    }
}

#[derive(Debug, Clone)]
struct Cond {
    attr: Attr,
    cmp: Ordering,
    rhs: i16,
}

impl Cond {
    fn eval(&self, part: &Part) -> bool {
        let lhs = self.attr.get(part);
        lhs.cmp(&self.rhs) == self.cmp
    }

    fn eval_range(&self, part_range: PartRange) -> (PartRange, PartRange) {
        let (in_min, in_max) = match self.attr {
            Attr::X => part_range.x,
            Attr::M => part_range.m,
            Attr::A => part_range.a,
            Attr::S => part_range.s,
        };
        let (true_min, true_max, false_min, false_max) = match self.cmp {
            Ordering::Less => {
                if in_max < self.rhs {
                    (in_min, in_max, 0, -1)
                } else if in_min < self.rhs {
                    (in_min, self.rhs - 1, self.rhs, in_max)
                } else {
                    (0, -1, in_min, in_max)
                }
            }
            Ordering::Greater => {
                if in_min > self.rhs {
                    (in_min, in_max, 0, -1)
                } else if in_max > self.rhs {
                    (self.rhs + 1, in_max, in_min, self.rhs)
                } else {
                    (0, -1, in_min, in_max)
                }
            }
            Ordering::Equal => unimplemented!(),
        };
        let (mut true_range, mut false_range) = (part_range, part_range);
        let (to_change_true, to_change_false) = match self.attr {
            Attr::X => (&mut true_range.x, &mut false_range.x),
            Attr::M => (&mut true_range.m, &mut false_range.m),
            Attr::A => (&mut true_range.a, &mut false_range.a),
            Attr::S => (&mut true_range.s, &mut false_range.s),
        };
        *to_change_true = (true_min, true_max);
        *to_change_false = (false_min, false_max);
        (true_range, false_range)
    }
}

impl std::str::FromStr for Cond {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (attr, rhs) = s
            .split_once(['<', '>', '='])
            .ok_or(anyhow!("no op found in {s}"))?;
        let cmp = match s.chars().nth(attr.len()).ok_or(anyhow!("unreachable"))? {
            '<' => Ordering::Less,
            '>' => Ordering::Greater,
            '=' => Ordering::Equal,
            _ => bail!("invalid op: {s}"),
        };
        Ok(Self {
            attr: attr.parse()?,
            cmp,
            rhs: rhs.parse()?,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Act {
    Accept,
    Reject,
    Send(WfId),
}

impl std::str::FromStr for Act {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "A" => Self::Accept,
            "R" => Self::Reject,
            s => Self::Send(s.parse()?),
        })
    }
}

#[derive(Debug, Clone)]
enum Rule {
    Act(Act),
    Cond(Cond, Act),
}

impl Rule {
    fn eval(&self, part: &Part) -> Option<Act> {
        match self {
            Self::Act(act) => Some(*act),
            Self::Cond(cond, act) => {
                if cond.eval(part) {
                    Some(*act)
                } else {
                    None
                }
            }
        }
    }

    fn eval_range(&self, part_range: PartRange) -> (PartRange, Act, PartRange) {
        match self {
            Self::Act(act) => (part_range, *act, PartRange::new_empty()),
            Self::Cond(cond, act) => {
                let (true_range, false_range) = cond.eval_range(part_range);
                (true_range, *act, false_range)
            }
        }
    }
}

impl std::str::FromStr for Rule {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split_once(':') {
            None => Ok(Self::Act(s.parse()?)),
            Some((cond, act)) => Ok(Self::Cond(cond.parse()?, act.parse()?)),
        }
    }
}

#[derive(Debug, Clone)]
struct Workflow {
    id: WfId,
    rules: Vec<Rule>,
}

impl Workflow {
    fn eval(&self, part: &Part) -> Option<Act> {
        self.rules.iter().find_map(|rule| rule.eval(part))
    }

    fn eval_range(&self, part_range: PartRange) -> Vec<(PartRange, Act)> {
        let mut result = vec![];
        let mut remaining = part_range;
        for rule in self.rules.iter() {
            let (true_range, act, false_range) = rule.eval_range(remaining);
            remaining = false_range;
            result.push((true_range, act));
        }
        assert!(remaining.card() == 0);
        result
    }
}

impl std::str::FromStr for Workflow {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (id, rest) = s.split_once('{').ok_or(anyhow!("no {{ in {s}"))?;
        let id = id.parse()?;
        let (rules, _) = rest.split_once('}').ok_or(anyhow!("no }} in {s}"))?;
        let rules = rules
            .split(',')
            .map(|r| r.parse())
            .collect::<Result<_, _>>()?;
        Ok(Self { id, rules })
    }
}

#[derive(Debug, Clone)]
struct WorkflowSet {
    workflows: std::collections::HashMap<WfId, Workflow>,
}

impl WorkflowSet {
    fn process(&self, part: &Part) -> Act {
        let mut act = Act::Send("in".parse().unwrap());
        while let Act::Send(id) = act {
            let wf = self.workflows.get(&id).unwrap();
            act = wf.eval(part).unwrap();
        }
        act
    }

    fn accepted_ranges(&self) -> Vec<PartRange> {
        let mut current = vec![(PartRange::new_full(), Act::Send("in".parse().unwrap()))];
        let mut next = vec![];
        let mut accepted = vec![];
        while !current.is_empty() {
            for (part_range, act) in current {
                match act {
                    Act::Accept => accepted.push(part_range),
                    Act::Reject => {}
                    Act::Send(id) => {
                        let wf = self.workflows.get(&id).unwrap();
                        let mut ranges = wf.eval_range(part_range);
                        next.append(&mut ranges);
                    }
                }
            }
            current = next;
            next = vec![];
        }
        accepted
    }
}

impl std::str::FromStr for WorkflowSet {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut workflows = std::collections::HashMap::new();
        for line in s.lines() {
            let workflow: Workflow = line.trim().parse()?;
            workflows.insert(workflow.id, workflow);
        }
        Ok(Self { workflows })
    }
}

#[derive(Debug, Clone)]
struct PartSet {
    parts: Vec<Part>,
}

impl std::str::FromStr for PartSet {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = Vec::new();
        for line in s.lines() {
            let part: Part = line.trim().parse()?;
            parts.push(part);
        }
        Ok(Self { parts })
    }
}

pub fn part1<R: std::io::BufRead>(mut reader: R) -> i64 {
    let mut input = String::new();
    reader.read_to_string(&mut input).unwrap();
    let (workflow_set, part_set) = input.split_once("\n\n").unwrap();
    let workflow_set: WorkflowSet = workflow_set.parse().unwrap();
    let part_set: PartSet = part_set.parse().unwrap();
    part_set
        .parts
        .iter()
        .filter(|p| workflow_set.process(p) == Act::Accept)
        .map(|&Part { x, m, a, s }| [x, m, a, s].iter().map(|&v| i64::from(v)).sum::<i64>())
        .sum()
}

pub fn part2<R: std::io::BufRead>(mut reader: R) -> u64 {
    let mut input = String::new();
    reader.read_to_string(&mut input).unwrap();
    let (workflow_set, _) = input.split_once("\n\n").unwrap();
    let workflow_set: WorkflowSet = workflow_set.parse().unwrap();
    workflow_set
        .accepted_ranges()
        .iter()
        .map(|p| p.card())
        .sum()
}

#[cfg(test)]
mod tests {
    const EXAMPLE1: &str = "\
px{a<2006:qkq,m>2090:A,rfg}
pv{a>1716:R,A}
lnx{m>1548:A,A}
rfg{s<537:gd,x>2440:R,A}
qs{s>3448:A,lnx}
qkq{x<1416:A,crn}
crn{x>2662:A,R}
in{s<1351:px,qqz}
qqz{s>2770:qs,m<1801:hdj,R}
gd{a>3333:R,R}
hdj{m>838:A,pv}

{x=787,m=2655,a=1222,s=2876}
{x=1679,m=44,a=2067,s=496}
{x=2036,m=264,a=79,s=2244}
{x=2461,m=1339,a=466,s=291}
{x=2127,m=1623,a=2188,s=1013}
";

    #[test]
    fn part1_example() {
        let reader = std::io::BufReader::new(EXAMPLE1.as_bytes());
        let result = super::part1(reader);
        assert_eq!(result, 19114);
    }

    #[test]
    fn part1_big() {
        let reader = std::io::BufReader::new(include_str!("big.txt").as_bytes());
        let result = super::part1(reader);
        assert_eq!(result, 432788);
    }

    #[test]
    fn part2_example() {
        let reader = std::io::BufReader::new(EXAMPLE1.as_bytes());
        let result = super::part2(reader);
        assert_eq!(result, 167409079868000u64);
    }

    #[test]
    fn part2_big() {
        let reader = std::io::BufReader::new(include_str!("big.txt").as_bytes());
        let result = super::part2(reader);
        assert_eq!(result, 142863718918201u64);
    }
}
