// Copyright (c) 2023 Akihiro Yamamoto.
// Licensed under the MIT License <https://spdx.org/licenses/MIT.html>
// See LICENSE file in the project root for full license information.
//
use crate::infrared_remote::*;
use nonempty::NonEmpty;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum HvacMode {
    Cool,
    DryCool,
    Dehumidify,
    Heat,
    Automatic,
    AutoDehumidifying,
    QuickLaundry,
    CondensationControl,
}

fn to_hvac_mode(input: usize) -> Option<HvacMode> {
    match input {
        0x3 => Some(HvacMode::Cool),
        0x4 => Some(HvacMode::DryCool),
        0x5 => Some(HvacMode::Dehumidify),
        0x6 => Some(HvacMode::Heat),
        0x7 => Some(HvacMode::Automatic),
        0x9 => Some(HvacMode::AutoDehumidifying),
        0xa => Some(HvacMode::QuickLaundry),
        0xc => Some(HvacMode::CondensationControl),
        _ => None,
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Switch(bool);

impl fmt::Display for Switch {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.0 {
            true => write!(f, "ON"),
            false => write!(f, "OFF"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FanSpeed {
    Auto,
    Silent,
    Low,
    Med,
    High,
}

impl fmt::Display for FanSpeed {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FanSpeed::Auto => write!(f, "自動"),
            FanSpeed::Silent => write!(f, "静音"),
            FanSpeed::Low => write!(f, "風速弱"),
            FanSpeed::Med => write!(f, "風速中"),
            FanSpeed::High => write!(f, "風速強"),
        }
    }
}

fn to_fan_speed(input: usize) -> Option<FanSpeed> {
    match input {
        0x1 => Some(FanSpeed::Silent),
        0x2 => Some(FanSpeed::Low),
        0x3 => Some(FanSpeed::Med),
        0x4 => Some(FanSpeed::High),
        0x5 => Some(FanSpeed::Auto),
        _ => None,
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HitachiHvac {
    temperature: u8,
    mode: HvacMode,
    fan: FanSpeed,
    switch: Switch,
}

/// デコード
pub fn decode_hitachi_hvac(
    input: &Vec<InfraredRemoteDecordedFrame>,
) -> Option<NonEmpty<HitachiHvac>> {
    let ys = input
        .iter()
        .map(|x| match x {
            InfraredRemoteDecordedFrame::Aeha(aeha) => decode_sub(aeha),
            _ => None,
        })
        .collect::<Vec<Option<HitachiHvac>>>();
    let zs = ys
        .into_iter()
        .filter(|a| a.is_some())
        .map(|a| a.unwrap())
        .collect::<Vec<HitachiHvac>>();
    NonEmpty::from_vec(zs)
}

fn decode_sub(frame: &ProtocolAeha) -> Option<HitachiHvac> {
    if let Some(v) = frame.octets.get(0..28) {
        let _ = if v[0..9] == constraint_first_9bytes() {
            Some(1)
        } else {
            None
        }?;
        // 温度
        let temp = bits_to_lsb_first(&v[13]) as u8 >> 2 & 0x1f;
        // モード
        let mode = to_hvac_mode(bits_to_lsb_first(&v[25]) & 0xf)?;
        // 風量
        let fan = to_fan_speed(bits_to_lsb_first(&v[25]) >> 4 & 0xf)?;
        // 電源
        let switch = Switch(if bits_to_lsb_first(&v[27]) >> 4 & 1 == 0 {
            false
        } else {
            true
        });
        //
        Some(HitachiHvac {
            temperature: temp,
            mode: mode,
            fan: fan,
            switch: switch,
        })
    } else {
        None
    }
}

fn bits_to_lsb_first(v: &[Bit]) -> usize {
    let lo = false.into();
    let mut w = v.to_owned();
    w.reverse();
    w.iter()
        .fold(0usize, |acc, x| acc * 2 + if *x == lo { 0 } else { 1 })
}

//
// Hitachi HVAC first 9bytes value is
// LSB first                                        -- MSB first
// 0x01 10 00 40 bf ff 00 cc 33                     -- 0x80 08 00 02 fd ff 00 33 cc
//
// first byte "10000000"
// LSB first                                        -- MSB first
// 1   2   4   8  16  32  64 128                    -- 128  64  32  16   8   4   2   1
// |   |   |   |   |   |   |   |                    --   |   |   |   |   |   |   |   |
// 1   0   0   0   0   0   0   0 == 01h             --   1   0   0   0   0   0   0   0 == 80h
//
// second byte "00001000"
// LSB first                                        -- MSB first
// 1   2   4   8  16  32  64 128                    -- 128  64  32  16   8   4   2   1
// |   |   |   |   |   |   |   |                    --   |   |   |   |   |   |   |   |
// 0   0   0   0   1   0   0   0 == 10h             --   0   0   0   0   1   0   0   0 == 08h
//
// 3rd byte "00000000"
//
// 4th byte "00000010"
// LSB first                                        -- MSB first
// 1   2   4   8  16  32  64 128                    -- 128  64  32  16   8   4   2   1
// |   |   |   |   |   |   |   |                    --   |   |   |   |   |   |   |   |
// 0   0   0   0   0   0   1   0 == 40h             --   0   0   0   0   0   0   1   0 == 02h
//
// 5th byte "11111101"
// LSB first                                        -- MSB first
// 1   2   4   8  16  32  64 128  1+2+4+8+16+       -- 128  64  32  16   8   4   2   1  1+4+8+16+
// |   |   |   |   |   |   |   |  32+128=191        --   |   |   |   |   |   |   |   |  32+64+128=253
// 1   1   1   1   1   1   0   1 == bfh             --   1   1   1   1   1   1   0   1 == fdh
//
// 6th byte "11111111"
//
// 7th byte "00000000"
//
// 8th byte "00110011"
// LSB first                                        -- MSB first
// 1   2   4   8  16  32  64 128                    -- 128  64  32  16   8   4   2   1
// |   |   |   |   |   |   |   |  4+8+64+128=204    --   |   |   |   |   |   |   |   |  1+2+16+32=51
// 0   0   1   1   0   0   1   1 == cch             --   0   0   1   1   0   0   1   1 == 33h
//
// 9th byte "11001100"
// LSB first                                        -- MSB first
// 1   2   4   8  16  32  64 128                    -- 128  64  32  16   8   4   2   1
// |   |   |   |   |   |   |   |  1+2+16+32=51      --   |   |   |   |   |   |   |   |  4+8+64+128=204
// 1   1   0   0   1   1   0   0 == 33h             --   1   1   0   0   1   1   0   0 == cch
//
fn constraint_first_9bytes() -> [Vec<Bit>; 9] {
    [
        from_binary_string("10000000").unwrap(), // 01 (LSB first) | 80 (MSB first)
        from_binary_string("00001000").unwrap(), // 10 (LSB first) | 08 (MSB first)
        from_binary_string("00000000").unwrap(), // 00 (LSB first) | 00 (MSB first)
        from_binary_string("00000010").unwrap(), // 40 (LSB first) | 02 (MSB first)
        from_binary_string("11111101").unwrap(), // bf (LSB first) | fd (MSB first)
        from_binary_string("11111111").unwrap(), // ff (LSB first) | ff (MSB first)
        from_binary_string("00000000").unwrap(), // 00 (LSB first) | 00 (MSB first)
        from_binary_string("00110011").unwrap(), // cc (LSB first) | 33 (MSB first)
        from_binary_string("11001100").unwrap(), // 33 (LSB first) | cc (MSB first)
    ]
}

#[test]
fn test1_decode_hitachi_hvac() {
    let ircode = "6D0458078200400011003000120010001100110013000F0011001100120010001200100012001000110010001200100012001000110011001200300012001000120010001200100012001000110010001200100012001000120010001100110012001000110011001200100012001000110011001100100012001000120010001200300012001000120030001100300011003100120030001100300012003000120010001200300012003000110030001200300012002F00120030001200300012002F00120030001100110011001100120010001100110010001100120010001200100012001000120010001100110013002F00110030001200110011001000120030001200300012002F00120030001100110011001100110031001100310012000F00120010001200100012003000110011001200100011003100100011001100110012003000120030001200100012002F00120030001100110012003000120030001100110011003100110031001100100011001100110031001100110012001000110011001100110011001000120030001200300012001000120030001100300011003100110011001100110012001000110030001200300011001100120030001200100012002F001100310011003100110011001200100011003100110011001100300012001000110011001200100011001100100011001200100011001100120010001200300012002F00110031001100310012002F001200300011003100120030001100110010001100110011001200100013000F00120010001100110013000F0011003000110031001100310010003100120030001100310011003000120030001100110011001100110011001100110010001100110011001100110011001100120030001200300012002F0012003000110031001000310011003100110031001100110010001100120010001200100011001100110011001200100011001100110030001100310011003100100031001100310011003100100031001100310011001100120010001100110011001100110011001000110011001100110011001200300011003100100031001100310011003100100031001200300012003000110011001100300012003000110011001100310011001100110031000F0013001100300011001100120010001100310011001100110031001100110011003100100031001100120010001100110011001100310011003100100031001100310010001200110031001000310011003100100012001100110011001100110011001100100010001100110011001100110011001100110011001100110011001100110030001100310011003100100031001100310011003100100031001100310011001100120010001000120011001100100012001000110011001100110011001100310011003100100031001100310011003100100031001100310010003200110011000F0013001000110010001200110011001000120011001100110031001000310010003200110031000F003200110031001100300011003100110011001000320010003200100012001000120011001100110011001100110010001200100012000F0013000F0032001100310011003000100032001100310011003000110031001100110011001100110011001100110010001200100011001000120011001100100032001000320010003100100032001100310010003100110031001100110011001100100012000F00320010001300100011001000120011003100110031000F00320011003100110011001100310011003000110031001100110011001100110011001100110010001200100011001100110011001100110011001100310010003100110031001100310010003100110031001100310010003100120010001100110010001200110011001000120011001000110011001100110011003100110031001000310011003100110031001000310011003100110031001100300012003000110031001100300011003100110031001000310011003100110011001100110011001100100012001100100011001100110011001200100011003100110030001100310011003100100031001100310011003100110031001000120010001100110011001200100011001100110011001100110011001100100031001100310011003100110030001200300012003000110030001200300012001000110011001100110011001100100011001100110012001000120010001200300012002F00120030001200300012002F00110031001200300012003000110010001200100011001100110011001200100012001000110011001100110010004F03";
    let markandspaces = crate::parsing::parse_infrared_code_text(ircode).unwrap();
    let frames = decode_phase1(&markandspaces).unwrap();
    let demodulated_frames = frames
        .iter()
        .map(|frame| decode_phase2(frame))
        .collect::<Vec<InfraredRemoteDemodulatedFrame>>();
    let decoded_frames = demodulated_frames
        .iter()
        .map(|a| decode_phase3(a))
        .collect::<Result<Vec<InfraredRemoteDecordedFrame>, _>>()
        .unwrap();
    let codes = decode_hitachi_hvac(&decoded_frames);
    assert_eq!(
        codes,
        Some(NonEmpty::new(HitachiHvac {
            temperature: 22,
            mode: HvacMode::Heat,
            fan: FanSpeed::Auto,
            switch: Switch(true),
        }))
    );
}
