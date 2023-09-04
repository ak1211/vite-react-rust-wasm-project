// Copyright (c) 2023 Akihiro Yamamoto.
// Licensed under the MIT License <https://spdx.org/licenses/MIT.html>
// See LICENSE file in the project root for full license information.
pub use crate::infrared_remote::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 復号後の赤外線リモコン信号
pub struct InfraredRemoteControlCode(pub HashMap<String, String>);

/// 復号
/// 復号後の赤外線リモコン信号
pub fn decord_ir_frames(frames: &[DecordedInfraredRemoteFrame]) -> Vec<InfraredRemoteControlCode> {
    vec![
        toshiba_tv::decode(frames),
        sirc::decode(frames),
        panasonic_hvac::decode(frames),
        daikin_hvac::decode(frames),
        hitachi_hvac::decode(frames),
        mitsubishi_electric_hvac::decode(frames),
    ]
    .iter()
    .find(|&v| !v.is_empty())
    .map(|a| a.to_owned())
    .unwrap_or(vec![])
}
