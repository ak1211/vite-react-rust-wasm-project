// Copyright (c) 2023 Akihiro Yamamoto.
// Licensed under the MIT License <https://spdx.org/licenses/MIT.html>
// See LICENSE file in the project root for full license information.
//
import {
  MarkAndSpaceMicros, InfraredRemoteControlCode, DecordedInfraredRemoteFrame,
  wasm_parse_infrared_code, wasm_decord_receiving_data, wasm_decord_ir_frames,
} from '../wasm/pkg/wasm'
import { useEffect, useState } from 'react';
import { Button, Input, Alert, Card, Space, Typography, Switch } from 'antd'
//import IrDecodedFrame from './IrDecodedFrame';
import IrSignal from './IrSignal';
import IrControlCode from './IrControlCode';
import IrDecordedFrame from './IrDecordedFrame';
import 'antd/dist/antd.min.css';
import './App.css';

const { TextArea } = Input;
const { Text, Title } = Typography;

interface State {
  msb_first: boolean,
  ir_mark_and_spaces: MarkAndSpaceMicros[],
  ir_decoded_frames: DecordedInfraredRemoteFrame[],
  ir_control_codes: InfraredRemoteControlCode[],
  input_text: string,
  alert: {
    type: 'success' | 'info' | 'warning' | 'error',
    message: string,
  },
};

const initState: State = {
  msb_first: false,
  ir_mark_and_spaces: [],
  ir_decoded_frames: [],
  ir_control_codes: [],
  input_text: '',
  alert: {
    type: 'info',
    message: '入力してね。',
  },
};

//
const App = (): JSX.Element => {
  const [state, setState] = useState<State>(initState)

  const handleReset = () => {
    setState(initState)
  }

  const handleParse = (input_text: string) => {
    if (input_text) {
      setState({ ...state, input_text: input_text });
    } else {
      handleReset();
    }
  }

  // ユーザー入力を解析する
  useEffect(
    () => {
      try {
        if (state.input_text.length) {
          const ir_mark_and_spaces = wasm_parse_infrared_code(state.input_text);
          setState(state => ({
            ...state, ir_mark_and_spaces: ir_mark_and_spaces,
            alert: { type: 'success', message: 'いいですね。' }
          }))
        } else {
          setState(state => ({ ...state, ir_mark_and_spaces: [] }))
        }
      } catch (err) {
        if (err instanceof Error) {
          let msg = err.message;
          setState(state => ({ ...state, ir_mark_and_spaces: [], alert: { type: 'error', message: msg } }))
        } else {
          throw err
        }
      }
    }
    , [state.input_text])

  // パルス列を復号する
  useEffect(
    () => {
      try {
        if (state.ir_mark_and_spaces.length) {
          const ir_decoded_frames = wasm_decord_receiving_data(state.ir_mark_and_spaces);
          setState(state => ({
            ...state, ir_decoded_frames: ir_decoded_frames,
            alert: { type: 'success', message: 'いいですね。' }
          }))
        } else {
          setState(state => ({ ...state, ir_decoded_frames: [] }))
        }
      } catch (err) {
        if (err instanceof Error) {
          let msg = err.message;
          setState(state => ({ ...state, ir_decoded_frames: [], alert: { type: 'error', message: msg } }))
        } else {
          throw err
        }
      }
    }
    , [state.ir_mark_and_spaces])

  /*
useEffect(
  () => {
    try {
      if (state.ir_demodulated_frames.length) {
        const ir_decoded_frames = state.ir_demodulated_frames.map((v) => wasm_decode_phase3(v));
        setState(state => ({
          ...state, ir_decoded_frames: ir_decoded_frames,
          alert: { type: 'success', message: 'いいですね。' }
        }))
      } else {
        setState(state => ({ ...state, ir_decoded_frames: [] }))
      }
    } catch (err) {
      if (err instanceof Error) {
        let msg = err.message;
        setState(state => ({ ...state, ir_decoded_frames: [], alert: { type: 'error', message: msg } }))
      } else {
        throw err
      }
    }
  }
  , [state.ir_demodulated_frames])
  */

  useEffect(
    () => {
      try {
        if (state.ir_decoded_frames.length) {
          const ir_control_codes = wasm_decord_ir_frames(state.ir_decoded_frames);
          setState(state => ({
            ...state, ir_control_codes: ir_control_codes,
            alert: { type: 'success', message: 'いいですね。' }
          }))
        } else {
          setState(state => ({ ...state, ir_control_codes: [] }))
        }
      } catch (err) {
        if (err instanceof Error) {
          let msg = err.message;
          setState(state => ({ ...state, ir_control_codes: [], alert: { type: 'error', message: msg } }))
        } else {
          throw err
        }
      }
    }
    , [state.ir_decoded_frames])

  return (
    <>
      <Space className='content' direction='vertical' size='middle' style={{ display: 'flex' }}>
        <Card size='small' title={<Title level={4}>解析する赤外線リモコン信号</Title>}>
          <Button type='primary' style={{ marginBottom: 3 }} onClick={handleReset}>Reset</Button>
          <TextArea
            rows={6}
            placeholder='ここに解析対象の赤外線リモコンコードを入れる。'
            value={state.input_text}
            onChange={(e) => { handleParse(e.target.value) }}
          />
          <Alert message={state.alert.message} type={state.alert.type} showIcon />
        </Card>
        <IrSignal ir_mark_and_spaces={state.ir_mark_and_spaces} />
        <Card size='small' title={<Title level={4}>復号信号</Title>}>
          <Switch unCheckedChildren="Least Significant Bit (LSB) first order" checkedChildren="Most Significant Bit (MSB) first order"
            onChange={(checked, _event) => { setState({ ...state, msb_first: checked }) }}
            checked={state.msb_first}
            id={'bit-order-selector'}
          />
          {state.ir_decoded_frames.map((decorded_frame: DecordedInfraredRemoteFrame, index: number) => {
            return <IrDecordedFrame key={'IrDecordedFrame' + index} msb_first={state.msb_first} decorded_frame={decorded_frame} index={index} />;
          })}
        </Card>
        <IrControlCode ir_control_codes={state.ir_control_codes} />
      </Space>
      <Text className='App-footer' >vite-react-rust-wasm-project &copy;2023 Akihiro Yamamoto</Text>
    </>
  );
}

export default App;