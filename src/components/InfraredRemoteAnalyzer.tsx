// Copyright (c) 2023 Akihiro Yamamoto.
// Licensed under the MIT License <https://spdx.org/licenses/MIT.html>
// See LICENSE file in the project root for full license information.
//
import {
  MarkAndSpaceMicros, InfraredRemoteControlCode, DecordedInfraredRemoteFrame,
  wasm_parse_infrared_code, wasm_decord_receiving_data, wasm_decord_ir_frames,
} from '../../wasm/pkg/wasm'
import { useEffect, useState } from 'react';
import { useRecoilValue } from 'recoil';
import { message, Button, Table, Input, Alert, Card, Space, Typography, Switch } from 'antd'
import type { ColumnsType } from 'antd/es/table'
import IrSignal from './IrSignal';
import IrControlCode from './IrControlCode';
import IrDecordedFrame from './IrDecordedFrame';
import * as SC from './SerialConnectionEffect';

const { TextArea } = Input;
const { Title, Text } = Typography;

interface State {
  msb_first: boolean,
  ir_mark_and_spaces: MarkAndSpaceMicros[],
  ir_decoded_frames: DecordedInfraredRemoteFrame[],
  ir_control_codes: InfraredRemoteControlCode[],
  input_text: string,
  confirmed_rx_json_data_length: number,
  automatically_paste: boolean,
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
  confirmed_rx_json_data_length: 0,
  automatically_paste: false,
  alert: {
    type: 'info',
    message: '入力してね。',
  },
};

//
export default function InfraredRemoteAnalyzer() {
  //
  const [messageApi, contextHolder] = message.useMessage();
  //
  const rxJsonData = useRecoilValue(SC.rxJsonDataState);
  //
  const [state, setState] = useState<State>(initState)

  const handleReset = () => {
    setState(initState)
  }

  const handleParse = (input_text: string) => {
    if (input_text) {
      setState(prev => ({ ...prev, input_text: input_text }));
    } else {
      handleReset();
    }
  }

  useEffect(() => {
    if (rxJsonData.length != state.confirmed_rx_json_data_length) {
      const newone: number = state.confirmed_rx_json_data_length;
      const text: string = `new IR code (#${newone}) arrival.`;
      messageApi.open({ key: text, type: 'success', content: text, });
      setState(prev => ({ ...prev, confirmed_rx_json_data_length: rxJsonData.length }));
      if (state.automatically_paste) {
        const new_input_text = JSON.stringify(rxJsonData[newone].rawData);
        setState(prev => ({ ...prev, input_text: new_input_text, }));
      }
    }
  }, [rxJsonData.length, state]);


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
          setState(prev => ({ ...prev, ir_mark_and_spaces: [] }))
        }
      } catch (err) {
        if (err instanceof Error) {
          let msg = err.message;
          setState(prev => ({ ...prev, ir_mark_and_spaces: [], alert: { type: 'error', message: msg } }))
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
          setState(prev => ({ ...prev, ir_decoded_frames: [] }))
        }
      } catch (err) {
        if (err instanceof Error) {
          let msg = err.message;
          setState(prev => ({ ...prev, ir_decoded_frames: [], alert: { type: 'error', message: msg } }))
        } else {
          throw err
        }
      }
    }
    , [state.ir_mark_and_spaces])

  useEffect(
    () => {
      try {
        if (state.ir_decoded_frames.length) {
          const ir_control_codes = wasm_decord_ir_frames(state.ir_decoded_frames);
          setState(prev => ({
            ...prev, ir_control_codes: ir_control_codes,
            alert: { type: 'success', message: 'いいですね。' }
          }))
        } else {
          setState(prev => ({ ...prev, ir_control_codes: [] }))
        }
      } catch (err) {
        if (err instanceof Error) {
          let msg = err.message;
          setState(prev => ({ ...prev, ir_control_codes: [], alert: { type: 'error', message: msg } }))
        } else {
          throw err
        }
      }
    }
    , [state.ir_decoded_frames])

  interface RxJsonSchemaWithIndexAndKey extends SC.RxJsonSchema {
    index: number,
    key: any,
  };

  const columns: ColumnsType<RxJsonSchemaWithIndexAndKey> = [
    {
      width: '5em',
      title: 'Index',
      dataIndex: 'index',
      key: 'index',
      render: (n: number) => {
        return (<>#{n}</>)
      }
    },
    {
      width: '9em',
      title: 'Timestamp',
      dataIndex: 'timestamp',
      key: 'timestamp',
      render: (n) => { return (n) }
    },
    {
      title: 'rawData',
      dataIndex: 'rawData',
      key: 'rawData',
      render: (data: number[]) => {
        const original: string = data.toString();
        const cropped: string = original.substring(0, 200);
        return ((original.length === cropped.length) ? original : cropped + ' ....');
      }
    },
  ]

  return (
    <>
      {contextHolder}
      <Title>赤外線リモコン信号解析</Title>
      <Space direction='vertical' size='middle'>
        <Card
          size='small'
          title={<Title level={4}>外部デバイスから受信した信号</Title>}
        >
          <Table
            onRow={(record) => {
              return {
                onClick: (_event) => {
                  messageApi.open({ type: 'success', content: '貼り付けました。' })
                  setState(prev => ({ ...prev, input_text: JSON.stringify(record.rawData) }));
                },
                onDoubleClick: (_event) => { },
                onMouseEnter: (_event) => { },
                onMouseLeave: (_event) => { },
              }
            }}
            dataSource={rxJsonData.map((item, index) => ({ ...item, index: index, key: item.timestamp, }))}
            columns={columns}
            scroll={{ y: 240 }}
          />
          <Space>
            <Text>自動貼り付け</Text>
            <Switch
              title={"Automatically paste new ones into textarea."}
              onChange={(checked) => {
                setState(prev => ({ ...prev, automatically_paste: checked }))
              }}
            />
          </Space>
        </Card>
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
            onChange={(checked, _event) => setState(prev => ({ ...prev, msb_first: checked }))}
            checked={state.msb_first}
            id={'bit-order-selector'}
          />
          {state.ir_decoded_frames.map((decorded_frame: DecordedInfraredRemoteFrame, index: number) => {
            return <IrDecordedFrame key={'IrDecordedFrame' + index} msb_first={state.msb_first} decorded_frame={decorded_frame} index={index} />;
          })}
        </Card>
        <IrControlCode ir_control_codes={state.ir_control_codes} />
      </Space >
    </>
  );
}