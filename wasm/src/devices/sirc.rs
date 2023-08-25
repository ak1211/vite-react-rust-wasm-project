// Copyright (c) 2023 Akihiro Yamamoto.
// Licensed under the MIT License <https://spdx.org/licenses/MIT.html>
// See LICENSE file in the project root for full license information.
//
use crate::infrared_remote::*;
use nonempty::NonEmpty;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Device {
    TV,
    VCR1,
    VCR2,
    LaserDisk,
    SurroundSound,
    CassetteDeckTuner,
    CDPlayer,
    Equalizer,
}

impl fmt::Display for Device {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Device::TV => write!(f, "テレビ"),
            Device::VCR1 => write!(f, "ビデオ1"),
            Device::VCR2 => write!(f, "ビデオ2"),
            Device::LaserDisk => write!(f, "レーザーディスク"),
            Device::SurroundSound => write!(f, "サラウンド"),
            Device::CassetteDeckTuner => write!(f, "カセットデッキ"),
            Device::CDPlayer => write!(f, "CDプレーヤー"),
            Device::Equalizer => write!(f, "イコライザ"),
        }
    }
}

fn to_device(input: usize) -> Option<Device> {
    match input {
        1 => Some(Device::TV),
        2 => Some(Device::VCR1),
        3 => Some(Device::VCR2),
        6 => Some(Device::LaserDisk),
        12 => Some(Device::SurroundSound),
        16 => Some(Device::CassetteDeckTuner),
        17 => Some(Device::CDPlayer),
        18 => Some(Device::Equalizer),
        _ => None,
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Command {
    DigitKey1,
    DigitKey2,
    DigitKey3,
    DigitKey4,
    DigitKey5,
    DigitKey6,
    DigitKey7,
    DigitKey8,
    DigitKey9,
    DigitKey0,
    ChannelPlus,
    ChannelMinus,
    VolumePlus,
    VolumeMinus,
    Mute,
    Power,
    Reset,
    AudioMode,
    ContrastPlus,
    ContrastMinus,
    ColourPlus,
    ColourMinus,
    BrightnessPlus,
    BrightnessMinus,
    AUXInputSelect,
    BalanceLeft,
    BalanceRight,
    Standby,
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Command::DigitKey1 => write!(f, "1ボタン"),
            Command::DigitKey2 => write!(f, "2ボタン"),
            Command::DigitKey3 => write!(f, "3ボタン"),
            Command::DigitKey4 => write!(f, "4ボタン"),
            Command::DigitKey5 => write!(f, "5ボタン"),
            Command::DigitKey6 => write!(f, "6ボタン"),
            Command::DigitKey7 => write!(f, "7ボタン"),
            Command::DigitKey8 => write!(f, "8ボタン"),
            Command::DigitKey9 => write!(f, "9ボタン"),
            Command::DigitKey0 => write!(f, "0ボタン"),
            Command::ChannelPlus => write!(f, "チャンネル+"),
            Command::ChannelMinus => write!(f, "チャンネル-"),
            Command::VolumePlus => write!(f, "音量+"),
            Command::VolumeMinus => write!(f, "音量-"),
            Command::Mute => write!(f, "ミュート"),
            Command::Power => write!(f, "電源"),
            Command::Reset => write!(f, "リセット"),
            Command::AudioMode => write!(f, "オーディオモード"),
            Command::ContrastPlus => write!(f, "コントラスト+"),
            Command::ContrastMinus => write!(f, "コントラスト-"),
            Command::ColourPlus => write!(f, "カラー+"),
            Command::ColourMinus => write!(f, "カラー-"),
            Command::BrightnessPlus => write!(f, "輝度+"),
            Command::BrightnessMinus => write!(f, "輝度-"),
            Command::AUXInputSelect => write!(f, "入力選択"),
            Command::BalanceLeft => write!(f, "バランス左"),
            Command::BalanceRight => write!(f, "バランス右"),
            Command::Standby => write!(f, "スタンバイ"),
        }
    }
}

fn to_command(input: usize) -> Option<Command> {
    match input {
        0 => Some(Command::DigitKey1),
        1 => Some(Command::DigitKey2),
        2 => Some(Command::DigitKey3),
        3 => Some(Command::DigitKey4),
        4 => Some(Command::DigitKey5),
        5 => Some(Command::DigitKey6),
        6 => Some(Command::DigitKey7),
        7 => Some(Command::DigitKey8),
        8 => Some(Command::DigitKey9),
        9 => Some(Command::DigitKey0),
        16 => Some(Command::ChannelPlus),
        17 => Some(Command::ChannelMinus),
        18 => Some(Command::VolumePlus),
        19 => Some(Command::VolumeMinus),
        20 => Some(Command::Mute),
        21 => Some(Command::Power),
        22 => Some(Command::Reset),
        23 => Some(Command::AudioMode),
        24 => Some(Command::ContrastPlus),
        25 => Some(Command::ContrastMinus),
        26 => Some(Command::ColourPlus),
        27 => Some(Command::ColourMinus),
        30 => Some(Command::BrightnessPlus),
        31 => Some(Command::BrightnessMinus),
        37 => Some(Command::AUXInputSelect),
        38 => Some(Command::BalanceLeft),
        39 => Some(Command::BalanceRight),
        47 => Some(Command::Standby),
        _ => None,
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sirc {
    command: Command,
    device: Device,
}

/// デコード
pub fn decode_sirc(input: &Vec<InfraredRemoteDecordedFrame>) -> Option<NonEmpty<Sirc>> {
    let xs = input
        .iter()
        .flat_map(|proto| match proto {
            InfraredRemoteDecordedFrame::Sirc12(p) => decode_sub(&p.command, &p.address),
            InfraredRemoteDecordedFrame::Sirc15(p) => decode_sub(&p.command, &p.address),
            InfraredRemoteDecordedFrame::Sirc20(p) => decode_sub(&p.command, &p.address),
            _ => None,
        })
        .collect::<Vec<Sirc>>();
    NonEmpty::from_vec(xs)
}

fn decode_sub(command: &[Bit], address: &[Bit]) -> Option<Sirc> {
    let test = (
        to_command(bits_to_lsb_first(command)),
        to_device(bits_to_lsb_first(address)),
    );
    if let (Some(com), Some(dev)) = test {
        Some(Sirc {
            command: com,
            device: dev,
        })
    } else {
        None
    }
}

#[test]
fn test1_decode_sirc() {
    let hi = Bit::Hi;
    let lo = Bit::Lo;
    let protocol = vec![InfraredRemoteDecordedFrame::Sirc12(ProtocolSirc12 {
        command: [hi, lo, hi, lo, hi, lo, lo],
        address: [hi, lo, lo, lo, lo],
    })];
    let codes = decode_sirc(&protocol).unwrap();
    assert_eq!(
        codes,
        NonEmpty::new(Sirc {
            command: Command::Power,
            device: Device::TV,
        })
    );
}

#[cfg(test)]
mod decode_tests {
    use crate::devices::sirc::*;
    use crate::parsing;
    use nonempty::NonEmpty;

    #[test]
    fn test2_decode_sirc() {
        let ircode= "5B0018002E001800180018002E001800170018002E00190017001800170018002E00180018001800170018001700180017004F03";
        let markandspaces = parsing::parse_infrared_code_text(ircode).unwrap();
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
        let codes = decode_sirc(&decoded_frames);
        assert_eq!(
            codes,
            Some(NonEmpty::new(Sirc {
                command: Command::Power,
                device: Device::TV,
            }))
        );
    }
}
