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
    Auto,
    Dry,
    Cool,
    Heat,
}

impl fmt::Display for HvacMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            HvacMode::Auto => write!(f, "自動"),
            HvacMode::Dry => write!(f, "ドライ"),
            HvacMode::Cool => write!(f, "冷房"),
            HvacMode::Heat => write!(f, "暖房"),
        }
    }
}

fn to_hvac_mode(input: usize) -> Option<HvacMode> {
    match input {
        0x4 => Some(HvacMode::Auto),
        0x2 => Some(HvacMode::Dry),
        0x3 => Some(HvacMode::Cool),
        0x1 => Some(HvacMode::Heat),
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
    Slowest,
    Notch2,
    Notch3,
    Notch4,
    Notch5,
}

impl fmt::Display for FanSpeed {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FanSpeed::Auto => write!(f, "自動"),
            FanSpeed::Slowest => write!(f, "最弱"),
            FanSpeed::Notch2 => write!(f, "風速2"),
            FanSpeed::Notch3 => write!(f, "風速3"),
            FanSpeed::Notch4 => write!(f, "風速4"),
            FanSpeed::Notch5 => write!(f, "風速5"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Checksum(u8);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MitsubishiElectricHvac {
    temperature: u8,
    mode1: HvacMode,
    switch: Switch,
    checksum: Checksum,
}

/// デコード
pub fn decode_mitsubishi_electric_hvac(
    input: &Vec<InfraredRemoteDecordedFrame>,
) -> Option<NonEmpty<MitsubishiElectricHvac>> {
    let ys = input
        .iter()
        .map(|x| match x {
            InfraredRemoteDecordedFrame::Aeha(aeha) => decode_sub(aeha),
            _ => None,
        })
        .collect::<Option<Vec<MitsubishiElectricHvac>>>();
    ys.map(|v| NonEmpty::from_vec(v)).flatten()
}

fn decode_sub(frame: &ProtocolAeha) -> Option<MitsubishiElectricHvac> {
    if let Some(v) = frame.octets.get(0..18) {
        // ヘッダの確認
        let _ = if v[0..5] == constraint_first_5bytes() {
            Some(1)
        } else {
            None
        }?;
        // 温度
        let temp = bits_to_lsb_first(&v[7]) as u8 & 0xf;
        // モード1
        let mode1 = to_hvac_mode(bits_to_lsb_first(&v[6]) >> 3 & 0x7)?;
        // 電源
        let switch = Switch(if bits_to_lsb_first(&v[5]) >> 5 & 1 == 0 {
            false
        } else {
            true
        });
        // チェックサム
        let checksum = Checksum(bits_to_lsb_first(&v[17]) as u8);
        //
        Some(MitsubishiElectricHvac {
            temperature: 16 + temp,
            mode1: mode1,
            switch: switch,
            checksum: checksum,
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
// Mitsubishi Electric HVAC first 5bytes value is
// LSB first                                        -- MSB first
// 0x23 cb 26 01 00                                 -- 0xc4 d3 64 80 00
//
// first byte "11000100"
// LSB first                                        -- MSB first
// 1   2   4   8  16  32  64 128                    -- 128  64  32  16   8   4   2   1
// |   |   |   |   |   |   |   | 1+2+32=35          --   |   |   |   |   |   |   |   | 4+64+128=196
// 1   1   0   0   0   1   0   0 == 23h             --   1   1   0   0   0   1   0   0 == c4h
//
// second byte "11010011"
// LSB first                                        -- MSB first
// 1   2   4   8  16  32  64 128                    -- 128  64  32  16   8   4   2   1
// |   |   |   |   |   |   |   | 1+2+8+64+128=203   --   |   |   |   |   |   |   |   | 1+2+16+64+128=211
// 1   1   0   1   0   0   1   1 == cbh             --   1   1   0   1   0   0   1   1 == d3h
//
// 3rd byte "01100100"
// LSB first                                        -- MSB first
// 1   2   4   8  16  32  64 128                    -- 128  64  32  16   8   4   2   1
// |   |   |   |   |   |   |   | 2+4+32=38          --   |   |   |   |   |   |   |   | 4+32+64=100
// 0   1   1   0   0   1   0   0 == 26h             --   0   1   1   0   0   1   0   0 == 64h
//
// 4th byte "10000000"
// LSB first                                        -- MSB first
// 1   2   4   8  16  32  64 128                    -- 128  64  32  16   8   4   2   1
// |   |   |   |   |   |   |   |                    --   |   |   |   |   |   |   |   |
// 1   0   0   0   0   0   0   0 == 01h             --   1   0   0   0   0   0   0   0 == 80h
//
// 5th byte "00000000"
//
fn constraint_first_5bytes() -> [Vec<Bit>; 5] {
    [
        from_binary_string("11000100").unwrap(), // 23 (LSB first) | c4 (MSB first)
        from_binary_string("11010011").unwrap(), // cb (LSB first) | d3 (MSB first)
        from_binary_string("01100100").unwrap(), // 26 (LSB first) | 64 (MSB first)
        from_binary_string("10000000").unwrap(), // 01 (LSB first) | 80 (MSB first)
        from_binary_string("00000000").unwrap(), // 00 (LSB first) | 00 (MSB first)
    ]
}

#[test]
fn test1_decode_mitsubishi_electric_hvac() {
    let ircode = "840044001200320012003100120011001200110010001200110033001200110012001100120031001100320013001000120032001200100013001000130031001200310013001000110032001300310012001100120011001200310011001200120011001000330012001100110012001200110012001100120010001300100013001000130010001300100012001100130010001200110011001200120011001200110012001000110012001300100013001000120011001200310013001000130010001300100012001100120011001200310013003100120010001300310012001100120010001100330012001000130031001200110012001100120011001200100013001000130031001000130012001000130010001300100012003200100033001200110010001300120011001200100013001000130010001200320012001000130010001300100013001000130010001100120012001100120011001200110012001000130010001100120013001000110012001300100012001100120011001000130012001100120011001200100013001000130010001100120013001000120011001200110012001100120011001200110012001000130031001200110012001100120011001200110012001000130031001200110012001000130031001300100013001000130010001300100012001100120011001200110012001100120011001200100013001000130010001300100013001000120011001200110012003100130010001300100012003100110012001200310013003100120011001200EB01";
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
    let codes = decode_mitsubishi_electric_hvac(&decoded_frames);
    assert_eq!(
        codes,
        Some(NonEmpty::new(MitsubishiElectricHvac {
            temperature: 26,
            mode1: HvacMode::Cool,
            switch: Switch(true),
            checksum: Checksum(105),
        }))
    );
}
