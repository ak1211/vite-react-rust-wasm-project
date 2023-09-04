// Copyright (c) 2023 Akihiro Yamamoto.
// Licensed under the MIT License <https://spdx.org/licenses/MIT.html>
// See LICENSE file in the project root for full license information.
pub use crate::infrared_remote::*;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 復号後の赤外線リモコン信号
pub enum DecordedInfraredRemoteFrame {
    Aeha(Vec<Bit>),
    Nec(Vec<Bit>),
    Sirc(Vec<Bit>),
    Unknown(Vec<MarkAndSpaceMicros>),
}

impl fmt::Display for DecordedInfraredRemoteFrame {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DecordedInfraredRemoteFrame::Aeha(bits) => write!(f, "AEHA {}", show_bit_pattern(bits)),
            DecordedInfraredRemoteFrame::Nec(bits) => write!(f, "NEC {}", show_bit_pattern(bits)),
            DecordedInfraredRemoteFrame::Sirc(bits) => write!(f, "SIRC {}", show_bit_pattern(bits)),
            DecordedInfraredRemoteFrame::Unknown(_) => write!(f, "Unknown protocol"),
        }
    }
}

/// 復号
pub fn decord_receiving_data(
    data_stream: &[MarkAndSpaceMicros],
) -> Result<Vec<DecordedInfraredRemoteFrame>, Box<dyn Error>> {
    // 入力マークアンドスペース列を各フレームに分ける
    let frames = data_stream.split_inclusive(|ms| THRESHOLD_FRAME_GAP <= ms.space);
    // 赤外線信号を復調して赤外線リモコン信号を取り出す
    frames
        .map(|single_frame| {
            // リーダーパルスとそれ以外に分ける
            let cons = single_frame
                .split_first()
                .ok_or(InfraredRemoteError::InputIsEmptyError)?;
            // 信号を復調する
            let result = match cons {
                (leader, trailer) if protocol_aeha::compare_leader_pulse(TOLERANCE, leader) => {
                    let mut bits = trailer
                        .iter()
                        .map(|&item| protocol_aeha::demodulate(item))
                        .collect::<Vec<Bit>>();
                    let _ = bits.pop(); // remove stop bit
                    DecordedInfraredRemoteFrame::Aeha(bits)
                }
                (leader, trailer) if protocol_nec::compare_leader_pulse(TOLERANCE, leader) => {
                    let mut bits = trailer
                        .iter()
                        .map(|&item| protocol_nec::demodulate(item))
                        .collect::<Vec<Bit>>();
                    let _ = bits.pop(); // remove stop bit
                    DecordedInfraredRemoteFrame::Nec(bits)
                }
                (leader, trailer) if protocol_sirc::compare_leader_pulse(TOLERANCE, leader) => {
                    let bits = trailer
                        .iter()
                        .map(|&item| protocol_sirc::demodulate(item))
                        .collect::<Vec<Bit>>();
                    DecordedInfraredRemoteFrame::Sirc(bits)
                }
                _ => DecordedInfraredRemoteFrame::Unknown(data_stream.to_owned()),
            };
            Ok(result)
        })
        .collect::<Result<Vec<DecordedInfraredRemoteFrame>, Box<dyn Error>>>()
}

#[cfg(test)]
mod decord_ir_data_stream_tests {
    use crate::infrared_remote::*;
    use std::error::Error;

    #[test]
    fn test1() -> Result<(), Box<dyn Error>> {
        let source= crate::parsing::parse_infrared_code_text("5B0018002E001800180018002E001800170018002E00190017001800170018002E00180018001800170018001700180017004F03")?;
        let result = decord_receiving_data(&source)?;
        let expected = vec![DecordedInfraredRemoteFrame::Sirc(
            vec![1, 0, 1, 0, 1, 0, 0, 1, 0, 0, 0, 0]
                .into_iter()
                .map(|n| Bit::try_from(n).unwrap())
                .collect::<Vec<Bit>>(),
        )];
        assert_eq!(result, expected);
        Ok(())
    }

    #[test]
    fn test2() -> Result<(), Box<dyn Error>> {
        let source= crate::parsing::parse_infrared_code_text("5601A900180015001800140018001400190013001900140019001400170040001700150018003F0019003E0018003E0019003F0019003E00170040001800140019003E001800150018003F00180014001800140019003F0018001400170016001700150018003F001800140018003F0018003F001800140019003F0018003F0018003E0019004F03")?;
        let result = decord_receiving_data(&source)?;
        let expected = vec![DecordedInfraredRemoteFrame::Nec(
            vec![
                0, 0, 0, 0, 0, 0, 1, 0, 1, 1, 1, 1, 1, 1, 0, 1, 0, 1, 0, 0, 1, 0, 0, 0, 1, 0, 1, 1,
                0, 1, 1, 1,
            ]
            .into_iter()
            .map(|n| Bit::try_from(n).unwrap())
            .collect::<Vec<Bit>>(),
        )];
        assert_eq!(result, expected);
        Ok(())
    }

    #[test]
    fn test3() -> Result<(), Box<dyn Error>> {
        let source= crate::parsing::parse_infrared_code_text("8700410014000F0014002F001400100013001000130010001300100013000F0014000F0014000F001300100014000E00140010001300100013002F001400100013001000130010001300100013000F0015000E0014000F0013001000130010001300300014000F0014000F0014000F0013001000130010001300100013000F0014000F0014002F00140010001300300014002F0015002F00150030001400100013000F0014002F00140010001300300014002F0015002F00140030001400100013002F0015004F03")?;
        let result = decord_receiving_data(&source)?;
        let expected = vec![DecordedInfraredRemoteFrame::Aeha(
            vec![
                0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0,
                0, 0, 0, 0, 1, 0, 1, 1, 1, 1, 0, 0, 1, 0, 1, 1, 1, 1, 0, 1,
            ]
            .into_iter()
            .map(|n| Bit::try_from(n).unwrap())
            .collect::<Vec<Bit>>(),
        )];
        assert_eq!(result, expected);
        Ok(())
    }
}
