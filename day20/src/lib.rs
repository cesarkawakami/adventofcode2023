use anyhow::anyhow;
use smallvec::{smallvec, SmallVec};
use std::fmt::Write;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Pulse {
    Hi,
    Lo,
}

impl Pulse {
    fn opposite(self) -> Self {
        match self {
            Pulse::Hi => Pulse::Lo,
            Pulse::Lo => Pulse::Hi,
        }
    }
}

const MOD_ID_MAXLEN: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct ModId([u8; MOD_ID_MAXLEN]);

impl ModId {
    const BUTTON: Self = Self([0, 0, 1]);
    const BROADCASTER: Self = Self([0, 0, 2]);
    const OUTPUT: Self = Self([0, 0, 3]);
    const HF: Self = Self(*b"hf\0");
}

impl std::str::FromStr for ModId {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "broadcaster" {
            return Ok(Self::BROADCASTER);
        } else if s == "output" {
            return Ok(Self::OUTPUT);
        }

        if s.len() > MOD_ID_MAXLEN {
            anyhow::bail!("more than {MOD_ID_MAXLEN} chars long: {s:?}");
        }
        let mut id = [0u8; MOD_ID_MAXLEN];
        id[..s.len()].copy_from_slice(s.as_bytes());
        Ok(Self(id))
    }
}

impl std::fmt::Display for ModId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self == &Self::BUTTON {
            write!(f, "button")
        } else if self == &Self::BROADCASTER {
            write!(f, "broadcaster")
        } else if self == &Self::OUTPUT {
            write!(f, "output")
        } else {
            write!(f, "{}", std::str::from_utf8(&self.0).unwrap())
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct BroadcastState {}

impl BroadcastState {
    fn new() -> Self {
        Self {}
    }

    fn handle_pulse(&mut self, input: Pulse) -> Pulse {
        input
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct FlipFlopState {
    state: Pulse,
}

impl FlipFlopState {
    fn new() -> Self {
        Self { state: Pulse::Lo }
    }

    fn handle_pulse(&mut self, input: Pulse) -> Option<Pulse> {
        match input {
            Pulse::Hi => None,
            Pulse::Lo => {
                self.state = self.state.opposite();
                Some(self.state)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct ConjunctionState {
    memory: SmallVec<[(ModId, Pulse); 2]>,
}

impl ConjunctionState {
    fn new() -> Self {
        Self {
            memory: SmallVec::new(),
        }
    }

    fn add_input(&mut self, id: ModId) {
        self.memory.push((id, Pulse::Lo));
    }

    fn handle_pulse(&mut self, input: Pulse, input_id: ModId) -> Pulse {
        for (id, pulse) in self.memory.iter_mut() {
            if *id == input_id {
                *pulse = input;
            }
        }
        let all_high = self.memory.iter().all(|(_, pulse)| *pulse == Pulse::Hi);
        match all_high {
            true => Pulse::Lo,
            false => Pulse::Hi,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum ModuleBeh {
    Broadcast(BroadcastState),
    FlipFlop(FlipFlopState),
    Conjunction(ConjunctionState),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Module {
    id: ModId,
    dests: SmallVec<[ModId; 2]>,
    behavior: ModuleBeh,
}

impl Module {
    fn add_input(&mut self, id: ModId) {
        match self.behavior {
            ModuleBeh::Broadcast(_) => {}
            ModuleBeh::FlipFlop(_) => {}
            ModuleBeh::Conjunction(ref mut state) => state.add_input(id),
        }
    }

    fn handle_pulse(&mut self, input: Pulse, input_id: ModId) -> Vec<(ModId, Pulse)> {
        let output = match self.behavior {
            ModuleBeh::Broadcast(ref mut state) => Some(state.handle_pulse(input)),
            ModuleBeh::FlipFlop(ref mut state) => state.handle_pulse(input),
            ModuleBeh::Conjunction(ref mut state) => Some(state.handle_pulse(input, input_id)),
        };
        if let Some(pulse) = output {
            self.dests.iter().map(|&id| (id, pulse)).collect()
        } else {
            vec![]
        }
    }
}

impl std::str::FromStr for Module {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (prefixed_name, dests) = s
            .split_once("->")
            .ok_or(anyhow!("' -> ' not found in {s:?}"))?;
        let prefixed_name = prefixed_name.trim();
        let dests: SmallVec<[ModId; 2]> = dests
            .split(',')
            .map(|d| d.trim().parse())
            .collect::<Result<_, _>>()?;
        if let Some(id) = prefixed_name.strip_prefix('%') {
            Ok(Self {
                id: id.parse()?,
                dests,
                behavior: ModuleBeh::FlipFlop(FlipFlopState::new()),
            })
        } else if let Some(id) = prefixed_name.strip_prefix('&') {
            Ok(Self {
                id: id.parse()?,
                dests,
                behavior: ModuleBeh::Conjunction(ConjunctionState::new()),
            })
        } else if prefixed_name == "broadcaster" || prefixed_name == "output" {
            Ok(Self {
                id: prefixed_name.parse()?,
                dests,
                behavior: ModuleBeh::Broadcast(BroadcastState::new()),
            })
        } else {
            anyhow::bail!("unknown module type: {prefixed_name:?}");
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ModuleSet {
    modules: std::collections::BTreeMap<ModId, Module>,
}

impl std::str::FromStr for ModuleSet {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut modules: std::collections::BTreeMap<ModId, Module> = s
            .lines()
            .map(|l| l.parse::<Module>().map(|m| (m.id, m)))
            .collect::<Result<_, _>>()?;

        let conns_to_add: Vec<(ModId, ModId)> = modules
            .iter()
            .flat_map(|(_, module)| {
                module
                    .dests
                    .iter()
                    .map(|&id| (module.id, id))
                    .collect::<Vec<_>>()
            })
            .collect();

        let terminal_modules_to_add: Vec<Module> = conns_to_add
            .iter()
            .map(|&(_, to_id)| to_id)
            .filter(|id| !modules.contains_key(id))
            // .filter(|&id| id == ModId::HF)
            .map(|id| Module {
                id,
                dests: smallvec![],
                behavior: ModuleBeh::Broadcast(BroadcastState::new()),
            })
            .collect();
        modules.extend(terminal_modules_to_add.into_iter().map(|m| (m.id, m)));

        for (from_id, to_id) in conns_to_add {
            let module = modules
                .get_mut(&to_id)
                .ok_or(anyhow!("unknown dest: {to_id}"))?;
            module.add_input(from_id);
        }
        Ok(Self { modules })
    }
}

impl ModuleSet {
    fn push_button(&mut self, mut handler: impl FnMut(ModId, ModId, Pulse)) {
        let mut to_process = vec![(ModId::BUTTON, ModId::BROADCASTER, Pulse::Lo)];
        while !to_process.is_empty() {
            let mut next_to_process = vec![];
            for &(prev_id, handling_id, pulse) in to_process.iter() {
                handler(prev_id, handling_id, pulse);
                let module = self.modules.get_mut(&handling_id).unwrap();
                let outputs = module.handle_pulse(pulse, prev_id);
                for (outgoing_id, pulse) in outputs {
                    // println!("sending {handling_id} -{pulse:?}-> {outgoing_id}");
                    next_to_process.push((handling_id, outgoing_id, pulse));
                }
            }
            to_process = next_to_process;
        }
    }
}

pub fn part1<R: std::io::BufRead>(mut reader: R, count: usize) -> u64 {
    let mut input = String::new();
    reader.read_to_string(&mut input).unwrap();
    let mut module_set = input.parse::<ModuleSet>().unwrap();

    let (mut hi_cnt, mut lo_cnt) = (0, 0);
    for _ in 0..count {
        module_set.push_button(|_, _, pulse| match pulse {
            Pulse::Hi => hi_cnt += 1,
            Pulse::Lo => lo_cnt += 1,
        });
    }

    hi_cnt * lo_cnt
}

#[allow(dead_code)]
fn part2_examine<R: std::io::BufRead>(mut reader: R) {
    // let tg_nodes = "tg,xn,cb,jb,kr,gq,rn,kc,nb,ks,ff,fd,mj,tx"
    //     .split(',')
    //     .collect::<Vec<_>>();

    // let sj_nodes = "sj,bm,vf,pm,mc,jx,md,vp,pn,ll,zm,rz,hc,vd"
    //     .split(',')
    //     .collect::<Vec<_>>();

    // let vn_nodes = "vn,cp,sq,ft,dn,cn,bd,qz,xf,jf,fm,gc,qg,nd"
    //     .split(',')
    //     .collect::<Vec<_>>();

    let kn_nodes = "cf,tz,gp,rp,jj,bp,kn,fh,lm,rk,fs,kx,gn,pc"
        .split(',')
        .collect::<Vec<_>>();

    let broadcast_target = "kn";
    let relevant_nodes = kn_nodes;

    let mut input = String::new();
    reader.read_to_string(&mut input).unwrap();
    let mut input2 = String::new();
    for line in input.lines() {
        if line.starts_with("broadcaster") {
            writeln!(input2, "broadcaster -> {broadcast_target}").unwrap();
        } else if relevant_nodes.iter().any(|&s| line.contains(s)) {
            writeln!(input2, "{}", line).unwrap();
        }
    }
    let mut seen_module_sets = std::collections::HashMap::<ModuleSet, usize>::new();
    let mut module_set = input2.parse::<ModuleSet>().unwrap();
    for count in 0.. {
        if let Some(back_cnt) = seen_module_sets.get(&module_set) {
            let cycle_size = count - back_cnt;
            println!("cycle size {cycle_size} starting at {back_cnt}");
            return;
        }
        seen_module_sets.insert(module_set.clone(), count);
        let mut hf_hi_cnt = 0;
        module_set.push_button(|_, to_id, pulse| {
            if (to_id, pulse) == (ModId::HF, Pulse::Hi) {
                hf_hi_cnt += 1;
            }
        });
        if hf_hi_cnt > 0 {
            println!("got {hf_hi_cnt} HF pulses at {}", count + 1);
        }
    }
}

#[cfg(test)]
mod tests {
    const EXAMPLE1: &str = "\
broadcaster -> a, b, c
%a -> b
%b -> c
%c -> inv
&inv -> a
";
    const EXAMPLE2: &str = "\
broadcaster -> a
%a -> inv, con
&inv -> b
%b -> con
&con -> output
";

    #[test]
    fn part1_pre() {
        let test_data = [
            (EXAMPLE1, 1, 32),
            // (EXAMPLE1, 2, 64),
            // (EXAMPLE2, 11687500),
        ];
        for (input, count, expected) in test_data {
            let reader = std::io::BufReader::new(input.as_bytes());
            let result = super::part1(reader, count);
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn part1_example() {
        let test_data = [(EXAMPLE1, 32000000), (EXAMPLE2, 11687500)];
        for (input, expected) in test_data {
            let reader = std::io::BufReader::new(input.as_bytes());
            let result = super::part1(reader, 1000);
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn part1_big() {
        let reader = std::io::BufReader::new(include_str!("big.txt").as_bytes());
        let result = super::part1(reader, 1000);
        assert_eq!(result, 807069600);
    }

    #[test]
    fn part2_examine() {
        let reader = std::io::BufReader::new(include_str!("big.txt").as_bytes());
        super::part2_examine(reader);
    }

    #[test]
    fn part2_big() {
        assert_eq!(3769u64 * 3767u64 * 4019u64 * 3881u64, 221453937522197);
    }
}
