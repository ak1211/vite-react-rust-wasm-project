// SIRCプロトコルの定義
//
// Copyright (c) 2023 Akihiro Yamamoto.
// Licensed under the MIT License <https://spdx.org/licenses/MIT.html>
// See LICENSE file in the project root for full license information.
use crate::infrared_remote::{Bit, MarkAndSpace, MarkAndSpaceMicros, Microseconds};
use std::ops::Range;

/// 基準時間 T = 600 μ秒
pub const TIME_BASE: Microseconds = Microseconds(600);

/// リーダーパルス
/// H-level width, 4 * T(600us) = typical 2400us
/// L-level width, 1 * T(600us) = typical 600us
pub const LEADER: MarkAndSpaceMicros = MarkAndSpace {
    mark: Microseconds(4 * TIME_BASE.0),
    space: Microseconds(1 * TIME_BASE.0),
};

/// 0を意味する信号
/// H-level width, 1 * T(600us) = typical 600us
/// L-level width, 1 * T(600us) = typical 600us
pub const TYPICAL_BIT_ZERO: MarkAndSpaceMicros = MarkAndSpace {
    mark: TIME_BASE,
    space: TIME_BASE,
};

/// 1を意味する信号
/// H-level width, 2 * T(600us) = typical 1200us
/// L-level width, 1 * T(600us) = typical 600us
pub const TYPICAL_BIT_ONE: MarkAndSpaceMicros = MarkAndSpace {
    mark: Microseconds(2 * TIME_BASE.0),
    space: TIME_BASE,
};

/// pulse width modulation: SIRC
pub fn modulate(bit: Bit) -> MarkAndSpaceMicros {
    match bit {
        Bit::Hi => TYPICAL_BIT_ONE,
        Bit::Lo => TYPICAL_BIT_ZERO,
    }
}

/// pulse width modulation: SIRC
pub fn demodulate(x: MarkAndSpaceMicros) -> Bit {
    // upper lower tolerance 0.1ms = 100us
    let tolerance = Microseconds(100);
    let threshold = Microseconds(1200);
    let upper = threshold + tolerance;
    let lower = threshold - tolerance;
    if lower <= x.mark && x.mark <= upper {
        // マーク時間が閾値(1200us)付近なら
        Bit::Hi
    } else {
        Bit::Lo
    }
}

///
pub fn compare_leader_pulse(tolerance: Microseconds, test: &MarkAndSpaceMicros) -> bool {
    let sirc: MarkAndSpace<Range<Microseconds>> = MarkAndSpace {
        mark: Range {
            start: LEADER.mark - tolerance,
            end: LEADER.mark + tolerance,
        },
        space: Range {
            start: LEADER.space - tolerance,
            end: LEADER.space + tolerance,
        },
    };

    sirc.mark.contains(&test.mark) && sirc.space.contains(&test.space)
}
