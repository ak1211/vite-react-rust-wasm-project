// Copyright (c) 2023 Akihiro Yamamoto.
// Licensed under the MIT License <https://spdx.org/licenses/MIT.html>
// See LICENSE file in the project root for full license information.
//
use crate::infrared_remote::*;
use once_cell::sync::Lazy;
use std::collections::HashMap;

//
// Daikin HVAC frame first 4bytes value is
// LSB first                                        -- MSB first
// 0x11 da 27 00                                    -- 0x88 5b e4 00
//
// first byte "10001000"
// LSB first                                        -- MSB first
// 1   2   4   8  16  32  64 128                    -- 128  64  32  16   8   4   2   1
// |   |   |   |   |   |   |   |                    --   |   |   |   |   |   |   |   |
// 1   0   0   0   1   0   0   0 == 11h             --   1   0   0   0   1   0   0   0 == 88h
//
// second byte "01011011"
// LSB first                                        -- MSB first
// 1   2   4   8  16  32  64 128                    -- 128  64  32  16   8   4   2   1
// |   |   |   |   |   |   |   |                    --   |   |   |   |   |   |   |   |
// 0   1   0   1   1   0   1   1 == dah             --   0   1   0   1   1   0   1   1 == 5bh
//
// 3th byte "11100100"
// LSB first                                        -- MSB first
// 1   2   4   8  16  32  64 128                    -- 128  64  32  16   8   4   2   1
// |   |   |   |   |   |   |   |                    --   |   |   |   |   |   |   |   |
// 1   1   1   0   0   1   0   0 == 27h             --   1   1   1   0   0   1   0   0 == e4h
//
// 4rd byte "00000000"
//
const FRAME_HEADER: [LsbFirst; 4] = [
    LsbFirst::new(0x11),
    LsbFirst::new(0xda),
    LsbFirst::new(0x27),
    LsbFirst::new(0x00),
];

// Daikin HVAC first frame
const COMFORT_MODE: Lazy<HashMap<[LsbFirst; 8], &'static str>> = Lazy::new(|| {
    let mut hm = HashMap::new();
    hm.insert(
        [
            LsbFirst::new(0x11),
            LsbFirst::new(0xda),
            LsbFirst::new(0x27),
            LsbFirst::new(0x00),
            LsbFirst::new(0xc5),
            LsbFirst::new(0x00),
            LsbFirst::new(0x10),
            LsbFirst::new(0xe7),
        ],
        "enabled",
    );
    hm.insert(
        [
            LsbFirst::new(0x11),
            LsbFirst::new(0xda),
            LsbFirst::new(0x27),
            LsbFirst::new(0x00),
            LsbFirst::new(0xc5),
            LsbFirst::new(0x00),
            LsbFirst::new(0x00),
            LsbFirst::new(0xd7),
        ],
        "disabled",
    );
    hm
});

// Daikin HVAC second frame
const SECOND_FRAME: [LsbFirst; 8] = [
    LsbFirst::new(0x11),
    LsbFirst::new(0xda),
    LsbFirst::new(0x27),
    LsbFirst::new(0x00),
    LsbFirst::new(0x42),
    LsbFirst::new(0x00),
    LsbFirst::new(0x00),
    LsbFirst::new(0x54),
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
const TIMER_ON: Lazy<HashMap<bool, &'static str>> = Lazy::new(|| {
    let mut hm = HashMap::new();
    hm.insert(false, "disabled");
    hm.insert(true, "enabled");
    hm
});

//
const TIMER_OFF: Lazy<HashMap<bool, &'static str>> = Lazy::new(|| {
    let mut hm = HashMap::new();
    hm.insert(false, "disabled");
    hm.insert(true, "enabled");
    hm
});

//
const POWER_SWITCH: Lazy<HashMap<bool, &'static str>> = Lazy::new(|| {
    let mut hm = HashMap::new();
    hm.insert(false, "power_off");
    hm.insert(true, "power_on");
    hm
});

//
const SWING: Lazy<HashMap<LsbFirst, &'static str>> = Lazy::new(|| {
    let mut hm = HashMap::new();
    hm.insert(LsbFirst::new(0x0), "disabled");
    hm.insert(LsbFirst::new(0xf), "enabled");
    hm
});

//
const FAN_SPEED: Lazy<HashMap<LsbFirst, &'static str>> = Lazy::new(|| {
    let mut hm = HashMap::new();
    hm.insert(LsbFirst::new(0x3), "notch1");
    hm.insert(LsbFirst::new(0x4), "notch2");
    hm.insert(LsbFirst::new(0x5), "notch3");
    hm.insert(LsbFirst::new(0x6), "notch4");
    hm.insert(LsbFirst::new(0x7), "notch5");
    hm.insert(LsbFirst::new(0xa), "auto");
    hm.insert(LsbFirst::new(0xb), "silent");
    hm
});

//
const POWERFUL: Lazy<HashMap<LsbFirst, &'static str>> = Lazy::new(|| {
    let mut hm = HashMap::new();
    hm.insert(LsbFirst::new(0), "disabled");
    hm.insert(LsbFirst::new(1), "enabled");
    hm
});

//
const ECONO: Lazy<HashMap<LsbFirst, &'static str>> = Lazy::new(|| {
    let mut hm = HashMap::new();
    hm.insert(LsbFirst::new(0x80), "disabled");
    hm.insert(LsbFirst::new(0x84), "enabled");
    hm
});

/// デコード
pub fn decode(frames: &[DecordedInfraredRemoteFrame]) -> Vec<InfraredRemoteControlCode> {
    decode_sub(frames).map_or(vec![], |a| vec![a])
}

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
                if actual_header[..4] == FRAME_HEADER {
                    Some(&aeha[0..])
                } else {
                    None
                }
            }
            _ => None,
        })
        .collect();
    // フレーム３個を取り出す
    if let Some([first_frame, second_frame, third_frame]) = target_frames.get(0..3) {
        let mut decorded: HashMap<String, String> = HashMap::new();
        // 第1フレーム
        // comfort mode
        let comfort_mode = [
            first_frame.get(0..8).map(|x| folding_to_lsb_first(x))?,
            first_frame.get(8..16).map(|x| folding_to_lsb_first(x))?,
            first_frame.get(16..24).map(|x| folding_to_lsb_first(x))?,
            first_frame.get(24..32).map(|x| folding_to_lsb_first(x))?,
            first_frame.get(32..40).map(|x| folding_to_lsb_first(x))?,
            first_frame.get(40..48).map(|x| folding_to_lsb_first(x))?,
            first_frame.get(48..56).map(|x| folding_to_lsb_first(x))?,
            first_frame.get(56..64).map(|x| folding_to_lsb_first(x))?,
        ];
        COMFORT_MODE
            .get(&comfort_mode)
            .map(|&item| decorded.insert("comfort_mode".to_owned(), item.to_owned()));
        // 第2フレーム
        let actual_second_frame = [
            second_frame.get(0..8).map(|x| folding_to_lsb_first(x))?,
            second_frame.get(8..16).map(|x| folding_to_lsb_first(x))?,
            second_frame.get(16..24).map(|x| folding_to_lsb_first(x))?,
            second_frame.get(24..32).map(|x| folding_to_lsb_first(x))?,
            second_frame.get(32..40).map(|x| folding_to_lsb_first(x))?,
            second_frame.get(40..48).map(|x| folding_to_lsb_first(x))?,
            second_frame.get(48..56).map(|x| folding_to_lsb_first(x))?,
            second_frame.get(56..64).map(|x| folding_to_lsb_first(x))?,
        ];
        let _ = if actual_second_frame == SECOND_FRAME {
            Some(1)
        } else {
            None
        }?;
        // 第3フレーム
        // data required 152bits
        let octets = third_frame.get(0..152).map(|x| {
            pack_to_octets(x)
                .iter()
                .map(|x| folding_to_lsb_first(x))
                .collect::<Vec<LsbFirst>>()
        })?;
        // ===================================================================================================================
        // https://github.com/blafois/Daikin-IR-Reverse
        //
        // offset   | Description           | Length    | Example       | Decoding
        // 00-03    | Header                | 4         | 11 da 27 00   |
        // 04       | Message Identifier    | 1         | 00            |
        // 05       | Mode, On/Off, Timer   | 1         | 49            | 49 = Heat, On, No Timer
        // 06       | Temperature           | 1         | 30            | It is temperature x2. 0x30 = 48 / 2 = 24C
        // 08       | Fan / Swing           | 1         | 30            | 30 = Fan 1/5 No Swing. 3F = Fan 1/5 + Swing.
        // 0a-0c    | Timer Delay           | 3         | 3c 00 60      |
        // 0d       | Powerful              | 1         | 01            | Powerful enabled
        // 10       | Econo                 | 1         | 84            | 4 last bits
        // 12       | Checksum              | 1         | 8e            | Add all previous bytes and do a OR with mask 0xff
        // ===================================================================================================================
        //
        // Message  Idetifier
        //
        let _message_identifier = u8::from(octets[0x4]);
        //
        // Mode, On/Off, Timer
        //
        let mode_onoff_timer = u8::from(octets[0x5]);
        HVAC_MODE
            .get(&LsbFirst::from(mode_onoff_timer >> 4 & 0xf))
            .map(|&item| decorded.insert("hvac_mode".to_owned(), item.to_owned()));
        // Always 1
        let _ = if mode_onoff_timer & 8 == 0 {
            None
        } else {
            Some(1)
        }?;
        TIMER_OFF
            .get(&(mode_onoff_timer & 4 != 0))
            .map(|&item| decorded.insert("timer_off".to_owned(), item.to_owned()));
        TIMER_ON
            .get(&(mode_onoff_timer & 2 != 0))
            .map(|&item| decorded.insert("timer_on".to_owned(), item.to_owned()));
        POWER_SWITCH
            .get(&(mode_onoff_timer & 1 != 0))
            .map(|&item| decorded.insert("power_switch".to_owned(), item.to_owned()));
        //
        // Temperature
        //
        decorded.insert(
            "temperature".to_owned(),
            (u8::from(octets[0x6]) / 2).to_string(),
        );
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
        // Timer Delay
        //
        decorded.insert("timer_on_duration_hour".to_owned(), {
            let higher_nibble = u8::from(octets[0xb]) & 0xf;
            let lower_byte = u8::from(octets[0xa]);
            let minutes = (higher_nibble as u16) << 8 | lower_byte as u16;
            (minutes / 60).to_string()
        });
        decorded.insert("timer_off_duration_hour".to_owned(), {
            let higher_byte = u8::from(octets[0xc]);
            let lower_nibble = u8::from(octets[0xb]) >> 4 & 0xf;
            let minutes = (higher_byte as u16) << 4 | lower_nibble as u16;
            (minutes / 60).to_string()
        });
        //
        // Powerful
        //
        POWERFUL
            .get(&octets[0xd])
            .map(|&item| decorded.insert("powerful".to_owned(), item.to_owned()));
        //
        // Econo
        //
        ECONO
            .get(&octets[0x10])
            .map(|&item| decorded.insert("econo".to_owned(), item.to_owned()));
        //
        // Checksum
        //
        decorded.insert("checksum".to_owned(), u8::from(octets[0x12]).to_string());
        //
        decorded.insert("manufacturer".to_owned(), "daikin".to_owned());
        Some(InfraredRemoteControlCode(decorded))
    } else {
        None
    }
}

#[cfg(test)]
mod decode_tests {
    use crate::infrared_remote::*;
    use crate::parsing;

    #[test]
    fn test1() {
        let rxdata= "[417,448,418,450,417,450,417,449,418,448,417,25329,3450,1747,418,1315,419,446,419,449,417,450,417,1315,418,449,417,449,417,449,417,450,417,1314,418,450,417,1315,417,1315,418,448,418,1315,418,1315,417,1315,418,1315,417,1315,418,450,417,448,419,1312,419,449,417,449,417,451,416,449,419,448,417,449,417,450,417,448,419,448,417,449,417,1316,417,450,416,1314,419,448,418,449,417,449,418,1314,418,1314,419,449,417,450,417,448,419,447,418,450,417,448,419,448,417,449,417,449,418,448,418,449,418,449,417,448,419,448,417,449,418,449,417,1315,418,1314,418,1315,418,448,419,1313,419,448,419,1313,419,1313,420,34665,3450,1748,418,1314,419,447,418,450,416,450,417,1316,416,450,418,448,417,449,418,449,417,1315,418,449,418,1315,417,1315,417,451,416,1316,417,1314,418,1314,418,1316,416,1316,417,450,417,450,417,1313,418,451,416,449,417,449,418,449,416,450,417,449,417,450,416,449,417,450,416,451,416,449,419,1314,418,448,417,449,417,451,416,449,418,1317,416,450,415,450,417,449,418,448,417,450,416,450,417,451,416,448,417,450,417,449,417,450,417,450,417,449,418,448,417,453,414,449,417,449,417,450,416,450,416,1316,418,449,417,1315,417,449,418,1315,418,449,417,34670,3449,1750,416,1316,417,451,416,449,416,450,417,1315,418,450,416,450,415,451,417,449,417,1316,416,450,418,1315,416,1316,417,449,418,1315,418,1315,417,1316,417,1315,417,1315,418,450,416,450,417,1316,416,454,412,450,416,451,416,450,416,450,416,450,416,451,416,451,417,448,417,450,416,449,418,450,417,448,417,450,417,450,416,450,416,450,417,450,416,1317,416,1316,416,450,416,1317,417,1315,417,1316,417,449,418,448,417,452,414,451,416,1316,416,1316,417,450,416,1316,417,449,418,450,417,449,416,450,417,450,417,450,416,450,416,451,415,450,419,448,416,1316,417,1316,417,1315,418,1317,416,450,417,449,417,1315,417,450,416,450,420,448,415,450,416,450,417,450,416,450,416,450,417,449,418,1315,417,451,416,449,417,1316,416,451,416,450,416,451,415,1316,417,451,416,1316,416,450,418,450,415,450,416,451,416,451,416,449,417,450,416,450,417,450,416,450,416,450,416,1316,417,1317,417,447,418,450,416,451,416,451,416,449,416,450,417,450,417,449,416,450,416,452,414,451,416,450,416,451,415,451,416,451,415,450,416,451,416,1317,416,451,415,451,416,451,415,452,414,451,415,1317,417,1316,416,451,416,451,416,450,415,453,414,451,415,451,416,451,415,452,414,452,415,450,417,451,416,451,414,451,416,451,416,451,414,451,416,451,415,451,416,1317,416,451,415,1317,416,1316,417,1316,416,451,416]";
        let mut decorded: HashMap<String, String> = HashMap::new();
        decorded.insert("comfort_mode".to_owned(), "disabled".to_owned());
        decorded.insert("hvac_mode".to_owned(), "hvac_mode_cool".to_owned());
        decorded.insert("power_switch".to_owned(), "power_on".to_owned());
        decorded.insert("timer_on".to_owned(), "enabled".to_owned());
        decorded.insert("timer_off".to_owned(), "disabled".to_owned());
        decorded.insert("timer_on_duration_hour".to_owned(), "10".to_owned());
        decorded.insert("timer_off_duration_hour".to_owned(), "25".to_owned());
        decorded.insert("temperature".to_owned(), "22".to_owned());
        decorded.insert("fan_speed".to_owned(), "notch2".to_owned());
        decorded.insert("powerful".to_owned(), "disabled".to_owned());
        decorded.insert("swing".to_owned(), "enabled".to_owned());
        decorded.insert("checksum".to_owned(), "116".to_owned());
        decorded.insert("manufacturer".to_owned(), "daikin".to_owned());
        //
        let markandspaces = parsing::parse_infrared_code_text(rxdata).unwrap();
        let frames = decord_receiving_data(&markandspaces).unwrap();
        let result = daikin_hvac::decode(&frames);
        let expected: Vec<InfraredRemoteControlCode> = vec![InfraredRemoteControlCode(decorded)];
        assert_eq!(result, expected)
    }
}
