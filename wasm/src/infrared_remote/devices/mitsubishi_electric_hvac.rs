// Copyright (c) 2023 Akihiro Yamamoto.
// Licensed under the MIT License <https://spdx.org/licenses/MIT.html>
// See LICENSE file in the project root for full license information.
//
use crate::infrared_remote::*;
use once_cell::sync::Lazy;
use std::collections::HashMap;

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
const FRAME_HEADER: [LsbFirst; 5] = [
    LsbFirst::new(0x23),
    LsbFirst::new(0xcb),
    LsbFirst::new(0x26),
    LsbFirst::new(0x01),
    LsbFirst::new(0x00),
];

//
const HVAC_MODE: Lazy<HashMap<u8, &'static str>> = Lazy::new(|| {
    let mut hm = HashMap::new();
    hm.insert(0x1, "heat");
    hm.insert(0x2, "dry");
    hm.insert(0x3, "cool");
    hm.insert(0x4, "auto");
    hm
});

//
const POWER_SWITCH: Lazy<HashMap<bool, &'static str>> = Lazy::new(|| {
    let mut hm = HashMap::new();
    hm.insert(false, "off");
    hm.insert(true, "on");
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
            // data required 144bits
            let octets = aeha.get(0..144).map(|x| {
                pack_to_octets(x)
                    .iter()
                    .map(|x| folding_to_lsb_first(x))
                    .collect::<Vec<LsbFirst>>()
            })?;
            let mut decorded: HashMap<String, String> = HashMap::new();
            // 温度
            let temp = 16 + (u8::from(octets[7]) & 0xf);
            decorded.insert("temperature".to_owned(), temp.to_string());
            // モード
            let hvac_mode = u8::from(octets[6]) >> 3 & 0x7;
            HVAC_MODE
                .get(&hvac_mode)
                .map(|&item| decorded.insert("hvac_mode".to_owned(), item.to_owned()));
            // 電源
            let power_switch = Bit::try_from(u8::from(octets[5]) >> 5 & 1).ok()?;
            POWER_SWITCH
                .get(&power_switch.into())
                .map(|&item| decorded.insert("power_switch".to_owned(), item.to_owned()));
            // チェックサム
            let checksum = u8::from(octets[17]);
            decorded.insert("checksum".to_owned(), checksum.to_string());
            //
            decorded.insert("manufacturer".to_owned(), "mitsubishi electric".to_owned());
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
        let rxdata= "840044001200320012003100120011001200110010001200110033001200110012001100120031001100320013001000120032001200100013001000130031001200310013001000110032001300310012001100120011001200310011001200120011001000330012001100110012001200110012001100120010001300100013001000130010001300100012001100130010001200110011001200120011001200110012001000110012001300100013001000120011001200310013001000130010001300100012001100120011001200310013003100120010001300310012001100120010001100330012001000130031001200110012001100120011001200100013001000130031001000130012001000130010001300100012003200100033001200110010001300120011001200100013001000130010001200320012001000130010001300100013001000130010001100120012001100120011001200110012001000130010001100120013001000110012001300100012001100120011001000130012001100120011001200100013001000130010001100120013001000120011001200110012001100120011001200110012001000130031001200110012001100120011001200110012001000130031001200110012001000130031001300100013001000130010001300100012001100120011001200110012001100120011001200100013001000130010001300100013001000120011001200110012003100130010001300100012003100110012001200310013003100120011001200EB01";
        let mut decorded: HashMap<String, String> = HashMap::new();
        decorded.insert("temperature".to_owned(), "26".to_owned());
        decorded.insert("hvac_mode".to_owned(), "cool".to_owned());
        decorded.insert("power_switch".to_owned(), "on".to_owned());
        decorded.insert("checksum".to_owned(), "105".to_owned());
        decorded.insert("manufacturer".to_owned(), "mitsubishi electric".to_owned());
        //
        let markandspaces = parsing::parse_infrared_code_text(rxdata).unwrap();
        let frames = decord_receiving_data(&markandspaces).unwrap();
        let result = mitsubishi_electric_hvac::decode(&frames);
        let expected: Vec<InfraredRemoteControlCode> = vec![InfraredRemoteControlCode(decorded)];
        assert_eq!(result, expected)
    }
}
