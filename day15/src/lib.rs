fn hash(buf: &str) -> u8 {
    let mut h = 0u8;
    for c in buf.bytes() {
        h = h.wrapping_add(c);
        h = h.wrapping_mul(17);
    }
    h
}

pub fn part1<R: std::io::BufRead>(mut reader: R) -> anyhow::Result<usize> {
    let mut input = String::new();
    reader.read_to_string(&mut input)?;
    let input = input.replace('\n', "");
    Ok(input.trim().split(',').map(|s| hash(s) as usize).sum())
}

#[derive(Debug, Clone)]
struct Lens {
    focal_length: usize,
    label: String,
}

#[derive(Debug, Clone)]
struct LensBox {
    index: usize,
    lenses: Vec<Lens>,
}

#[derive(Debug, Clone)]
enum Command<'a> {
    AddOrReplace { label: &'a str, focal_length: usize },
    Remove { label: &'a str },
}

impl<'a> Command<'a> {
    fn label(&self) -> &'a str {
        match self {
            Self::AddOrReplace { label, .. } => label,
            Self::Remove { label, .. } => label,
        }
    }

    fn from_str(s: &'a str) -> anyhow::Result<Self> {
        let pat = ['=', '-'];
        let (label, focal_length) = s
            .split_once(pat)
            .ok_or(anyhow::anyhow!("= or - not found"))?;
        let op = s
            .matches(pat)
            .next()
            .ok_or(anyhow::anyhow!("pat not found"))?
            .chars()
            .next()
            .ok_or(anyhow::anyhow!("pat not found"))?;
        match op {
            '=' => Ok(Self::AddOrReplace {
                label,
                focal_length: focal_length.parse::<usize>()?,
            }),
            '-' => Ok(Self::Remove { label }),
            _ => anyhow::bail!("invalid op: {op}"),
        }
    }
}

impl LensBox {
    fn new(index: usize) -> Self {
        Self {
            index,
            lenses: Vec::new(),
        }
    }

    fn interpret(&mut self, cmd: &Command) -> anyhow::Result<()> {
        match *cmd {
            Command::AddOrReplace {
                label,
                focal_length,
            } => {
                if let Some(lens) = self.lenses.iter_mut().find(|lens| lens.label == label) {
                    lens.focal_length = focal_length;
                } else {
                    self.lenses.push(Lens {
                        focal_length,
                        label: label.to_string(),
                    });
                }
            }
            Command::Remove { label } => {
                if let Some((index_to_remove, _)) = self
                    .lenses
                    .iter()
                    .enumerate()
                    .find(|(_, lens)| lens.label == label)
                {
                    self.lenses.remove(index_to_remove);
                }
            }
        }
        Ok(())
    }

    fn focusing_power(&self) -> usize {
        let index = self.index + 1;
        self.lenses
            .iter()
            .enumerate()
            .map(|(i, lens)| index * (i + 1) * lens.focal_length)
            .sum()
    }
}

#[derive(Debug, Clone)]
struct LensBoxSet {
    boxes: [LensBox; 256],
}

impl LensBoxSet {
    fn new() -> anyhow::Result<Self> {
        Ok(Self {
            boxes: (0..256)
                .map(LensBox::new)
                .collect::<Vec<_>>()
                .try_into()
                .map_err(|_| anyhow::anyhow!("failed to convert vec to array"))?,
        })
    }

    fn interpret(&mut self, cmd: &str) -> anyhow::Result<()> {
        let cmd = Command::from_str(cmd)?;
        self.boxes
            .get_mut(hash(cmd.label()) as usize)
            .ok_or(anyhow::anyhow!("out of bounds?"))?
            .interpret(&cmd)?;
        Ok(())
    }

    fn focusing_power(&self) -> usize {
        self.boxes.iter().map(LensBox::focusing_power).sum()
    }
}

pub fn part2<R: std::io::BufRead>(mut reader: R) -> anyhow::Result<usize> {
    let mut input = String::new();
    reader.read_to_string(&mut input)?;
    let input = input.trim().replace('\n', "");
    let lens_box_set =
        input
            .split(',')
            .try_fold(LensBoxSet::new()?, |mut acc, cmd| -> anyhow::Result<_> {
                acc.interpret(cmd)?;
                Ok(acc)
            })?;
    Ok(lens_box_set.focusing_power())
}

#[cfg(test)]
mod tests {
    #[test]
    fn hash() {
        assert_eq!(super::hash("HASH"), 52u8);
    }

    const EXAMPLE1: &str = "\
rn=1,cm-,qp=3,cm=2,qp-,pc=4,ot=9,ab=5,pc-,pc=6,ot=7
";

    #[test]
    fn part1_example() -> anyhow::Result<()> {
        let reader = std::io::BufReader::new(EXAMPLE1.as_bytes());
        let result = super::part1(reader)?;
        assert_eq!(result, 1320);
        Ok(())
    }

    #[test]
    fn part1_big() -> anyhow::Result<()> {
        let reader = std::io::BufReader::new(include_str!("big.txt").as_bytes());
        let result = super::part1(reader)?;
        assert_eq!(result, 505427);
        Ok(())
    }

    #[test]
    fn part2_example() -> anyhow::Result<()> {
        let reader = std::io::BufReader::new(EXAMPLE1.as_bytes());
        let result = super::part2(reader)?;
        assert_eq!(result, 145);
        Ok(())
    }

    #[test]
    fn part2_big() -> anyhow::Result<()> {
        let reader = std::io::BufReader::new(include_str!("big.txt").as_bytes());
        let result = super::part2(reader)?;
        assert_eq!(result, 243747);
        Ok(())
    }
}
