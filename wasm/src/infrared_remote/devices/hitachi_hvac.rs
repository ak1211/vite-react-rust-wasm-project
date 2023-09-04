// Copyright (c) 2023 Akihiro Yamamoto.
// Licensed under the MIT License <https://spdx.org/licenses/MIT.html>
// See LICENSE file in the project root for full license information.
//
use crate::infrared_remote::*;
use once_cell::sync::Lazy;
use std::collections::HashMap;

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
const FRAME_HEADER: [LsbFirst; 9] = [
    LsbFirst::new(0x01),
    LsbFirst::new(0x10),
    LsbFirst::new(0x00),
    LsbFirst::new(0x40),
    LsbFirst::new(0xbf),
    LsbFirst::new(0xff),
    LsbFirst::new(0x00),
    LsbFirst::new(0xcc),
    LsbFirst::new(0x33),
];

//
const HVAC_MODE: Lazy<HashMap<u8, &'static str>> = Lazy::new(|| {
    let mut hm = HashMap::new();
    hm.insert(0x3, "hvac_mode_cool");
    hm.insert(0x4, "hvac_mode_dry_cool");
    hm.insert(0x5, "hvac_mode_dehumidify");
    hm.insert(0x6, "hvac_mode_heat");
    hm.insert(0x7, "hvac_mode_auto");
    hm.insert(0x9, "hvac_mode_auto_dehumidifying");
    hm.insert(0xa, "hvac_mode_quick_laundry");
    hm.insert(0xc, "hvac_mode_condensation_control");
    hm
});

//
const POWER_SWITCH: Lazy<HashMap<Bit, &'static str>> = Lazy::new(|| {
    let mut hm = HashMap::new();
    hm.insert(Bit::Lo, "power_off");
    hm.insert(Bit::Hi, "power_on");
    hm
});

//
const FAN_SPEED: Lazy<HashMap<u8, &'static str>> = Lazy::new(|| {
    let mut hm = HashMap::new();
    hm.insert(0x1, "silent");
    hm.insert(0x2, "low");
    hm.insert(0x3, "med");
    hm.insert(0x4, "high");
    hm.insert(0x5, "auto");
    hm
});

/// デコード
pub fn decode(frames: &[DecordedInfraredRemoteFrame]) -> Vec<InfraredRemoteControlCode> {
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
                    aeha.get(32..40).map(|x| folding_to_lsb_first(x))?,
                    aeha.get(40..48).map(|x| folding_to_lsb_first(x))?,
                    aeha.get(48..56).map(|x| folding_to_lsb_first(x))?,
                    aeha.get(56..64).map(|x| folding_to_lsb_first(x))?,
                    aeha.get(64..72).map(|x| folding_to_lsb_first(x))?,
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
    //
    target_frames
        .iter()
        .flat_map(|&aeha| {
            // data required 296bits
            let octets = aeha.get(0..296).map(|x| {
                pack_to_octets(x)
                    .iter()
                    .map(|x| folding_to_lsb_first(x))
                    .collect::<Vec<LsbFirst>>()
            })?;
            let mut decorded: HashMap<String, String> = HashMap::new();
            // 温度
            decorded.insert(
                "temperature".to_owned(),
                (u8::from(octets[13]) >> 2 & 0x1f).to_string(),
            );
            // モード
            HVAC_MODE
                .get(&(u8::from(octets[25]) & 0xf))
                .map(|&item| decorded.insert("hvac_mode".to_owned(), item.to_owned()));
            // 風量
            FAN_SPEED
                .get(&(u8::from(octets[25]) >> 4 & 0xf))
                .map(|&item| decorded.insert("fan_speed".to_owned(), item.to_owned()));
            // 電源
            let power_switch = Bit::try_from(u8::from(octets[27]) >> 4 & 1).ok()?;
            POWER_SWITCH
                .get(&power_switch)
                .map(|&item| decorded.insert("power_switch".to_owned(), item.to_owned()));
            // オフタイマ―
            decorded.insert("off_timer_duration_minutes".to_owned(), {
                let lower_nibble = u8::from(octets[17]) >> 4 & 0xf;
                let higher_byte = u8::from(octets[19]);
                let minutes = (higher_byte as u16) << 4 | lower_nibble as u16;
                minutes.to_string()
            });
            // オンタイマ―
            decorded.insert("on_timer_duration_minutes".to_owned(), {
                let lower_byte = u8::from(octets[21]);
                let higher_nibble = u8::from(octets[23]);
                let minutes = (higher_nibble as u16) << 8 | lower_byte as u16;
                minutes.to_string()
            });
            //
            decorded.insert("manufacturer".to_owned(), "hitachi".to_owned());
            Some(InfraredRemoteControlCode(decorded))
        })
        .collect::<Vec<InfraredRemoteControlCode>>()
}

#[cfg(test)]
mod decode_tests {
    use crate::infrared_remote::*;
    use crate::parsing;

    #[test]
    fn test1() {
        let rxdata= "6D0458078200400011003000120010001100110013000F0011001100120010001200100012001000110010001200100012001000110011001200300012001000120010001200100012001000110010001200100012001000120010001100110012001000110011001200100012001000110011001100100012001000120010001200300012001000120030001100300011003100120030001100300012003000120010001200300012003000110030001200300012002F00120030001200300012002F00120030001100110011001100120010001100110010001100120010001200100012001000120010001100110013002F00110030001200110011001000120030001200300012002F00120030001100110011001100110031001100310012000F00120010001200100012003000110011001200100011003100100011001100110012003000120030001200100012002F00120030001100110012003000120030001100110011003100110031001100100011001100110031001100110012001000110011001100110011001000120030001200300012001000120030001100300011003100110011001100110012001000110030001200300011001100120030001200100012002F001100310011003100110011001200100011003100110011001100300012001000110011001200100011001100100011001200100011001100120010001200300012002F00110031001100310012002F001200300011003100120030001100110010001100110011001200100013000F00120010001100110013000F0011003000110031001100310010003100120030001100310011003000120030001100110011001100110011001100110010001100110011001100110011001100120030001200300012002F0012003000110031001000310011003100110031001100110010001100120010001200100011001100110011001200100011001100110030001100310011003100100031001100310011003100100031001100310011001100120010001100110011001100110011001000110011001100110011001200300011003100100031001100310011003100100031001200300012003000110011001100300012003000110011001100310011001100110031000F0013001100300011001100120010001100310011001100110031001100110011003100100031001100120010001100110011001100310011003100100031001100310010001200110031001000310011003100100012001100110011001100110011001100100010001100110011001100110011001100110011001100110011001100110030001100310011003100100031001100310011003100100031001100310011001100120010001000120011001100100012001000110011001100110011001100310011003100100031001100310011003100100031001100310010003200110011000F0013001000110010001200110011001000120011001100110031001000310010003200110031000F003200110031001100300011003100110011001000320010003200100012001000120011001100110011001100110010001200100012000F0013000F0032001100310011003000100032001100310011003000110031001100110011001100110011001100110010001200100011001000120011001100100032001000320010003100100032001100310010003100110031001100110011001100100012000F00320010001300100011001000120011003100110031000F00320011003100110011001100310011003000110031001100110011001100110011001100110010001200100011001100110011001100110011001100310010003100110031001100310010003100110031001100310010003100120010001100110010001200110011001000120011001000110011001100110011003100110031001000310011003100110031001000310011003100110031001100300012003000110031001100300011003100110031001000310011003100110011001100110011001100100012001100100011001100110011001200100011003100110030001100310011003100100031001100310011003100110031001000120010001100110011001200100011001100110011001100110011001100100031001100310011003100110030001200300012003000110030001200300012001000110011001100110011001100100011001100110012001000120010001200300012002F00120030001200300012002F00110031001200300012003000110010001200100011001100110011001200100012001000110011001100110010004F03";
        let mut decorded: HashMap<String, String> = HashMap::new();
        decorded.insert("temperature".to_owned(), "22".to_owned());
        decorded.insert("hvac_mode".to_owned(), "hvac_mode_heat".to_owned());
        decorded.insert("fan_speed".to_owned(), "auto".to_owned());
        decorded.insert("power_switch".to_owned(), "power_on".to_owned());
        decorded.insert("off_timer_duration_minutes".to_owned(), "0".to_owned());
        decorded.insert("on_timer_duration_minutes".to_owned(), "0".to_owned());
        decorded.insert("manufacturer".to_owned(), "hitachi".to_owned());
        //
        let markandspaces = parsing::parse_infrared_code_text(rxdata).unwrap();
        let frames = decord_receiving_data(&markandspaces).unwrap();
        let result = hitachi_hvac::decode(&frames);
        let expected: Vec<InfraredRemoteControlCode> = vec![InfraredRemoteControlCode(decorded)];
        assert_eq!(result, expected)
    }
}
