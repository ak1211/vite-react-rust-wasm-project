// Copyright (c) 2023 Akihiro Yamamoto.
// Licensed under the MIT License <https://spdx.org/licenses/MIT.html>
// See LICENSE file in the project root for full license information.
//
import { wasm_parse_infrared_code, wasm_decode_phase1, wasm_decode_phase2, wasm_decode_phase3, wasm_decode_phase4 } from '../wasm/pkg/wasm'
import { useEffect, useState } from 'react';
import { MarkAndSpaceMicros, InfraredRemoteDemodulatedFrame, InfraredRemoteFrame, InfraredRemoteControlCode, InfraredRemoteDecordedFrame } from './types';
import { Button, Input, Alert, Card, Radio, Space, Typography } from 'antd'
import IrDecodedFrame from './IrDecodedFrame';
import IrSignal from './IrSignal';
import IrControlCode from './IrControlCode';
import IrDemodulatedFrame from './IrDemodulatedFrame';
import 'antd/dist/antd.min.css';
import './App.css';

const { TextArea } = Input;
const { Text, Title, Paragraph } = Typography;

interface State {
  msb_first: Boolean,
  ir_mark_and_spaces: MarkAndSpaceMicros[],
  ir_splitted_frames: InfraredRemoteFrame[],
  ir_demodulated_frames: InfraredRemoteDemodulatedFrame[],
  ir_decoded_frames: InfraredRemoteDecordedFrame[],
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
  ir_splitted_frames: [],
  ir_demodulated_frames: [],
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

  useEffect(
    () => {
      if (state.input_text.length == 0) {
        setState(initState);
        return;
      }
      try {
        const ir_mark_and_spaces: MarkAndSpaceMicros[] = wasm_parse_infrared_code(state.input_text);
        setState(state => ({ ...state, ir_mark_and_spaces: ir_mark_and_spaces, alert: { type: 'success', message: 'いいですね。' } }))
      } catch (error) {
        if (error instanceof Error) {
          let msg = error.message;
          setState(state => ({ ...state, alert: { type: 'error', message: msg } }))
        } else {
          throw error
        }
      }
    }
    , [state.input_text])

  useEffect(
    () => {
      if (state.ir_mark_and_spaces.length == 0) {
        setState(state => ({ ...state, ir_splitted_frames: [] }))
        return;
      }
      try {
        const ir_splitted_frames: InfraredRemoteFrame[] = wasm_decode_phase1(state.ir_mark_and_spaces);
        setState(state => ({ ...state, ir_splitted_frames: ir_splitted_frames, alert: { type: 'success', message: 'いいですね。' } }))
      } catch (error) {
        if (error instanceof Error) {
          let msg = error.message;
          setState(state => ({ ...state, ir_splitted_frames: [], alert: { type: 'error', message: msg } }))
        } else {
          throw error
        }
      }
    }
    , [state.ir_mark_and_spaces])

  useEffect(
    () => {
      if (state.ir_splitted_frames.length == 0) {
        setState(state => ({ ...state, ir_demodulated_frames: [] }))
        return;
      }
      try {
        const ir_demodulated_frames = state.ir_splitted_frames.map((v) => wasm_decode_phase2(v));
        setState(state => ({ ...state, ir_demodulated_frames: ir_demodulated_frames, alert: { type: 'success', message: 'いいですね。' } }))
      } catch (error) {
        if (error instanceof Error) {
          let msg = error.message;
          setState(state => ({ ...state, ir_demodulated_frames: [], alert: { type: 'error', message: msg } }))
        } else {
          throw error
        }
      }
    }
    , [state.ir_splitted_frames])

  useEffect(
    () => {
      if (state.ir_demodulated_frames.length == 0) {
        setState(state => ({ ...state, ir_decoded_frames: [] }))
        return;
      }
      try {
        const ir_decoded_frames = state.ir_demodulated_frames.map((v) => wasm_decode_phase3(v));
        setState(state => ({ ...state, ir_decoded_frames: ir_decoded_frames, alert: { type: 'success', message: 'いいですね。' } }))
      } catch (error) {
        if (error instanceof Error) {
          let msg = error.message;
          setState(state => ({ ...state, ir_decoded_frames: [], alert: { type: 'error', message: msg } }))
        } else {
          throw error
        }
      }
    }
    , [state.ir_demodulated_frames])

  useEffect(
    () => {
      if (state.ir_decoded_frames.length == 0) {
        setState(state => ({ ...state, ir_control_codes: [] }))
        return;
      }
      try {
        const ir_control_codes = wasm_decode_phase4(state.ir_decoded_frames);
        setState(state => ({ ...state, ir_control_codes: ir_control_codes, alert: { type: 'success', message: 'いいですね。' } }))
      } catch (error) {
        if (error instanceof Error) {
          let msg = error.message;
          setState(state => ({ ...state, ir_control_codes: [], alert: { type: 'error', message: msg } }))
        } else {
          throw error
        }
      }
    }
    , [state.ir_decoded_frames])

  const handleReset = () => {
    setState(initState)
  }

  const handleParse = (input_text: string) => {
    handleReset();
    setState({ ...state, input_text: input_text })
  }

  return (
    <>
      <Space className='content' direction='vertical' size='middle' style={{ display: 'flex' }}>
        <Card size='small' title={<Title level={4}>解析する赤外線リモコン信号</Title>}>
          <Paragraph>
            <Text>ビットオーダー&nbsp;</Text>
            <Radio.Group
              name='radiogroup'
              defaultValue={state.msb_first === true ? 1 : 0}
              onChange={e => { setState({ ...state, msb_first: e.target.value }) }}
              optionType='button'
              buttonStyle='solid'
            >
              <Radio value={0}>Least Significant Bit (LSB) first</Radio>
              <Radio value={1}>Most Significant Bit (MSB) first</Radio>
            </Radio.Group>
          </Paragraph>
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
        <Card size='small' title={<Title level={4}>復調(Demodulated)信号</Title>}>
          {state.ir_demodulated_frames.map((demodulated_frame: InfraredRemoteDemodulatedFrame, index: number) => {
            return <IrDemodulatedFrame key={'IrDemodulatedFrame' + index} msb_first={state.msb_first} ir_demodulated_frame={demodulated_frame} index={index} />;
          })}
        </Card>
        <Card size='small' title={<Title level={4}>復号(Decorded)信号</Title>}>
          {state.ir_decoded_frames.map((decoded_frame: InfraredRemoteDecordedFrame, index: number) => {
            return <IrDecodedFrame key={'IrDecodedFrame' + index} msb_first={state.msb_first} decorded_frame={decoded_frame} index={index} />;
          })}
        </Card>
        <IrControlCode ir_control_codes={state.ir_control_codes} />
      </Space>
      <Text className='App-footer' >vite-react-rust-wasm-project &copy;2023 Akihiro Yamamoto</Text>
    </>
  );
}

export default App;