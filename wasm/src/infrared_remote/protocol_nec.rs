// NECプロトコルの定義
//
// Copyright (c) 2023 Akihiro Yamamoto.
// Licensed under the MIT License <https://spdx.org/licenses/MIT.html>
// See LICENSE file in the project root for full license information.
use crate::infrared_remote::{Bit, MarkAndSpace, MarkAndSpaceMicros, Microseconds};
use std::ops::Range;

/// 基準時間 T = 562 μ秒
pub const TIME_BASE: Microseconds = Microseconds(562);

/// リーダーパルス
/// H-level width, 16 * T(562us) = typical 8992us
/// L-level width, 8 * T(562us) = typical 4496us
pub const LEADER: MarkAndSpaceMicros = MarkAndSpace {
    mark: Microseconds(16 * TIME_BASE.0),
    space: Microseconds(8 * TIME_BASE.0),
};

/// リピートパルス
/// H-level width, 16 * T(562us) = typical 8992us
/// L-level width, 4 * T(562us) = typical 2248 us
pub const REPEAT: MarkAndSpaceMicros = MarkAndSpace {
    mark: Microseconds(16 * TIME_BASE.0),
    space: Microseconds(4 * TIME_BASE.0),
};

/// 0を意味する信号
/// H-level width, 1 * T(562us) = typical 562us
/// L-level width, 1 * T(562us) = typical 562us
pub const TYPICAL_BIT_ZERO: MarkAndSpaceMicros = MarkAndSpace {
    mark: TIME_BASE,
    space: TIME_BASE,
};

/// 1を意味する信号
/// H-level width, 1 * T(562us) = typical 562us
/// L-level width, 3 * T(562us) = typical 1686us
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
    let nec: MarkAndSpace<Range<Microseconds>> = MarkAndSpace {
        mark: Range {
            start: LEADER.mark - tolerance,
            end: LEADER.mark + tolerance,
        },
        space: Range {
            start: LEADER.space - tolerance,
            end: LEADER.space + tolerance,
        },
    };
    nec.mark.contains(&test.mark) && nec.space.contains(&test.space)
}

///
pub fn compare_repeat_pulse(tolerance: Microseconds, test: &MarkAndSpaceMicros) -> bool {
    let nec_repeat: MarkAndSpace<Range<Microseconds>> = MarkAndSpace {
        mark: Range {
            start: REPEAT.mark - tolerance,
            end: REPEAT.mark + tolerance,
        },
        space: Range {
            start: REPEAT.space - tolerance,
            end: REPEAT.space + tolerance,
        },
    };
    nec_repeat.mark.contains(&test.mark) && nec_repeat.space.contains(&test.space)
}
