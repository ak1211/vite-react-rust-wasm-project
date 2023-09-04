// 家製協プロトコルの定義
//
// Copyright (c) 2023 Akihiro Yamamoto.
// Licensed under the MIT License <https://spdx.org/licenses/MIT.html>
// See LICENSE file in the project root for full license information.
use crate::infrared_remote::{Bit, MarkAndSpace, MarkAndSpaceMicros, Microseconds};
use std::ops::Range;

/// 基準時間 350us ～ 500us typical 425. T = 440 μ秒(実測)
pub const TIME_BASE: Microseconds = Microseconds(440);

/// リーダーパルス
/// H-level width, 8 * T(425us) = typical 3400us
/// L-level width, 4 * T(425us) = typical 1700us
pub const LEADER: MarkAndSpaceMicros = MarkAndSpace {
    mark: Microseconds(8 * TIME_BASE.0),
    space: Microseconds(4 * TIME_BASE.0),
};

/// 0を意味する信号
/// H-level width, 1 * T(425us) = typical 425us
/// L-level width, 1 * T(425us) = typical 425us
pub const TYPICAL_BIT_ZERO: MarkAndSpaceMicros = MarkAndSpace {
    mark: TIME_BASE,
    space: TIME_BASE,
};

/// 1を意味する信号
/// H-level width, 1 * T(425us) = typical 425us
/// L-level width, 3 * T(425us) = typical 1275us
pub const TYPICAL_BIT_ONE: MarkAndSpaceMicros = MarkAndSpace {
    mark: TIME_BASE,
    space: Microseconds(3 * TIME_BASE.0),
};

/// pulse distance modulation
pub fn modulate(bit: Bit) -> MarkAndSpaceMicros {
    match bit {
        Bit::Hi => TYPICAL_BIT_ONE,
        Bit::Lo => TYPICAL_BIT_ZERO,
    }
}

/// pulse distance modulation
pub fn demodulate(x: MarkAndSpaceMicros) -> Bit {
    if x.mark + x.mark <= x.space {
        // マーク時間の２倍以上スペース時間があれば
        Bit::Hi
    } else {
        Bit::Lo
    }
}

///
pub fn compare_leader_pulse(tolerance: Microseconds, test: &MarkAndSpaceMicros) -> bool {
    let aeha: MarkAndSpace<Range<Microseconds>> = MarkAndSpace {
        mark: Range {
            start: LEADER.mark - tolerance,
            end: LEADER.mark + tolerance,
        },
        space: Range {
            start: LEADER.space - tolerance,
            end: LEADER.space + tolerance,
        },
    };

    aeha.mark.contains(&test.mark) && aeha.space.contains(&test.space)
}
