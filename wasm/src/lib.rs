// Copyright (c) 2023 Akihiro Yamamoto.
// Licensed under the MIT License <https://spdx.org/licenses/MIT.html>
// See LICENSE file in the project root for full license information.
//
mod devices;
mod infrared_remote;
mod parsing;

use infrared_remote::{
    decode_phase1, decode_phase2, decode_phase3, decode_phase4, InfraredRemoteControlCode,
    InfraredRemoteDecordedFrame, InfraredRemoteDemodulatedFrame, InfraredRemoteFrame,
    MarkAndSpaceMicros,
};
use nonempty::NonEmpty;
use parsing::parse_infrared_code_text;
use serde_wasm_bindgen::Error;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    alert("Hello, wasm!");
}

#[wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = r#"
export function wasm_parse_infrared_code(ircode: string): MarkAndSpaceMicros[];
"#;
#[wasm_bindgen(skip_typescript)]
pub fn wasm_parse_infrared_code(input: &str) -> Result<JsValue, Error> {
    parse_infrared_code_text(input)
        .map_err(|e| Error::new(e.to_string()))
        .and_then(|mark_and_spaces: Vec<MarkAndSpaceMicros>| {
            serde_wasm_bindgen::to_value(&mark_and_spaces)
        })
}

#[wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = r#"
export function wasm_decode_phase1(input: MarkAndSpaceMicros[]): InfraredRemoteFrame[];
"#;
#[wasm_bindgen(skip_typescript)]
pub fn wasm_decode_phase1(input: JsValue) -> Result<JsValue, Error> {
    serde_wasm_bindgen::from_value(input)
        .and_then(|mark_and_spaces: Vec<MarkAndSpaceMicros>| {
            decode_phase1(&mark_and_spaces).map_err(|e| Error::new(e.to_string()))
        })
        .and_then(|ir_frames: Vec<InfraredRemoteFrame>| serde_wasm_bindgen::to_value(&ir_frames))
}

#[wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = r#"
export function wasm_decode_phase2(input: InfraredRemoteFrame): InfraredRemoteDemodulatedFrame;
"#;
#[wasm_bindgen(skip_typescript)]
pub fn wasm_decode_phase2(input: JsValue) -> Result<JsValue, Error> {
    serde_wasm_bindgen::from_value(input)
        .and_then(|ir_frame: InfraredRemoteFrame| Ok(decode_phase2(&ir_frame)))
        .and_then(|demodulated_frame: InfraredRemoteDemodulatedFrame| {
            serde_wasm_bindgen::to_value(&demodulated_frame)
        })
}

#[wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = r#"
export function wasm_decode_phase3(input: InfraredRemoteDemodulatedFrame): InfraredRemoteDecordedFrame;
"#;
#[wasm_bindgen(skip_typescript)]
pub fn wasm_decode_phase3(input: JsValue) -> Result<JsValue, Error> {
    serde_wasm_bindgen::from_value(input)
        .and_then(|demodulated_frame: InfraredRemoteDemodulatedFrame| {
            decode_phase3(&demodulated_frame).map_err(|e| Error::new(e.to_string()))
        })
        .and_then(|v: InfraredRemoteDecordedFrame| serde_wasm_bindgen::to_value(&v))
}

#[wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = r#"
export function wasm_decode_phase4(input: InfraredRemoteDecordedFrame[]): InfraredRemoteControlCode[];
"#;
#[wasm_bindgen(skip_typescript)]
pub fn wasm_decode_phase4(input: JsValue) -> Result<JsValue, Error> {
    serde_wasm_bindgen::from_value(input)
        .and_then(|vs: Vec<InfraredRemoteDecordedFrame>| Ok(decode_phase4(&vs)))
        .and_then(|v: NonEmpty<InfraredRemoteControlCode>| {
            serde_wasm_bindgen::to_value(&Vec::from(v))
        })
}
