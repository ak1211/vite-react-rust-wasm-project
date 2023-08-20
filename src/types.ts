// Copyright (c) 2023 Akihiro Yamamoto.
// Licensed under the MIT License <https://spdx.org/licenses/MIT.html>
// See LICENSE file in the project root for full license information.
//

// 受信コード
//export type RxIrRemoteCode = MarkAndSpaceMicros[]

// 送信コード
//export type TxIrRemoteCode = InfraredRemoteDemodulatedFrame[]

/// マークアンドスペース(マイクロ秒ベース)
export interface MarkAndSpaceMicros {
	mark: number,
	space: number,
};

/// 赤外線リモコン信号フレーム
export type InfraredRemoteFrame = MarkAndSpaceMicros[];

/// 復調後の赤外線リモコン信号フレーム
export type InfraredRemoteDemodulatedFrame =
	| { Aeha: Uint8Array }
	| { Nec: Uint8Array }
	| { Sirc: Uint8Array }
	| { Unknown: MarkAndSpaceMicros[] }

/// 復号後の赤外線リモコン信号フレーム
export type InfraredRemoteDecordedFrame =
	| { Aeha: { octets: [Uint8Array], stop: number } }
	| { Nec: { custom0: Uint8Array, custom1: Uint8Array, data0: Uint8Array, data1: Uint8Array, stop: number } }
	| { Sirc12: { command: Uint8Array, address: Uint8Array } }
	| { Sirc15: { command: Uint8Array, address: Uint8Array } }
	| { Sirc20: { command: Uint8Array, address: Uint8Array, extended: Uint8Array } }
	| { Unknown: MarkAndSpaceMicros[] }

/// 赤外線リモコンコード
export type InfraredRemoteControlCode =
	| { Sirc: { command: string, device: string } }
	| { PanasonicHvac: { temperature: number, mode: string, switch: Boolean, swing: string, fan: string, profile: string, checksum: number } }
	| { DaikinHvac: { temperature: number, mode: string, on_timer: Boolean, on_timer_duration_hour: number, off_timer: Boolean, off_timer_duration_hour: number, switch: Boolean, swing: string, fan: string, checksum: number } }
	| { HitachiHvac: { temperature: number, mode: string, fan: string, switch: Boolean } }
	| { MitsubishiElectricHvac: { temperature: number, mode1: string, switch: Boolean, checksum: number } }
	| { Unknown: InfraredRemoteDecordedFrame[] }
