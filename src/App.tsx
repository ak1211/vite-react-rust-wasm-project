// Copyright (c) 2023 Akihiro Yamamoto.
// Licensed under the MIT License <https://spdx.org/licenses/MIT.html>
// See LICENSE file in the project root for full license information.
//
import { wasm_parse_infrared_code } from '../wasm/pkg/wasm'
import { useEffect, useState } from 'react';
import { InfraredRemoteFrame } from './types';
import { Button, Card, Input, Alert, Typography, Space } from 'antd';
import init from '../wasm/pkg/wasm'
import IrSignal from './IrSignal';
import IrControlCode from './IrControlCode';
import IrBitStream from './IrBitStream';
import 'antd/dist/antd.min.css';
import './App.css';

const { TextArea } = Input;
const { Text, Title } = Typography;

interface State {
  ir_frame: InfraredRemoteFrame,
  text: string,
  alert: {
    type: 'success' | 'info' | 'warning' | 'error',
    message: string,
  },
};

const initState: State = {
  ir_frame: [],
  text: '',
  alert: {
    type: 'info',
    message: '入力してね。',
  },
};

//
const App = (): JSX.Element => {
  useEffect(() => {
    init()
  }, [])

  const [state, setState] = useState<State>(initState)

  const handleReset = () => {
    setState(initState)
  }

  const handleParse = (text: string) => {
    setState({ ...state, text: text })
    try {
      const ir_frame = wasm_parse_infrared_code(text);
      setState(state => ({ ...state, ir_frame: ir_frame, alert: { type: 'success', message: 'いいですね。' } }))
    } catch (error) {
      const err = error as string;
      setState(state => ({ ...state, alert: { type: 'error', message: err } }))
    }
  }

  return (
    <>
      <Space className='content' direction='vertical' size='middle' style={{ display: 'flex' }}>
        <Card size='small' title={<Title level={4}>解析する赤外線リモコン信号</Title>}>
          <Button type='primary' style={{ marginBottom: 3 }} onClick={handleReset}>Reset</Button>
          <TextArea
            rows={6}
            placeholder='ここに解析対象の赤外線リモコンコードを入れる。'
            value={state.text}
            onChange={(e) => { handleParse(e.target.value) }}
          />
          <Alert message={state.alert.message} type={state.alert.type} showIcon />
        </Card>
        <IrSignal ir_frame={state.ir_frame} />
        <IrBitStream ir_frame={state.ir_frame} />
        <IrControlCode ir_frame={state.ir_frame} />
      </Space>
      <Text className='App-footer' >vite-react-rust-wasm-project &copy;2023 Akihiro Yamamoto</Text>
    </>
  );
}

export default App;