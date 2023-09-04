// Copyright (c) 2023 Akihiro Yamamoto.
// Licensed under the MIT License <https://spdx.org/licenses/MIT.html>
// See LICENSE file in the project root for full license information.
//
use crate::infrared_remote::*;
use once_cell::sync::Lazy;
use std::collections::HashMap;

//
const ADDRESS_HM: Lazy<HashMap<u8, &'static str>> = Lazy::new(|| {
    let mut hm = HashMap::new();
    hm.insert(1u8, "TV");
    hm.insert(2u8, "VideoCasetteRecorder1");
    hm.insert(3u8, "VideoCasetteRecorder2");
    hm.insert(6u8, "LaserDisk");
    hm.insert(12u8, "SurroundSound");
    hm.insert(16u8, "CassetteDeckTuner");
    hm.insert(17u8, "CDPlayer");
    hm.insert(18u8, "Equalizer");
    hm
});

//
const COMMAND_HM: Lazy<HashMap<u8, &'static str>> = Lazy::new(|| {
    let mut hm = HashMap::new();
    hm.insert(0u8, "DigitKey1");
    hm.insert(1u8, "DigitKey2");
    hm.insert(2u8, "DigitKey3");
    hm.insert(3u8, "DigitKey4");
    hm.insert(4u8, "DigitKey5");
    hm.insert(5u8, "DigitKey6");
    hm.insert(6u8, "DigitKey7");
    hm.insert(7u8, "DigitKey8");
    hm.insert(8u8, "DigitKey9");
    hm.insert(9u8, "DigitKey0");
    hm.insert(16u8, "ChannelPlus");
    hm.insert(17u8, "ChannelMinus");
    hm.insert(18u8, "VolumePlus");
    hm.insert(19u8, "VolumeMinus");
    hm.insert(20u8, "Mute");
    hm.insert(21u8, "Power");
    hm.insert(22u8, "Reset");
    hm.insert(23u8, "AudioMode");
    hm.insert(24u8, "ContrastPlus");
    hm.insert(25u8, "ContrastMinus");
    hm.insert(26u8, "ColourPlus");
    hm.insert(27u8, "ColourMinus");
    hm.insert(30u8, "BrightnessPlus");
    hm.insert(31u8, "BrightnessMinus");
    hm.insert(37u8, "AUXInputSelect");
    hm.insert(38u8, "BalanceLeft");
    hm.insert(39u8, "BalanceRight");
    hm.insert(47u8, "Standby");
    hm
});

/// デコード
pub fn decode(frames: &[DecordedInfraredRemoteFrame]) -> Vec<InfraredRemoteControlCode> {
    frames
        .iter()
        .flat_map(|f: &DecordedInfraredRemoteFrame| match f {
            DecordedInfraredRemoteFrame::Sirc(bits) => {
                let mut decorded: HashMap<String, String> = HashMap::new();
                //
                let command = bits
                    .get(0..7)
                    .map(|xs| u8::from(folding_to_lsb_first(xs)))?;
                COMMAND_HM
                    .get(&command)
                    .map(|&item| decorded.insert("command".to_owned(), item.to_owned()));
                //
                let address = bits.get(7..).map(|xs| u8::from(folding_to_lsb_first(xs)))?;
                ADDRESS_HM
                    .get(&address)
                    .map(|&item| decorded.insert("address".to_owned(), item.to_owned()));
                //
                decorded.insert("manufacturer".to_owned(), "sony".to_owned());
                Some(InfraredRemoteControlCode(decorded))
            }
            _ => None,
        })
        .collect::<Vec<InfraredRemoteControlCode>>()
}

#[cfg(test)]
mod decode_tests {
    use crate::infrared_remote::*;
    use crate::parsing;

    #[test]
    fn test1() {
        let rxdata= "5B0018002E001800180018002E001800170018002E00190017001800170018002E00180018001800170018001700180017004F03";
        let mut decorded: HashMap<String, String> = HashMap::new();
        decorded.insert("address".to_owned(), "TV".to_owned());
        decorded.insert("command".to_owned(), "Power".to_owned());
        decorded.insert("manufacturer".to_owned(), "sony".to_owned());
        //
        let markandspaces = parsing::parse_infrared_code_text(rxdata).unwrap();
        let frames = decord_receiving_data(&markandspaces).unwrap();
        let result = sirc::decode(&frames);
        let expected: Vec<InfraredRemoteControlCode> = vec![InfraredRemoteControlCode(decorded)];
        assert_eq!(result, expected)
    }
}
