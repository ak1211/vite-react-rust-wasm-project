//
// Copyright (c) 2023 Akihiro Yamamoto.
// Licensed under the MIT License <https://spdx.org/licenses/MIT.html>
// See LICENSE file in the project root for full license information.
use serde::de::Unexpected;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
/// 1ビットを表す型
pub enum Bit {
    Lo = 0,
    Hi = 1,
}

impl std::fmt::Display for Bit {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Bit::Lo => write!(f, "0"),
            Bit::Hi => write!(f, "1"),
        }
    }
}

impl Serialize for Bit {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u8(match self {
            Bit::Lo => 0,
            Bit::Hi => 1,
        })
    }
}

impl<'de> Deserialize<'de> for Bit {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let n = u8::deserialize(deserializer)?;
        Bit::try_from(n).ok().ok_or(serde::de::Error::invalid_value(
            Unexpected::Unsigned(n as u64),
            &"Must be 0 or 1",
        ))
    }
}

impl From<bool> for Bit {
    fn from(item: bool) -> Self {
        match item {
            false => Bit::Lo,
            true => Bit::Hi,
        }
    }
}

impl Into<bool> for Bit {
    fn into(self) -> bool {
        match self {
            Bit::Lo => false,
            Bit::Hi => true,
        }
    }
}

impl Into<u8> for Bit {
    fn into(self) -> u8 {
        match self {
            Bit::Lo => 0,
            Bit::Hi => 1,
        }
    }
}

impl TryFrom<char> for Bit {
    type Error = &'static str;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '0' => Ok(Bit::Lo),
            '1' => Ok(Bit::Hi),
            _ => Err("Must be 0 or 1"),
        }
    }
}

impl TryFrom<u8> for Bit {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Bit::Lo),
            1 => Ok(Bit::Hi),
            _ => Err("Must be 0 or 1"),
        }
    }
}

#[macro_export]
macro_rules! vec_bits {
    ( $($x:expr ),*) => {{
        {
            let mut temp_vec:Vec<Bit> = Vec::new();
            $(
                let xs = $x.chars()
                .filter(|ch| *ch != '_')
                .map(|ch| Bit::try_from(ch).unwrap())
                .collect::<Vec<Bit>>() ;
                 temp_vec.extend(&xs);
            )*
            temp_vec
        }
    }};
}
pub(crate) use vec_bits;

//
pub fn bits_from_string(str: &str) -> Option<Vec<Bit>> {
    str.chars().map(|ch| ch.try_into().ok()).collect()
}

#[derive(Clone, Copy, Debug, PartialEq, Hash, Eq)]
pub struct MsbFirst(u8);

impl MsbFirst {
    pub const fn new(value: u8) -> MsbFirst {
        MsbFirst(value)
    }
}

impl From<MsbFirst> for u8 {
    fn from(value: MsbFirst) -> u8 {
        value.0
    }
}

impl From<u8> for MsbFirst {
    fn from(value: u8) -> Self {
        Self::new(value)
    }
}

//
pub fn folding_to_msb_first(bs: &[Bit]) -> MsbFirst {
    // 右端ビットが最下位になるように畳み込む
    let value = bs.iter().fold(0, |accumulator, &bit| match bit {
        Bit::Lo => accumulator << 1 | 0,
        Bit::Hi => accumulator << 1 | 1,
    });
    MsbFirst(value)
}

#[derive(Clone, Copy, Debug, PartialEq, Hash, Eq)]
pub struct LsbFirst(u8);

impl LsbFirst {
    pub const fn new(value: u8) -> LsbFirst {
        LsbFirst(value)
    }
}

impl From<LsbFirst> for u8 {
    fn from(value: LsbFirst) -> u8 {
        value.0
    }
}

impl From<u8> for LsbFirst {
    fn from(value: u8) -> Self {
        Self::new(value)
    }
}

//
pub fn folding_to_lsb_first(bs: &[Bit]) -> LsbFirst {
    // 左端ビットが最下位になるように畳み込む
    let value = bs.iter().rfold(0, |accumulator, &bit| match bit {
        Bit::Lo => accumulator << 1 | 0,
        Bit::Hi => accumulator << 1 | 1,
    });
    LsbFirst(value)
}

/// ビット型の配列を8ビットごとに空白を入れて表示する。
pub fn show_bit_pattern(input: &[Bit]) -> String {
    let mut s = String::new();
    for (index, item) in input.iter().enumerate() {
        if index >= 8 && index & 7 == 0 {
            s.push(' ');
        }
        s = format!("{}{}", s, item);
    }
    s
}

#[cfg(test)]
mod bit_type_tests {
    use crate::infrared_remote::*;

    #[test]
    fn test1() {
        let result = vec_bits!("0");
        let expected = vec![Bit::Lo];
        assert_eq!(result, expected)
    }

    #[test]
    fn test2() {
        let result = vec_bits!("1");
        let expected = vec![Bit::Hi];
        assert_eq!(result, expected)
    }

    #[test]
    #[should_panic]
    fn test3() {
        let _ = vec_bits!("01X");
    }

    #[test]
    fn test4() {
        let result = bits_from_string("01000000");
        let expected = Some(vec_bits!("01000000"));
        assert_eq!(result, expected)
    }

    #[test]
    fn test5() {
        let result = bits_from_string("01000000");
        let expected = Some(vec_bits!("01000000"));
        assert_eq!(result, expected)
    }

    #[test]
    fn test6() {
        let result = bits_from_string("00000100");
        let expected = Some(vec_bits!("00000100"));
        assert_eq!(result, expected)
    }

    #[test]
    fn test7() {
        let result = folding_to_lsb_first(&vec_bits!("10010010"));
        let expected = LsbFirst::new(1 << 0 | 1 << 3 | 1 << 6);
        assert_eq!(result, expected)
    }

    #[test]
    fn test8() {
        let result = folding_to_msb_first(&vec_bits!("10010010"));
        let expected = MsbFirst::new(1 << 7 | 1 << 4 | 1 << 1);
        assert_eq!(result, expected)
    }

    #[test]
    fn test9() {
        let result = folding_to_lsb_first(&vec_bits!("0101_0111"));
        let expected = LsbFirst::new(1 << 1 | 1 << 3 | 1 << 5 | 1 << 6 | 1 << 7);
        assert_eq!(result, expected)
    }

    #[test]
    fn test10() {
        let result = folding_to_msb_first(&vec_bits!("0101_0111"));
        let expected = MsbFirst::new(1 << 6 | 1 << 4 | 1 << 2 | 1 << 1 | 1 << 0);
        assert_eq!(result, expected)
    }

    #[test]
    fn test11() {
        let result = folding_to_lsb_first(&vec_bits!("10000000"));
        let expected = LsbFirst::new(1);
        assert_eq!(result, expected)
    }

    #[test]
    fn test12() {
        let result = folding_to_msb_first(&vec_bits!("10000000"));
        let expected = MsbFirst::new(1 << 7);
        assert_eq!(result, expected)
    }

    #[test]
    fn test13() {
        let result = folding_to_lsb_first(&vec_bits!("00001111"));
        let expected = LsbFirst::new(1 << 4 | 1 << 5 | 1 << 6 | 1 << 7);
        assert_eq!(result, expected)
    }

    #[test]
    fn test14() {
        let result = folding_to_msb_first(&vec_bits!("00001111"));
        let expected = MsbFirst::new(1 << 3 | 1 << 2 | 1 << 1 | 1 << 0);
        assert_eq!(result, expected)
    }

    #[test]
    fn test15() {
        let result = show_bit_pattern(&vec_bits!("00000100"));
        let expected = "00000100";
        assert_eq!(result, expected)
    }

    #[test]
    fn test16() {
        let result = show_bit_pattern(&vec_bits!("0100000000000100"));
        let expected = "01000000 00000100";
        assert_eq!(result, expected)
    }

    #[test]
    fn test17() {
        let result = show_bit_pattern(&vec_bits!("0100000_000000100"));
        let expected = "01000000 00000100";
        assert_eq!(result, expected)
    }

    #[test]
    fn test18() {
        assert_eq!(serde_json::to_string(&Bit::Lo).unwrap(), "0");
    }

    #[test]
    fn test19() {
        assert_eq!(serde_json::to_string(&Bit::Hi).unwrap(), "1");
    }

    #[test]
    fn test20() {
        let result: Bit = serde_json::from_str("0").unwrap();
        assert_eq!(result, Bit::Lo);
    }

    #[test]
    fn test21() {
        let result: Bit = serde_json::from_str("1").unwrap();
        assert_eq!(result, Bit::Hi);
    }

    #[test]
    #[should_panic]
    fn test22() {
        let result: Bit = serde_json::from_str("3").unwrap();
        assert_eq!(result, Bit::Hi);
    }
}
