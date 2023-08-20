// Copyright (c) 2023 Akihiro Yamamoto.
// Licensed under the MIT License <https://spdx.org/licenses/MIT.html>
// See LICENSE file in the project root for full license information.
//
use crate::infrared_remote::*;
use nonempty::NonEmpty;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComfortMode(bool);

impl fmt::Display for ComfortMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ComfortMode(true) => write!(f, "ON"),
            ComfortMode(false) => write!(f, "OFF"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum HvacMode {
    Auto,
    Dry,
    Cool,
    Heat,
    Fan,
}

impl fmt::Display for HvacMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            HvacMode::Auto => write!(f, "自動"),
            HvacMode::Dry => write!(f, "除湿"),
            HvacMode::Cool => write!(f, "冷房"),
            HvacMode::Heat => write!(f, "暖房"),
            HvacMode::Fan => write!(f, "送風"),
        }
    }
}

fn to_hvac_mode(input: usize) -> Option<HvacMode> {
    match input {
        0 => Some(HvacMode::Auto),
        2 => Some(HvacMode::Dry),
        3 => Some(HvacMode::Cool),
        4 => Some(HvacMode::Heat),
        6 => Some(HvacMode::Fan),
        _ => None,
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Switch(bool);

impl fmt::Display for Switch {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Switch(true) => write!(f, "ON"),
            Switch(false) => write!(f, "OFF"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Swing(bool);

impl fmt::Display for Swing {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Swing(true) => write!(f, "スイングON"),
            Swing(false) => write!(f, "スイングOFF"),
        }
    }
}

fn to_swing(input: usize) -> Option<Swing> {
    match input {
        0xf => Some(Swing(true)),
        0x0 => Some(Swing(false)),
        _ => None,
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FanSpeed {
    Auto,
    Silent,
    Notch1,
    Notch2,
    Notch3,
    Notch4,
    Notch5,
}

impl fmt::Display for FanSpeed {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FanSpeed::Auto => write!(f, "風量自動"),
            FanSpeed::Silent => write!(f, "静音"),
            FanSpeed::Notch1 => write!(f, "風量1"),
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
        0xb => Some(FanSpeed::Silent),
        0x3 => Some(FanSpeed::Notch1),
        0x4 => Some(FanSpeed::Notch2),
        0x5 => Some(FanSpeed::Notch3),
        0x6 => Some(FanSpeed::Notch4),
        0x7 => Some(FanSpeed::Notch5),
        _ => None,
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Checksum(u8);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DaikinHvac {
    temperature: u8,
    mode: HvacMode,
    on_timer: bool,
    on_timer_duration_hour: u8,
    off_timer: bool,
    off_timer_duration_hour: u8,
    switch: Switch,
    fan: FanSpeed,
    swing: Swing,
    checksum: Checksum,
}

/// デコード
pub fn decode_daikin_hvac(
    input: &Vec<InfraredRemoteDecordedFrame>,
) -> Option<NonEmpty<DaikinHvac>> {
    let go = |xs: &[InfraredRemoteDecordedFrame]| {
        // フレーム３個を取り出す
        match xs.get(0..3) {
            Some([first_frame, second_frame, third_frame])
                if *second_frame == constraint_second_frame() =>
            {
                let _comfort_mode = take_comfort_mode(first_frame)?;
                match third_frame {
                    InfraredRemoteDecordedFrame::Aeha(aeha) => {
                        decode_sub(&aeha).map(|a| NonEmpty::singleton(a))
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    };
    // 先頭に意味のないフレームがついている場合があるので,
    // デコード失敗後に先頭フレームを取り除いて再度試行する
    go(input).or(go(&input[1..]))
}

fn decode_sub(frame: &ProtocolAeha) -> Option<DaikinHvac> {
    let header: [Vec<Bit>; 5] = [
        from_binary_string("10001000").unwrap(), // 11 (LSB first)
        from_binary_string("01011011").unwrap(), // da (LSB first)
        from_binary_string("11100100").unwrap(), // 27 (LSB first)
        from_binary_string("00000000").unwrap(), // 00 (LSB first)
        from_binary_string("00000000").unwrap(), // 00 (LSB first)
    ];
    if let Some(v) = frame.octets.get(0..=18) {
        // ヘッダの確認
        let _ = if v[0..5] == header { Some(1) } else { None }?;
        // 温度
        let temp = bits_to_lsb_first(&v[6]) as u8 / 2;
        // モード
        let mode = to_hvac_mode(bits_to_lsb_first(&v[5]) >> 4 & 0xf)?;
        // オフタイマ
        let off_timer = if (bits_to_lsb_first(&v[5]) as u8 >> 2 & 1) == 0 {
            false
        } else {
            true
        };
        // オフ継続時間
        let off_duration = (bits_to_lsb_first(&v[12]) << 4 | bits_to_lsb_first(&v[11]) >> 4) / 60;
        // オンタイマ
        let on_timer = if (bits_to_lsb_first(&v[5]) as u8 >> 1 & 1) == 0 {
            false
        } else {
            true
        };
        // オン継続時間
        let on_duration = ((bits_to_lsb_first(&v[11]) & 0xf) << 8 | bits_to_lsb_first(&v[10])) / 60;
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
        // チェックサム
        let checksum = Checksum(bits_to_lsb_first(&v[18]) as u8);
        //
        Some(DaikinHvac {
            temperature: temp,
            mode: mode,
            on_timer: on_timer,
            on_timer_duration_hour: on_duration as u8,
            off_timer: off_timer,
            off_timer_duration_hour: off_duration as u8,
            switch: switch,
            fan: fan,
            swing: swing,
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

/// Daikin HVAC first frame value (comfort mode)
fn take_comfort_mode(frame: &InfraredRemoteDecordedFrame) -> Option<ComfortMode> {
    let comfort_mode_enabled: InfraredRemoteDecordedFrame =
        InfraredRemoteDecordedFrame::Aeha(ProtocolAeha {
            octets: vec![
                from_binary_string("10001000").unwrap(), // 11 (LSB first)
                from_binary_string("01011011").unwrap(), // da (LSB first)
                from_binary_string("11100100").unwrap(), // 27 (LSB first)
                from_binary_string("00000000").unwrap(), // 00 (LSB first)
                from_binary_string("10100011").unwrap(), // c5 (LSB first)
                from_binary_string("00000000").unwrap(), // 00 (LSB first)
                from_binary_string("10000000").unwrap(), // 10 (LSB first)
                from_binary_string("11100111").unwrap(), // e7 (LSB first)
            ],
            stop: Bit::new(1),
        });
    let comfort_mode_disabled: InfraredRemoteDecordedFrame =
        InfraredRemoteDecordedFrame::Aeha(ProtocolAeha {
            octets: vec![
                from_binary_string("10001000").unwrap(), // 11 (LSB first)
                from_binary_string("01011011").unwrap(), // da (LSB first)
                from_binary_string("11100100").unwrap(), // 27 (LSB first)
                from_binary_string("00000000").unwrap(), // 00 (LSB first)
                from_binary_string("10100011").unwrap(), // c5 (LSB first)
                from_binary_string("00000000").unwrap(), // 00 (LSB first)
                from_binary_string("00000000").unwrap(), // 00 (LSB first)
                from_binary_string("11101011").unwrap(), // d7 (LSB first)
            ],
            stop: Bit::new(1),
        });
    match *frame {
        _ if *frame == comfort_mode_enabled => Some(ComfortMode(true)),
        _ if *frame == comfort_mode_disabled => Some(ComfortMode(false)),
        _ => None,
    }
}

/// Daikin HVAC second frame value
fn constraint_second_frame() -> InfraredRemoteDecordedFrame {
    InfraredRemoteDecordedFrame::Aeha(ProtocolAeha {
        octets: vec![
            from_binary_string("10001000").unwrap(), // 11 (LSB first) | 88 (MSB first)
            from_binary_string("01011011").unwrap(), // da (LSB first) | 5b (MSB first)
            from_binary_string("11100100").unwrap(), // 27 (LSB first) | e4 (MSB first)
            from_binary_string("00000000").unwrap(), // 00 (LSB first) | 00 (MSB first)
            from_binary_string("01000010").unwrap(), // 42 (LSB first) | 42 (MSB first)
            from_binary_string("00000000").unwrap(), // 00 (LSB first) | 00 (MSB first)
            from_binary_string("00000000").unwrap(), // 00 (LSB first) | 00 (MSB first)
            from_binary_string("00101010").unwrap(), // 54 (LSB first) | 2a (MSB first)
        ],
        stop: Bit::new(1),
    })
}

#[test]
fn test1_decode_daikin_hvac() {
    let ircode= "10001100100011001000110010001100100011001000ce0384004300100032001000110010001100100011001000320010001100100011001000110010001100100032001000110010003200100032001000110010003200100032001000320010003200100032001000110010001100100032001000110010001100100011001000110010001100100011001000110010001100100011001000110010003200100011001000320010001100100011001000110010003200100032001000110010001100100011001000110010001100100011001000110010001100100011001000110010001100100011001000110010001100100011001000110010003200100032001000320010001100100032001000110010003200100032001000350584004300100032001000110010001100100011001000320010001100100011001000110010001100100032001000110010003200100032001000110010003200100032001000320010003200100032001000110010001100100032001000110010001100100011001000110010001100100011001000110010001100100011001000110010001100100032001000110010001100100011001000110010003200100011000f001100100011001000110010001100100011001000110010001100100011001000110010001100100011001000110010001100100011000f0011001000110010001100100011001000320010001100100032001000110010003200100011001000350584004300100032001000110010001100100011001000320010001100100011000f0011001000110010003200100011001000320010003200100011001000320010003200100032001000320010003200100011001000110010003200100011000f001100100011001000110010001100100011001000110010001100100011001000110010001100100011001000110010001100100011001000110010001100100011001000320010003200100011001000320010003200100032001000110010001100100011000f0011001000320010003200100011001000320010001100100011001000110010001100100011001000110010001100100011000f00110010001100100032001000320010003200100032001000110010001100100032001000110010001100100011000f0011001000110010001100100011001000110010001100100032001000110010001100100032001000110010001100100011000f003200100011001000320010001100100011000f00110010001100100011001000110010001100100011001000110010001100100011001000320010003200100011001000110010001100100011001000110010001100100011001000110010001100100011000f00110010001100100011000f001100100011000f0011001000110010003200100011000f001100100011000f0011000f0011000f003200100032001000110010001100100011000f0011000f0011000f001100100011000f0011000f0011000f00110010001100100011000f00110010001100100011000f001100100011000f00110010003200100011000f00320010003200100032001000110010004205";
    let markandspaces = crate::parsing::parse_infrared_code_text(ircode).unwrap();
    let frames = decode_phase1(&markandspaces).unwrap();
    let demodulated_frames = frames
        .iter()
        .map(|frame| decode_phase2(frame))
        .collect::<Vec<InfraredRemoteDemodulatedFrame>>();
    let decoded_frame = demodulated_frames
        .iter()
        .map(|a| decode_phase3(a))
        .collect::<Result<Vec<InfraredRemoteDecordedFrame>, _>>()
        .unwrap();
    let codes = decode_daikin_hvac(&decoded_frame);
    assert_eq!(
        codes,
        Some(NonEmpty::new(DaikinHvac {
            temperature: 22,
            mode: HvacMode::Cool,
            switch: Switch(true),
            swing: Swing(true),
            fan: FanSpeed::Notch2,
            on_timer: true,
            on_timer_duration_hour: 10,
            off_timer: false,
            off_timer_duration_hour: 25,
            checksum: Checksum(116),
        }))
    );
}
