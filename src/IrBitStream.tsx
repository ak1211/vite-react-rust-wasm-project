// Copyright (c) 2023 Akihiro Yamamoto.
// Licensed under the MIT License <https://spdx.org/licenses/MIT.html>
// See LICENSE file in the project root for full license information.
//
import { wasm_decode_phase1, wasm_decode_phase2 } from '../wasm/pkg/wasm'
import { useState, useEffect } from 'react'
import { InfraredRemoteFrame, InfraredRemoteDemodulatedFrame } from './types'
import { Empty, Alert, Card, Radio, Space, Typography } from 'antd'
import IrDemodulatedFrame from './IrDemodulatedFrame';
import 'antd/dist/antd.min.css'

const { Title, Text, Paragraph } = Typography

type Props = {
  ir_frame: InfraredRemoteFrame,
}

type State = {
  msb_first: Boolean,
  ir_demodulated_frames: InfraredRemoteDemodulatedFrame[],
  alert: {
    type: 'success' | 'info' | 'warning' | 'error',
    message: string,
  },
};

const initState: State = {
  msb_first: false,
  ir_demodulated_frames: [],
  alert: {
    type: 'info',
    message: "",
  },
};

//
const IrBitStream = (props: Props): JSX.Element => {
  const [state, setState] = useState<State>(initState)

  useEffect(
    () => {
      if (props.ir_frame.length) {
        try {
          const demodulated_frames = wasm_decode_phase1(props.ir_frame)
            .map((x) => wasm_decode_phase2(x));
          setState({
            ...state,
            ir_demodulated_frames: demodulated_frames,
            alert: { type: 'success', message: 'デコード成功' },
          })
        } catch (error) {
          setState(state => ({ ...state, ir_demodulated_frames: [], message: error }));
        }
      } else {
        setState(state => ({ ...state, ir_demodulated_frames: [] }))
      }
    }
    , [props.ir_frame])

  //
  const content: JSX.Element =
    <>
      <Space direction='vertical' size='large' style={{ display: 'flex' }}>
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
        {state.ir_demodulated_frames.map((item, index) =>
          <IrDemodulatedFrame key={index} msb_first={state.msb_first} index={index} demodulated_frame={item} />
        )}
        <Alert message={state.alert.message} type={state.alert.type} showIcon />
      </Space>
    </>

  return (
    <Card size='small' title={<Title level={4}>ビットストリーム</Title>}>
      {state.ir_demodulated_frames.length > 0 ? content : <Empty image={Empty.PRESENTED_IMAGE_SIMPLE} />}
    </Card >
  )
}

export default IrBitStream;
