use std::{io::BufRead, str::FromStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum HandType {
    HighCard,
    OnePair,
    TwoPairs,
    ThreeOfAKind,
    FullHouse,
    FourOfAKind,
    FiveOfAKind,
}

impl HandType {
    fn from_cards(cards: &[Card]) -> Self {
        let mut counts = std::collections::HashMap::<Card, u64>::new();
        for card in cards {
            *counts.entry(*card).or_default() += 1;
        }
        let mut counts = counts.into_values().filter(|&x| x > 0).collect::<Vec<_>>();
        counts.sort();
        counts.reverse();
        match counts.as_slice() {
            [5] => HandType::FiveOfAKind,
            [4, 1] => HandType::FourOfAKind,
            [3, 2] => HandType::FullHouse,
            [3, 1, 1] => HandType::ThreeOfAKind,
            [2, 2, 1] => HandType::TwoPairs,
            [2, 1, 1, 1] => HandType::OnePair,
            [1, 1, 1, 1, 1] => HandType::HighCard,
            _ => panic!("invalid hand: {:?}", cards),
        }
    }

    fn from_cards2(cards: &[Card2]) -> Self {
        let mut counts = std::collections::HashMap::<Card2, u64>::new();
        let mut joker_count = 0;
        for card in cards {
            match card {
                Card2::J => joker_count += 1,
                _ => *counts.entry(*card).or_default() += 1,
            }
        }
        if joker_count == 5 {
            return HandType::FiveOfAKind;
        }
        let mut counts = counts.into_values().filter(|&x| x > 0).collect::<Vec<_>>();
        counts.sort();
        counts.reverse();
        counts[0] += joker_count;
        match counts.as_slice() {
            [5] => HandType::FiveOfAKind,
            [4, 1] => HandType::FourOfAKind,
            [3, 2] => HandType::FullHouse,
            [3, 1, 1] => HandType::ThreeOfAKind,
            [2, 2, 1] => HandType::TwoPairs,
            [2, 1, 1, 1] => HandType::OnePair,
            [1, 1, 1, 1, 1] => HandType::HighCard,
            _ => panic!("invalid hand: {:?}", cards),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Card {
    _2,
    _3,
    _4,
    _5,
    _6,
    _7,
    _8,
    _9,
    T,
    J,
    Q,
    K,
    A,
}

impl TryFrom<char> for Card {
    type Error = String;

    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            '2' => Ok(Card::_2),
            '3' => Ok(Card::_3),
            '4' => Ok(Card::_4),
            '5' => Ok(Card::_5),
            '6' => Ok(Card::_6),
            '7' => Ok(Card::_7),
            '8' => Ok(Card::_8),
            '9' => Ok(Card::_9),
            'T' => Ok(Card::T),
            'J' => Ok(Card::J),
            'Q' => Ok(Card::Q),
            'K' => Ok(Card::K),
            'A' => Ok(Card::A),
            _ => Err(format!("invalid card: {}", c)),
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Hand {
    type_: HandType,
    cards: [Card; 5],
    bid: i64,
}

impl FromStr for Hand {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut iter = s.split_ascii_whitespace();
        let cards: [Card; 5] = iter
            .next()
            .unwrap()
            .chars()
            .map(|c| Card::try_from(c).unwrap())
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        let bid = iter.next().unwrap().parse::<i64>().unwrap();
        Ok(Hand {
            type_: HandType::from_cards(&cards),
            cards,
            bid,
        })
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Hand2 {
    type_: HandType,
    cards: [Card2; 5],
    bid: i64,
}

impl From<Hand> for Hand2 {
    fn from(value: Hand) -> Self {
        let cards: [Card2; 5] = value
            .cards
            .iter()
            .map(|&x| x.into())
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        Hand2 {
            type_: HandType::from_cards2(&cards),
            cards,
            bid: value.bid,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Card2 {
    J,
    _2,
    _3,
    _4,
    _5,
    _6,
    _7,
    _8,
    _9,
    T,
    Q,
    K,
    A,
}

impl From<Card> for Card2 {
    fn from(value: Card) -> Self {
        match value {
            Card::J => Card2::J,
            Card::_2 => Card2::_2,
            Card::_3 => Card2::_3,
            Card::_4 => Card2::_4,
            Card::_5 => Card2::_5,
            Card::_6 => Card2::_6,
            Card::_7 => Card2::_7,
            Card::_8 => Card2::_8,
            Card::_9 => Card2::_9,
            Card::T => Card2::T,
            Card::Q => Card2::Q,
            Card::K => Card2::K,
            Card::A => Card2::A,
        }
    }
}

pub fn part1<R: std::io::Read>(reader: R) -> i64 {
    let reader = std::io::BufReader::new(reader);
    let mut hands = reader
        .lines()
        .map(|x| x.unwrap().parse::<Hand>().unwrap())
        .collect::<Vec<_>>();
    hands.sort();
    hands
        .into_iter()
        .enumerate()
        .map(|(i, h)| (i64::try_from(i).unwrap() + 1) * h.bid)
        .sum()
}

pub fn part2<R: std::io::Read>(reader: R) -> i64 {
    let reader = std::io::BufReader::new(reader);
    let mut hands: Vec<Hand2> = reader
        .lines()
        .map(|x| x.unwrap().parse::<Hand>().unwrap().into())
        .collect::<Vec<_>>();
    hands.sort();
    hands
        .into_iter()
        .enumerate()
        .map(|(i, h)| (i64::try_from(i).unwrap() + 1) * h.bid)
        .sum()
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_part1_example() {
        let input = include_str!("example1.txt");
        let answer = super::part1(input.as_bytes());
        assert_eq!(answer, 6440);
    }

    #[test]
    fn test_part1_final() {
        let input = include_str!("final.txt");
        let answer = super::part1(input.as_bytes());
        assert_eq!(answer, 250898830);
    }

    #[test]
    fn test_part2_example() {
        let input = include_str!("example1.txt");
        let answer = super::part2(input.as_bytes());
        assert_eq!(answer, 5905);
    }

    #[test]
    fn test_part2_final() {
        let input = include_str!("final.txt");
        let answer = super::part2(input.as_bytes());
        assert_eq!(answer, 252127335);
    }
}
