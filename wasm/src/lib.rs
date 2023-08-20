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
use parsing::parse_infrared_code_text;
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
export function wasm_parse_infrared_code(ircode: string): MarkAndSpaceMicros;
"#;
#[wasm_bindgen(skip_typescript)]
pub fn wasm_parse_infrared_code(ircode: &str) -> Result<JsValue, String> {
    let mark_and_spaces: Vec<MarkAndSpaceMicros> = parse_infrared_code_text(ircode)?;
    serde_wasm_bindgen::to_value(&mark_and_spaces).map_err(|e| e.to_string())
}

#[wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = r#"
export function wasm_decode_phase1(input: MarkAndSpaceMicros[]): InfraredRemoteFrame[];
"#;
#[wasm_bindgen(skip_typescript)]
pub fn wasm_decode_phase1(input: JsValue) -> Result<JsValue, String> {
    let mark_and_spaces: Vec<MarkAndSpaceMicros> =
        serde_wasm_bindgen::from_value(input).map_err(|e| e.to_string())?;
    let ir_frames: Vec<InfraredRemoteFrame> = decode_phase1(&mark_and_spaces)?;
    serde_wasm_bindgen::to_value(&ir_frames).map_err(|e| e.to_string())
}

#[wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = r#"
export function wasm_decode_phase2(input: InfraredRemoteFrame): InfraredRemoteDemodulatedFrame;
"#;
#[wasm_bindgen(skip_typescript)]
pub fn wasm_decode_phase2(input: JsValue) -> Result<JsValue, String> {
    let ir_frame: InfraredRemoteFrame =
        serde_wasm_bindgen::from_value(input).map_err(|e| e.to_string())?;
    let demodulated_frame: InfraredRemoteDemodulatedFrame = decode_phase2(&ir_frame);
    serde_wasm_bindgen::to_value(&demodulated_frame).map_err(|e| e.to_string())
}

#[wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = r#"
export function wasm_decode_phase3(input: InfraredRemoteDemodulatedFrame): InfraredRemoteDecordedFrame;
"#;
#[wasm_bindgen(skip_typescript)]
pub fn wasm_decode_phase3(input: JsValue) -> Result<JsValue, String> {
    let demodulated_frame: InfraredRemoteDemodulatedFrame =
        serde_wasm_bindgen::from_value(input).map_err(|e| e.to_string())?;
    let protocol: InfraredRemoteDecordedFrame = decode_phase3(&demodulated_frame)?;
    serde_wasm_bindgen::to_value(&protocol).map_err(|e| e.to_string())
}

#[wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = r#"
export function wasm_decode_phase4(input: InfraredRemoteDecordedFrame[]): InfraredRemoteControlCode[];
"#;
#[wasm_bindgen(skip_typescript)]
pub fn wasm_decode_phase4(input: JsValue) -> Result<JsValue, String> {
    let protocols: Vec<InfraredRemoteDecordedFrame> =
        serde_wasm_bindgen::from_value(input).map_err(|e| e.to_string())?;
    let ir_codes: Vec<InfraredRemoteControlCode> = decode_phase4(&protocols).into();
    serde_wasm_bindgen::to_value(&ir_codes).map_err(|e| e.to_string())
}
