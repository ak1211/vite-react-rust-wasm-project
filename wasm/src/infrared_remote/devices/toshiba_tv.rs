// Copyright (c) 2023 Akihiro Yamamoto.
// Licensed under the MIT License <https://spdx.org/licenses/MIT.html>
// See LICENSE file in the project root for full license information.
//
use crate::infrared_remote::*;
use once_cell::sync::Lazy;
use std::collections::HashMap;

//
const ADDRESS: Lazy<HashMap<[LsbFirst; 2], &'static str>> = Lazy::new(|| {
    let mut hm = HashMap::new();
    hm.insert([LsbFirst::new(0x40), LsbFirst::new(0xbf)], "tv");
    hm
});

//
const COMMAND: Lazy<HashMap<[LsbFirst; 2], &'static str>> = Lazy::new(|| {
    let mut hm = HashMap::new();
    hm.insert([LsbFirst::new(0x0f), LsbFirst::new(0xf0)], "InputSelect");
    hm.insert([LsbFirst::new(0x10), LsbFirst::new(0xef)], "Mute");
    hm.insert([LsbFirst::new(0x12), LsbFirst::new(0xed)], "Power");
    hm.insert([LsbFirst::new(0x13), LsbFirst::new(0xec)], "SoundSelect");
    hm.insert([LsbFirst::new(0x1a), LsbFirst::new(0xe5)], "Volume+");
    hm.insert([LsbFirst::new(0x1b), LsbFirst::new(0xe4)], "ChannelUp");
    hm.insert([LsbFirst::new(0x1e), LsbFirst::new(0xe1)], "Volume-");
    hm.insert([LsbFirst::new(0x1f), LsbFirst::new(0xe0)], "ChannelDown");
    hm.insert([LsbFirst::new(0x61), LsbFirst::new(0x9e)], "DigitKey1");
    hm.insert([LsbFirst::new(0x62), LsbFirst::new(0x9d)], "DigitKey2");
    hm.insert([LsbFirst::new(0x63), LsbFirst::new(0x9c)], "DigitKey3");
    hm.insert([LsbFirst::new(0x64), LsbFirst::new(0x9b)], "DigitKey4");
    hm.insert([LsbFirst::new(0x65), LsbFirst::new(0x9a)], "DigitKey5");
    hm.insert([LsbFirst::new(0x66), LsbFirst::new(0x99)], "DigitKey6");
    hm.insert([LsbFirst::new(0x67), LsbFirst::new(0x98)], "DigitKey7");
    hm.insert([LsbFirst::new(0x68), LsbFirst::new(0x97)], "DigitKey8");
    hm.insert([LsbFirst::new(0x69), LsbFirst::new(0x96)], "DigitKey9");
    hm.insert([LsbFirst::new(0x6a), LsbFirst::new(0x95)], "DigitKey10");
    hm.insert([LsbFirst::new(0x6b), LsbFirst::new(0x94)], "DigitKey11");
    hm.insert([LsbFirst::new(0x6c), LsbFirst::new(0x93)], "DigitKey12");
    hm.insert([LsbFirst::new(0x6d), LsbFirst::new(0x92)], "Media");
    hm.insert([LsbFirst::new(0x73), LsbFirst::new(0x8c)], "Blue");
    hm.insert([LsbFirst::new(0x74), LsbFirst::new(0x8b)], "Red");
    hm.insert([LsbFirst::new(0x75), LsbFirst::new(0x8a)], "Green");
    hm.insert([LsbFirst::new(0x76), LsbFirst::new(0x89)], "Yellow");
    hm
});

/// デコード
pub fn decode(frames: &[DecordedInfraredRemoteFrame]) -> Vec<InfraredRemoteControlCode> {
    frames
        .iter()
        .flat_map(|f: &DecordedInfraredRemoteFrame| match f {
            DecordedInfraredRemoteFrame::Nec(bits) => {
                let mut decorded: HashMap<String, String> = HashMap::new();
                // data required 32bits
                let address = [
                    bits.get(0..8).map(|x| folding_to_lsb_first(x))?,
                    bits.get(8..16).map(|x| folding_to_lsb_first(x))?,
                ];
                let command = [
                    bits.get(16..24).map(|x| folding_to_lsb_first(x))?,
                    bits.get(24..32).map(|x| folding_to_lsb_first(x))?,
                ];
                //
                ADDRESS
                    .get(&address)
                    .map(|&item| decorded.insert("address".to_owned(), item.to_owned()));
                COMMAND
                    .get(&command)
                    .map(|&item| decorded.insert("command".to_owned(), item.to_owned()));
                //
                decorded.insert("manufacturer".to_owned(), "toshiba".to_owned());
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
        let rxdata= "5601A900180015001800140018001400190013001900140019001400170040001700150018003F0019003E0018003E0019003F0019003E00170040001800140019003E001800150018003F00180014001800140019003F0018001400170016001700150018003F001800140018003F0018003F001800140019003F0018003F0018003E0019004F03";
        let mut decorded: HashMap<String, String> = HashMap::new();
        decorded.insert("address".to_owned(), "tv".to_owned());
        decorded.insert("command".to_owned(), "Power".to_owned());
        decorded.insert("manufacturer".to_owned(), "toshiba".to_owned());
        //
        let markandspaces = parsing::parse_infrared_code_text(rxdata).unwrap();
        let frames = decord_receiving_data(&markandspaces).unwrap();
        let result = toshiba_tv::decode(&frames);
        let expected: Vec<InfraredRemoteControlCode> = vec![InfraredRemoteControlCode(decorded)];
        assert_eq!(result, expected)
    }

    #[test]
    fn test2() {
        let rxdata= "5501AA0018001500180014001800140019001300190014001800150017004000180014001700400018003F0018003E0019003F0018003F0019003E001700150018003F001800150018001400170016001700150018003F001800140019001400180014001700400019003E0019003E0017004000180014001800400019003E0018003E0019004F03";
        let mut decorded: HashMap<String, String> = HashMap::new();
        decorded.insert("address".to_owned(), "tv".to_owned());
        decorded.insert("command".to_owned(), "Mute".to_owned());
        decorded.insert("manufacturer".to_owned(), "toshiba".to_owned());
        //
        let markandspaces = parsing::parse_infrared_code_text(rxdata).unwrap();
        let frames = decord_receiving_data(&markandspaces).unwrap();
        let result = toshiba_tv::decode(&frames);
        let expected: Vec<InfraredRemoteControlCode> = vec![InfraredRemoteControlCode(decorded)];
        assert_eq!(result, expected)
    }
}
