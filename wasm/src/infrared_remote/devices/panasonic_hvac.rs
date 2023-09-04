// Copyright (c) 2023 Akihiro Yamamoto.
// Licensed under the MIT License <https://spdx.org/licenses/MIT.html>
// See LICENSE file in the project root for full license information.
//
use crate::infrared_remote::*;
use once_cell::sync::Lazy;
use std::collections::HashMap;

//
// Panasonic HVAC first 4bytes value is
// LSB first                                    -- MSB first
// 0x02 20 e0 04                                -- 0x40 04 07 20
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
const FRAME_HEADER: [LsbFirst; 4] = [
    LsbFirst::new(0x02),
    LsbFirst::new(0x20),
    LsbFirst::new(0xe0),
    LsbFirst::new(0x04),
];

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
const FIRST_FRAME: [LsbFirst; 8] = [
    LsbFirst::new(0x02),
    LsbFirst::new(0x20),
    LsbFirst::new(0xe0),
    LsbFirst::new(0x04),
    LsbFirst::new(0x00),
    LsbFirst::new(0x00),
    LsbFirst::new(0x00),
    LsbFirst::new(0x06),
];

//
const HVAC_MODE: Lazy<HashMap<LsbFirst, &'static str>> = Lazy::new(|| {
    let mut hm = HashMap::new();
    hm.insert(LsbFirst::new(0x0), "hvac_mode_auto");
    hm.insert(LsbFirst::new(0x2), "hvac_mode_dry");
    hm.insert(LsbFirst::new(0x3), "hvac_mode_cool");
    hm.insert(LsbFirst::new(0x4), "hvac_mode_heat");
    hm.insert(LsbFirst::new(0x6), "hvac_mode_fan");
    hm
});

//
const POWER_SWITCH: Lazy<HashMap<LsbFirst, &'static str>> = Lazy::new(|| {
    let mut hm = HashMap::new();
    hm.insert(LsbFirst::new(0x8), "power_off");
    hm.insert(LsbFirst::new(0x9), "power_on");
    hm
});

//
const SWING: Lazy<HashMap<LsbFirst, &'static str>> = Lazy::new(|| {
    let mut hm = HashMap::new();
    hm.insert(LsbFirst::new(0x1), "horizontal");
    hm.insert(LsbFirst::new(0x2), "notch2");
    hm.insert(LsbFirst::new(0x3), "notch3");
    hm.insert(LsbFirst::new(0x4), "notch4");
    hm.insert(LsbFirst::new(0x5), "notch5");
    hm.insert(LsbFirst::new(0xf), "auto");
    hm
});

//
const FAN_SPEED: Lazy<HashMap<LsbFirst, &'static str>> = Lazy::new(|| {
    let mut hm = HashMap::new();
    hm.insert(LsbFirst::new(0x3), "slowest");
    hm.insert(LsbFirst::new(0x4), "notch2");
    hm.insert(LsbFirst::new(0x5), "notch3");
    hm.insert(LsbFirst::new(0x6), "notch4");
    hm.insert(LsbFirst::new(0x7), "notch5");
    hm.insert(LsbFirst::new(0xa), "auto");
    hm
});

//
const PROFILE: Lazy<HashMap<LsbFirst, &'static str>> = Lazy::new(|| {
    let mut hm = HashMap::new();
    hm.insert(LsbFirst::new(0x10), "normal");
    hm.insert(LsbFirst::new(0x11), "boost");
    hm.insert(LsbFirst::new(0x30), "quiet");
    hm
});

/// デコード
pub fn decode(frames: &[DecordedInfraredRemoteFrame]) -> Vec<InfraredRemoteControlCode> {
    decode_sub(frames).map_or(vec![], |a| vec![a])
}

/// デコード
fn decode_sub(frames: &[DecordedInfraredRemoteFrame]) -> Option<InfraredRemoteControlCode> {
    let target_frames: Vec<&[Bit]> = frames
        .iter()
        .flat_map(|fr: &DecordedInfraredRemoteFrame| match fr {
            DecordedInfraredRemoteFrame::Aeha(aeha) => {
                // ヘッダの確認
                let actual_header = [
                    aeha.get(0..8).map(|x| folding_to_lsb_first(x))?,
                    aeha.get(8..16).map(|x| folding_to_lsb_first(x))?,
                    aeha.get(16..24).map(|x| folding_to_lsb_first(x))?,
                    aeha.get(24..32).map(|x| folding_to_lsb_first(x))?,
                ];
                if actual_header == FRAME_HEADER {
                    Some(&aeha[0..])
                } else {
                    None
                }
            }
            _ => None,
        })
        .collect();
    // フレーム2個を取り出す
    if let Some([first_frame, second_frame]) = target_frames.get(0..2) {
        let mut decorded: HashMap<String, String> = HashMap::new();
        // 第1フレーム
        let actual_first_frame = [
            first_frame.get(0..8).map(|x| folding_to_lsb_first(x))?,
            first_frame.get(8..16).map(|x| folding_to_lsb_first(x))?,
            first_frame.get(16..24).map(|x| folding_to_lsb_first(x))?,
            first_frame.get(24..32).map(|x| folding_to_lsb_first(x))?,
            first_frame.get(32..40).map(|x| folding_to_lsb_first(x))?,
            first_frame.get(40..48).map(|x| folding_to_lsb_first(x))?,
            first_frame.get(48..56).map(|x| folding_to_lsb_first(x))?,
            first_frame.get(56..64).map(|x| folding_to_lsb_first(x))?,
        ];
        let _ = if actual_first_frame == FIRST_FRAME {
            Some(1)
        } else {
            None
        }?;
        // 第2フレーム
        // data required 152bits
        let octets = second_frame.get(0..152).map(|x| {
            pack_to_octets(x)
                .iter()
                .map(|x| folding_to_lsb_first(x))
                .collect::<Vec<LsbFirst>>()
        })?;
        // ===================================================================================================================
        // https://www.analysir.com/blog/2014/12/27/reverse-engineering-panasonic-ac-infrared-protocol/
        //
        // offset   | Description           | Length
        // 05       | Mode, On/Off          | 1
        // 06       | Temperature           | 1
        // 08       | Fan / Swing           | 1
        // 0d       | Profile               | 1
        // 12       | Checksum              | 1
        // ===================================================================================================================
        //
        // Mode, On/Off
        //
        HVAC_MODE
            .get(&LsbFirst::from(u8::from(octets[0x5]) >> 4 & 0xf))
            .map(|&item| decorded.insert("hvac_mode".to_owned(), item.to_owned()));
        POWER_SWITCH
            .get(&LsbFirst::from(u8::from(octets[0x5]) >> 0 & 0xf))
            .map(|&item| decorded.insert("power_switch".to_owned(), item.to_owned()));
        //
        // Temperature
        //
        decorded.insert(
            "temperature".to_owned(),
            (16 + (u8::from(octets[0x6]) >> 1 & 0xf)).to_string(),
        );
        // Always 0
        let _ = if u8::from(octets[0x6]) & 1 == 0 {
            Some(1)
        } else {
            None
        }?;
        //
        // Fan / Swing
        //
        FAN_SPEED
            .get(&LsbFirst::from(u8::from(octets[0x8]) >> 4 & 0xf))
            .map(|&item| decorded.insert("fan_speed".to_owned(), item.to_owned()));
        SWING
            .get(&LsbFirst::from(u8::from(octets[0x8]) >> 0 & 0xf))
            .map(|&item| decorded.insert("swing".to_owned(), item.to_owned()));
        //
        // Profile
        //
        PROFILE
            .get(&octets[0xd])
            .map(|&item| decorded.insert("profile".to_owned(), item.to_owned()));

        //
        // Checksum
        //
        decorded.insert("checksum".to_owned(), u8::from(octets[0x12]).to_string());
        //
        decorded.insert("manufacturer".to_owned(), "panasonic".to_owned());
        Some(InfraredRemoteControlCode(decorded))
    } else {
        None
    }
}

#[cfg(test)]
mod decode_panasonic_tests {
    use crate::infrared_remote::*;
    use crate::parsing;

    #[test]
    fn test1() {
        let rxdata = "8800410014001000130032001300100013001000130010001300100014001000130010001300100013001000130010001400100013001000130032001300100013001000130010001400100013001000130010001300100014003100130032001300320013001000130010001300320013001000130010001400100013001000130010001300100014000F00140010001300100013001000130010001300100014001000130010001300100013001000140010001300100013001000130010001300100014001000130010001300100013001000130010001400100013001000130010001400100013003100140031001300100013001000140010001300100013001000130082018800410013001000140031001300100014001000130010001300100014000F001400100013001000130010001300100014000F00140010001300320013001000130010001300100014000F001400100013001000130010001300320013003200130032001300100013001000130032001300100013001000140010001300100013001000130010001300100014001000130010001300100013001000140010001300100013003200130010001300100014003100130032001300310014001000130010001300100013001000140031001300100014003100130032001300100013000F0013001000140010001300100013001000130010001300100014001000130031001400310013003200130032001300320013001000130032001300100013003200130031001400100013003100140031001300100013001000140010001300100013001000130010001400100013001000130010001300100013001000140010001300100013003200130032001300310014000F0014001000130010001300100013001000130010001400100013001000130010001300320013003200130031001400100013001000130010001300100013001000140010001300320013001000130010001300100014000F001400100013001000130010001300100014001000130010001300320013003200130010001300100013001000130011001300310013001000140010001300100013001000130010001300100014001000130010001300100013001000130011001300310013001000140010001300100013001000130032001300320013001000130032001300100013003200130032001300100013004F03";
        let mut decorded: HashMap<String, String> = HashMap::new();
        decorded.insert("hvac_mode".to_owned(), "hvac_mode_cool".to_owned());
        decorded.insert("power_switch".to_owned(), "power_on".to_owned());
        decorded.insert("temperature".to_owned(), "26".to_owned());
        decorded.insert("fan_speed".to_owned(), "auto".to_owned());
        decorded.insert("swing".to_owned(), "auto".to_owned());
        decorded.insert("checksum".to_owned(), "107".to_owned());
        decorded.insert("manufacturer".to_owned(), "panasonic".to_owned());
        //
        let markandspaces = parsing::parse_infrared_code_text(rxdata).unwrap();
        let frames = decord_receiving_data(&markandspaces).unwrap();
        let result = panasonic_hvac::decode(&frames);
        let expected: Vec<InfraredRemoteControlCode> = vec![InfraredRemoteControlCode(decorded)];
        assert_eq!(result, expected)
    }

    #[test]
    fn test2() {
        let rxdata = "870041001400100013003200130010001300100013001000130010001400100013001000130010001300100014000F00140010001300100013003200130010001300100013001000130010001400100013001000130010001300320013003100140031001300100014001000130031001400100013001000130010001300100014000F00140010001300100013001000130010001400100013001000130010001300100014000F00140010001300100013001000130010001400100013001000130010001300100013001000140010001300100013001000130010001400100013001000130010001300320013003200130010001300100013001000130010001400100013008201880041001300100013003200130010001300100013001100130010001300100013001000130010001400100013001000130010001300100014003100130010001300100014000F001400100013001000130010001300100014003100130032001300320013001000130010001400310013001000140010001300100013001000130010001300100014001000130010001300100014000F001400100013001000130010001300320013001000130010001400310013001000140031001300100014001000130010001300100013000F0014001000130010001300100013003200130031001400100013001000130010001300100013001000140010001300100013003200130031001400310013003200130031001400100013003200130010001300320013003200130010001300320013003200130010001300100013001000130010001400100013001000130010001300100014001000130010001300100013001000130010001400310013003200130032001300100013001000140010001300100013001000130010001300100014001000130010001300320013003100140031001300100014000F00140010001300100013001000130010001400310014001000130010001300100013001000130010001400100013001000130010001300100014000F001400310013003200130010001400100013001000130010001300320013001000130010001300100014000F001400100013001000130010001300100014001000130010001300100013003100130010001300100014001000130010001300320013003100140031001300100014000F0014003100130032001300320013004F03";
        let mut decorded: HashMap<String, String> = HashMap::new();
        decorded.insert("hvac_mode".to_owned(), "hvac_mode_dry".to_owned());
        decorded.insert("power_switch".to_owned(), "power_on".to_owned());
        decorded.insert("temperature".to_owned(), "16".to_owned());
        decorded.insert("fan_speed".to_owned(), "auto".to_owned());
        decorded.insert("swing".to_owned(), "auto".to_owned());
        decorded.insert("checksum".to_owned(), "231".to_owned());
        decorded.insert("manufacturer".to_owned(), "panasonic".to_owned());
        //
        let markandspaces = parsing::parse_infrared_code_text(rxdata).unwrap();
        let frames = decord_receiving_data(&markandspaces).unwrap();
        let result = panasonic_hvac::decode(&frames);
        let expected: Vec<InfraredRemoteControlCode> = vec![InfraredRemoteControlCode(decorded)];
        assert_eq!(result, expected)
    }
}
