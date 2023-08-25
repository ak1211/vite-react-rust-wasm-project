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
    Fan,
    Dry,
    Cool,
    Heat,
}

impl fmt::Display for HvacMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            HvacMode::Auto => write!(f, "自動"),
            HvacMode::Fan => write!(f, "送風"),
            HvacMode::Dry => write!(f, "除湿"),
            HvacMode::Cool => write!(f, "冷房"),
            HvacMode::Heat => write!(f, "暖房"),
        }
    }
}

fn to_hvac_mode(input: usize) -> Option<HvacMode> {
    match input {
        0x0 => Some(HvacMode::Auto),
        0x2 => Some(HvacMode::Dry),
        0x3 => Some(HvacMode::Cool),
        0x4 => Some(HvacMode::Heat),
        0x6 => Some(HvacMode::Fan),
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
pub enum Swing {
    Auto,
    Horizontal,
    Notch2,
    Notch3,
    Notch4,
    Notch5,
}

impl fmt::Display for Swing {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Swing::Auto => write!(f, "自動"),
            Swing::Horizontal => write!(f, "水平"),
            Swing::Notch2 => write!(f, "角度2"),
            Swing::Notch3 => write!(f, "角度3"),
            Swing::Notch4 => write!(f, "角度4"),
            Swing::Notch5 => write!(f, "角度5"),
        }
    }
}

fn to_swing(input: usize) -> Option<Swing> {
    match input {
        0xf => Some(Swing::Auto),
        0x1 => Some(Swing::Horizontal),
        0x2 => Some(Swing::Notch2),
        0x3 => Some(Swing::Notch3),
        0x4 => Some(Swing::Notch4),
        0x5 => Some(Swing::Notch5),
        _ => None,
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
            FanSpeed::Auto => write!(f, "風量自動"),
            FanSpeed::Slowest => write!(f, "風量1"),
            FanSpeed::Notch2 => write!(f, "風量2"),
            FanSpeed::Notch3 => write!(f, "風量3"),
            FanSpeed::Notch4 => write!(f, "風量4"),
            FanSpeed::Notch5 => write!(f, "風量5"),
        }
    }
}

fn to_fan_speed(input: usize) -> Option<FanSpeed> {
    match input {
        0xa => Some(FanSpeed::Auto),
        0x3 => Some(FanSpeed::Slowest),
        0x4 => Some(FanSpeed::Notch2),
        0x5 => Some(FanSpeed::Notch3),
        0x6 => Some(FanSpeed::Notch4),
        0x7 => Some(FanSpeed::Notch5),
        _ => None,
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Profile {
    Normal,
    Boost,
    Quiet,
    Other(usize),
}

impl fmt::Display for Profile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Profile::Normal => write!(f, "通常"),
            Profile::Boost => write!(f, "パワフル"),
            Profile::Quiet => write!(f, "静穏"),
            Profile::Other(a) => write!(f, "その他({})", a),
        }
    }
}

fn to_profile(input: usize) -> Profile {
    match input {
        0x10 => Profile::Normal,
        0x11 => Profile::Boost,
        0x30 => Profile::Quiet,
        n => Profile::Other(n),
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Checksum(u8);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PanasonicHvac {
    temperature: u8,
    mode: HvacMode,
    switch: Switch,
    swing: Swing,
    fan: FanSpeed,
    profile: Profile,
    checksum: Checksum,
}

/// デコード
pub fn decode_panasonic_hvac(
    input: &Vec<InfraredRemoteDecordedFrame>,
) -> Option<NonEmpty<PanasonicHvac>> {
    // フレーム２つを取り出す
    match input.get(0..2) {
        Some([first_frame, second_frame]) if *first_frame == constraint_first_frame() => {
            match second_frame {
                InfraredRemoteDecordedFrame::Aeha(aeha) => {
                    decode_sub(&aeha).map(|a| NonEmpty::singleton(a))
                }
                _ => None,
            }
        }
        _ => None,
    }
}

fn decode_sub(frame: &ProtocolAeha) -> Option<PanasonicHvac> {
    if let Some(v) = frame.octets.get(0..19) {
        // 温度
        let temp = bits_to_lsb_first(&v[6]) as u8 >> 1 & 0xf;
        // モード
        let mode = to_hvac_mode(bits_to_lsb_first(&v[5]) >> 4 & 0xf)?;
        // 電源
        let switch = Switch(if bits_to_lsb_first(&v[5]) & 0x1 == 0 {
            false
        } else {
            true
        });
        // ファン
        let fan = to_fan_speed(bits_to_lsb_first(&v[8]) >> 4 & 0xf)?;
        // 風向
        let swing = to_swing(bits_to_lsb_first(&v[8]) & 0xf)?;
        // プロファイル
        let prof = to_profile(bits_to_lsb_first(&v[13]) & 0xff);
        // チェックサム
        let checksum = Checksum(bits_to_lsb_first(&v[18]) as u8);
        //
        Some(PanasonicHvac {
            temperature: 16 + temp,
            mode: mode,
            switch: switch,
            swing: swing,
            fan: fan,
            profile: prof,
            checksum: checksum,
        })
    } else {
        None
    }
}

//
// Panasonic HVAC first frame value is
// LSB first                                    -- MSB first
// 0x02 20 e0 04 00 00 00 06                    -- 0x40 04 07 20 00 00 00 60
//
// first byte "01000000"
// LSB first                                    -- MSB first
// 1   2   4   8  16  32  64 128                -- 128  64  32  16   8   4   2   1
// |   |   |   |   |   |   |   |                --   |   |   |   |   |   |   |   |
// 0   1   0   0   0   0   0   0 == 02h         --   0   1   0   0   0   0   0   0 == 40h
//
// second byte "00000100"
// LSB first                                    -- MSB first
// 1   2   4   8  16  32  64 128                -- 128  64  32  16   8   4   2   1
// |   |   |   |   |   |   |   |                --   |   |   |   |   |   |   |   |
// 0   0   0   0   0   1   0   0 == 20h         --   0   0   0   0   0   1   0   0 == 04h
//
// 3rd byte "00000111"
// LSB first                                    -- MSB first
// 1   2   4   8  16  32  64 128                -- 128  64  32  16   8   4   2   1
// |   |   |   |   |   |   |   | 32+64+128=224  --   |   |   |   |   |   |   |   | 1+2+4=7
// 0   0   0   0   0   1   1   1 == e0h         --   0   0   0   0   0   1   1   1 == 07h
//
// 4th byte "00100000"
// LSB first                                    -- MSB first
// 1   2   4   8  16  32  64 128                -- 128  64  32  16   8   4   2   1
// |   |   |   |   |   |   |   |                --   |   |   |   |   |   |   |   |
// 0   0   1   0   0   0   0   0 == 04h         --   0   0   1   0   0   0   0   0 == 20h
//
// 5th byte "00000000"
// 6th byte "00000000"
// 7th byte "00000000"
//
// 8th byte "01100000"
// LSB first                                    -- MSB first
// 1   2   4   8  16  32  64 128                -- 128  64  32  16   8   4   2   1
// |   |   |   |   |   |   |   | 2+4=6          --   |   |   |   |   |   |   |   | 32+64=96
// 0   1   1   0   0   0   0   0 == 06h         --   0   1   1   0   0   0   0   0 == 60h
//
fn constraint_first_frame() -> InfraredRemoteDecordedFrame {
    InfraredRemoteDecordedFrame::Aeha(ProtocolAeha {
        octets: vec![
            from_binary_string("01000000").unwrap(), // 02 (LSB first) | 40 (MSB first)
            from_binary_string("00000100").unwrap(), // 20 (LSB first) | 04 (MSB first)
            from_binary_string("00000111").unwrap(), // e0 (LSB first) | 07 (MSB first)
            from_binary_string("00100000").unwrap(), // 04 (LSB first) | 20 (MSB first)
            from_binary_string("00000000").unwrap(), // 00 (LSB first) | 00 (MSB first)
            from_binary_string("00000000").unwrap(), // 00 (LSB first) | 00 (MSB first)
            from_binary_string("00000000").unwrap(), // 00 (LSB first) | 00 (MSB first)
            from_binary_string("01100000").unwrap(), // 06 (LSB first) | 60 (MSB first)
        ],
        stop: Bit::new(1).unwrap(),
    })
}

#[test]
fn test1_decode_panasonic_hvac() {
    let ircode= "8800410014001000130032001300100013001000130010001300100014001000130010001300100013001000130010001400100013001000130032001300100013001000130010001400100013001000130010001300100014003100130032001300320013001000130010001300320013001000130010001400100013001000130010001300100014000F0014001000130010001300100013001000130010001400100013001000130010001300100014001000130010001300100013001000130010001400100013001000130010001300100013001000140010001300100013001000140010001300310014003100130010001300100014001000130010001300100013008201";
    let markandspaces0 = crate::parsing::parse_infrared_code_text(ircode).unwrap();
    let markandspaces = markandspaces0
        .into_iter()
        .map(|x| MarkAndSpaceMicros::from(x))
        .collect::<Vec<MarkAndSpaceMicros>>();
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
    assert_eq!(decoded_frames, [constraint_first_frame()]);
}

#[test]
fn test2_decode_panasonic_hvac() {
    let ircode= "8800410014001000130032001300100013001000130010001300100014001000130010001300100013001000130010001400100013001000130032001300100013001000130010001400100013001000130010001300100014003100130032001300320013001000130010001300320013001000130010001400100013001000130010001300100014000F00140010001300100013001000130010001300100014001000130010001300100013001000140010001300100013001000130010001300100014001000130010001300100013001000130010001400100013001000130010001400100013003100140031001300100013001000140010001300100013001000130082018800410013001000140031001300100014001000130010001300100014000F001400100013001000130010001300100014000F00140010001300320013001000130010001300100014000F001400100013001000130010001300320013003200130032001300100013001000130032001300100013001000140010001300100013001000130010001300100014001000130010001300100013001000140010001300100013003200130010001300100014003100130032001300310014001000130010001300100013001000140031001300100014003100130032001300100013000F0013001000140010001300100013001000130010001300100014001000130031001400310013003200130032001300320013001000130032001300100013003200130031001400100013003100140031001300100013001000140010001300100013001000130010001400100013001000130010001300100013001000140010001300100013003200130032001300310014000F0014001000130010001300100013001000130010001400100013001000130010001300320013003200130031001400100013001000130010001300100013001000140010001300320013001000130010001300100014000F001400100013001000130010001300100014001000130010001300320013003200130010001300100013001000130011001300310013001000140010001300100013001000130010001300100014001000130010001300100013001000130011001300310013001000140010001300100013001000130032001300320013001000130032001300100013003200130032001300100013004F03";
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
    let codes = decode_panasonic_hvac(&decoded_frames);
    assert_eq!(
        codes,
        Some(NonEmpty::new(PanasonicHvac {
            temperature: 26,
            mode: HvacMode::Cool,
            switch: Switch(true),
            swing: Swing::Auto,
            fan: FanSpeed::Auto,
            profile: Profile::Other(64),
            checksum: Checksum(107),
        }))
    );
}
