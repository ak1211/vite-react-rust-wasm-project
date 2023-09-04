// Copyright (c) 2023 Akihiro Yamamoto.
// Licensed under the MIT License <https://spdx.org/licenses/MIT.html>
// See LICENSE file in the project root for full license information.
pub mod bit;
pub mod decord_ir_frames;
pub mod decord_receiving_data;
pub mod devices;
pub mod protocol_aeha;
pub mod protocol_nec;
pub mod protocol_sirc;
pub use crate::infrared_remote::bit::*;
pub use crate::infrared_remote::decord_ir_frames::*;
pub use crate::infrared_remote::decord_receiving_data::*;
pub use crate::infrared_remote::devices::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert;
use std::convert::TryInto;
use std::fmt;
use std::iter;
use std::ops;
use thiserror::Error;

/// オクテット単位にまとめる
pub fn pack_to_octets(bits: &[Bit]) -> Vec<[Bit; 8]> {
    let mut octets: Vec<[Bit; 8]> = vec![];
    for idx in (0..bits.len()).step_by(8) {
        let slice = bits.get(idx..idx + 8).unwrap_or(&bits[idx..]);
        octets.push(slice.try_into().expect("slice with incorrect length"));
    }
    octets
}

#[derive(Error, Debug, PartialEq)]
pub enum InfraredRemoteError {
    #[error("input is empty.")]
    InputIsEmptyError,
    #[error("unknown protocol.")]
    UnknownProtocolError,
    #[error("insufficient input data. (expected {0})")]
    InsufficientInputData(&'static str),
}

/// ずれ時間の許容範囲はとりあえず 300us
pub const TOLERANCE: Microseconds = Microseconds(300);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
/// マイクロ秒型
pub struct Microseconds(pub u32);

impl ops::Add for Microseconds {
    type Output = Microseconds;
    /// マイクロ秒型の加算演算子
    fn add(self, other: Self) -> Self::Output {
        Microseconds(self.0 + other.0)
    }
}

impl ops::Sub for Microseconds {
    type Output = Microseconds;
    /// マイクロ秒型の減算演算子
    fn sub(self, other: Self) -> Self::Output {
        Microseconds(self.0 - other.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
/// 赤外線リモコン信号のキャリア周波数カウンタ型
pub struct IrCarrierCounter(pub u16);

impl IrCarrierCounter {
    /// 16ビットリトルエンディアンで出力する
    pub fn to_string_littel_endian_u16(self) -> String {
        let upper = (self.0 >> 8) & 0xff;
        let lower = self.0 & 0xff;
        format!("{lower:02X}{upper:02X}")
    }
}

#[test]
fn test_to_string_littel_endian_u16() {
    assert_eq!(
        IrCarrierCounter(0x1234).to_string_littel_endian_u16(),
        "3412"
    );
    assert_eq!(
        IrCarrierCounter(0xabcd).to_string_littel_endian_u16(),
        "CDAB"
    );
    assert_eq!(
        IrCarrierCounter(0xf0a0).to_string_littel_endian_u16(),
        "A0F0"
    );
    assert_eq!(
        IrCarrierCounter(0xa0f0).to_string_littel_endian_u16(),
        "F0A0"
    );
    assert_eq!(
        IrCarrierCounter(0xff00).to_string_littel_endian_u16(),
        "00FF"
    );
    assert_eq!(
        IrCarrierCounter(0x00ff).to_string_littel_endian_u16(),
        "FF00"
    );
}

impl ops::Add for IrCarrierCounter {
    type Output = IrCarrierCounter;
    /// 赤外線リモコン信号のキャリア周波数カウンタ型の加算演算子
    fn add(self, other: Self) -> Self::Output {
        IrCarrierCounter(self.0 + other.0)
    }
}

impl ops::Sub for IrCarrierCounter {
    type Output = IrCarrierCounter;
    /// 赤外線リモコン信号のキャリア周波数カウンタ型の減算演算子
    fn sub(self, other: Self) -> Self::Output {
        IrCarrierCounter(self.0 - other.0)
    }
}

impl fmt::Display for IrCarrierCounter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// 赤外線リモコン信号のキャリア周波数
/// 38000 Hz = 38 kHz
pub const IR_CARRIER_FREQ: u16 = 38000;

impl convert::From<IrCarrierCounter> for Microseconds {
    /// 赤外線リモコン信号のキャリア周波数カウンタ型からマイクロ秒型へ変換する
    fn from(x: IrCarrierCounter) -> Self {
        // 1 カウント が 1/IR_CARRIER_FREQ 秒 なので
        // 1000倍してミリ秒に
        // さらに1000倍してマイクロ秒にする
        let y = 1_000_000u64 * x.0 as u64 / IR_CARRIER_FREQ as u64;
        Self(y as u32)
    }
}

impl convert::From<Microseconds> for IrCarrierCounter {
    /// マイクロ秒型から赤外線リモコン信号のキャリア周波数カウンタ型へ変換する
    fn from(x: Microseconds) -> Self {
        // 1 秒が IR_CARRIER_FREQ カウントなので
        // 1マイクロ秒 が IrCarrirFreq/(1000*1000) カウント
        let y = x.0 as u64 * IR_CARRIER_FREQ as u64 / 1_000_000u64;
        Self(y as u16)
    }
}

#[test]
fn test_microseconds_to_ircarriercounter() {
    assert_eq!(
        MarkAndSpaceIrCarrier::from(MarkAndSpaceMicros {
            mark: Microseconds(9000),
            space: Microseconds(4500),
        }),
        MarkAndSpaceIrCarrier {
            mark: IrCarrierCounter(0x0156),
            space: IrCarrierCounter(0x00AB),
        },
    );
    assert_eq!(
        IrCarrierCounter::from(Microseconds(4500)),
        IrCarrierCounter(0x00AB)
    );
    assert_eq!(
        IrCarrierCounter::from(Microseconds(9000)),
        IrCarrierCounter(0x0156)
    );
    assert_eq!(
        IrCarrierCounter::from(Microseconds(4500)),
        IrCarrierCounter(0x00AB)
    );
    assert_eq!(
        IrCarrierCounter::from(Microseconds(3400)),
        IrCarrierCounter(0x0081)
    );
    assert_eq!(
        IrCarrierCounter::from(Microseconds(1700)),
        IrCarrierCounter(0x0040)
    );
    assert_eq!(
        IrCarrierCounter::from(Microseconds(2400)),
        IrCarrierCounter(0x005B)
    );
    assert_eq!(
        IrCarrierCounter::from(Microseconds(600)),
        IrCarrierCounter(0x0016)
    );
}

#[test]
fn test_ircarriercounter_to_microseconds() {
    assert_eq!(
        MarkAndSpaceMicros::from(MarkAndSpaceIrCarrier {
            mark: IrCarrierCounter(0x0156),
            space: IrCarrierCounter(0x00AB),
        }),
        MarkAndSpaceMicros {
            mark: Microseconds(9000),
            space: Microseconds(4500),
        },
    );
    assert_eq!(
        Microseconds::from(IrCarrierCounter(0x0156)),
        Microseconds(9000)
    );
    assert_eq!(
        Microseconds::from(IrCarrierCounter(0x00AB)),
        Microseconds(4500)
    );
    assert_eq!(
        Microseconds::from(IrCarrierCounter(0x0081)),
        Microseconds(3394)
    );
    assert_eq!(
        Microseconds::from(IrCarrierCounter(0x0040)),
        Microseconds(1684)
    );
    assert_eq!(
        Microseconds::from(IrCarrierCounter(0x005B)),
        Microseconds(2394)
    );
    assert_eq!(
        Microseconds::from(IrCarrierCounter(0x0016)),
        Microseconds(578)
    );
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
/// マークアンドスペース型
pub struct MarkAndSpace<T> {
    pub mark: T,
    pub space: T,
}

impl<T> convert::From<MarkAndSpace<T>> for (T, T) {
    /// マークアンドスペース型からタプル型へ変換する
    fn from(x: MarkAndSpace<T>) -> Self {
        (x.mark, x.space)
    }
}

impl<T> convert::From<(T, T)> for MarkAndSpace<T> {
    /// タプル型からマークアンドスペース型へ変換する
    fn from((a, b): (T, T)) -> Self {
        MarkAndSpace { mark: a, space: b }
    }
}

impl<T: ops::Add<Output = T>> ops::Add for MarkAndSpace<T> {
    type Output = MarkAndSpace<T>;
    /// マークアンドスペースの加算演算子
    fn add(self, other: Self) -> Self::Output {
        Self::Output {
            mark: self.mark + other.mark,
            space: self.space + other.space,
        }
    }
}

impl<T: ops::Sub<Output = T>> ops::Sub for MarkAndSpace<T> {
    type Output = MarkAndSpace<T>;
    /// マークアンドスペースの減算演算子
    fn sub(self, other: Self) -> Self::Output {
        Self::Output {
            mark: self.mark - other.mark,
            space: self.space - other.space,
        }
    }
}

/// マークアンドスペース(マイクロ秒ベース)
pub type MarkAndSpaceMicros = MarkAndSpace<Microseconds>;

/// マークアンドスペース(キャリア周波数カウンタ型ベース)
pub type MarkAndSpaceIrCarrier = MarkAndSpace<IrCarrierCounter>;

impl MarkAndSpaceIrCarrier {
    /// 16ビットリトルエンディアンで出力する
    pub fn to_string_littel_endian_u16(&self) -> String {
        format!(
            "{}{}",
            IrCarrierCounter::from(self.mark).to_string_littel_endian_u16(),
            IrCarrierCounter::from(self.space).to_string_littel_endian_u16()
        )
    }
}

impl convert::From<MarkAndSpaceIrCarrier> for MarkAndSpaceMicros {
    /// マークアンドスペース(キャリア周波数カウンタ型ベース)から
    /// マークアンドスペース(マイクロ秒ベース)へ
    /// 変換する
    fn from(carrir: MarkAndSpaceIrCarrier) -> Self {
        Self {
            mark: carrir.mark.into(),
            space: carrir.space.into(),
        }
    }
}

impl convert::From<MarkAndSpaceMicros> for MarkAndSpaceIrCarrier {
    /// マークアンドスペース(マイクロ秒ベース)から
    /// マークアンドスペース(キャリア周波数カウンタ型ベース)へ
    /// 変換する
    fn from(micros: MarkAndSpaceMicros) -> Self {
        Self {
            mark: micros.mark.into(),
            space: micros.space.into(),
        }
    }
}

/// 第1,2,3...フレームを区切る時間(8ms = 8000us)
pub const THRESHOLD_FRAME_GAP: Microseconds = Microseconds(8000);

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
/// 赤外線リモコン信号フレーム
pub struct InfraredRemoteFrame(pub Vec<MarkAndSpaceMicros>);

impl iter::FromIterator<MarkAndSpaceMicros> for InfraredRemoteFrame {
    fn from_iter<T: IntoIterator<Item = MarkAndSpaceMicros>>(iter: T) -> Self {
        let mut c = InfraredRemoteFrame(Vec::new());
        for i in iter {
            c.0.push(i);
        }
        c
    }
}
