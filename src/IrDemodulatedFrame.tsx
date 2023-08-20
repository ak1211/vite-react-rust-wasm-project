// Copyright (c) 2023 Akihiro Yamamoto.
// Licensed under the MIT License <https://spdx.org/licenses/MIT.html>
// See LICENSE file in the project root for full license information.
//
import { wasm_decode_phase3 } from '../wasm/pkg/wasm'
import { useState, useEffect } from 'react'
import { Statistic, Typography, Descriptions } from 'antd'
import { InfraredRemoteDemodulatedFrame, InfraredRemoteDecordedFrame } from './types'
import 'antd/dist/antd.min.css'

const { Text } = Typography

type Props = {
  msb_first: Boolean,
  index: number,
  demodulated_frame: InfraredRemoteDemodulatedFrame,
}

type State = {
  decorded_frame: InfraredRemoteDecordedFrame | undefined,
  bitstream: Uint8Array,
};

const initState: State = {
  decorded_frame: undefined,
  bitstream: new Uint8Array(),
};

// 16進表記にする
const to_hex_string = (octets: Uint8Array): string => {
  return octets
    .reduce((acc, x) => 2 * acc + x, 0)
    .toString(16)
    .padStart(2, '0')
}

//
const descriptions_item = (msb_first: Boolean, key: any, label: string, xs: Uint8Array) => {
  return (<Descriptions.Item key={key} label={label} style={{ textAlign: 'center' }} span={1}>
    <Text> {xs.join('')}</Text>
    <Statistic title='hex' value={to_hex_string(msb_first ? xs : xs.slice().reverse())} />
  </Descriptions.Item >);
};

//
const display_decorded_frame = (msb_first: Boolean, decorded_frame: InfraredRemoteDecordedFrame): JSX.Element => {
  var frame_label: string = '';
  var display_items = (): JSX.Element[] => { return [] };
  if ('Aeha' in decorded_frame) {
    frame_label = 'AEHA';
    display_items = (): JSX.Element[] => {
      return (decorded_frame.Aeha.octets.map((xs, index) => {
        let offset = 8 * index;
        let label = 'offset ' + offset;
        return descriptions_item(msb_first, label, label, xs);
      }))
    };
  } else if ('Nec' in decorded_frame) {
    frame_label = 'NEC'
    display_items = (): JSX.Element[] => {
      const list = [
        { label: 'custom0', value: decorded_frame.Nec.custom0 },
        { label: 'custom1', value: decorded_frame.Nec.custom1 },
        { label: 'data0', value: decorded_frame.Nec.data0 },
        { label: 'data1', value: decorded_frame.Nec.data1 },
      ];
      return (list.map((xs) => { return descriptions_item(msb_first, xs.label, xs.label, xs.value); }));
    };
  } else if ('Sirc12' in decorded_frame) {
    frame_label = 'SIRC12'
    display_items = (): JSX.Element[] => {
      const list = [
        { label: 'command', value: decorded_frame.Sirc12.command },
        { label: 'address', value: decorded_frame.Sirc12.address },
      ];
      return (list.map((xs) => { return descriptions_item(msb_first, xs.label, xs.label, xs.value); }));
    };
  } else if ('Sirc15' in decorded_frame) {
    frame_label = 'SIRC15'
    display_items = (): JSX.Element[] => {
      const list = [
        { label: 'command', value: decorded_frame.Sirc15.command },
        { label: 'address', value: decorded_frame.Sirc15.address },
      ];
      return (list.map((xs) => { return descriptions_item(msb_first, xs.label, xs.label, xs.value); }));
    };
  } else if ('Sirc20' in decorded_frame) {
    frame_label = 'SIRC20'
    display_items = (): JSX.Element[] => {
      const list = [
        { label: 'command', value: decorded_frame.Sirc20.command },
        { label: 'address', value: decorded_frame.Sirc20.address },
        { label: 'extended', value: decorded_frame.Sirc20.extended },
      ];
      return (list.map((xs) => { return descriptions_item(msb_first, xs.label, xs.label, xs.value); }));
    };
  } else if ('Unknown' in decorded_frame) {
    frame_label = 'UNKNOWN'
  } else {
    throw new Error('unimplemented')
  }
  return (<>
    <Descriptions.Item label='Protocol' style={{ textAlign: 'center' }} span={1}>
      <Statistic value={frame_label} />
    </Descriptions.Item>
    {display_items()}
  </>);
}

//
const IrDemodulatedFrame = (props: Props): JSX.Element => {
  const [state, setState] = useState<State>(initState);

  useEffect(
    () => {
      var bitstream = new Uint8Array();
      if ('Aeha' in props.demodulated_frame) {
        bitstream = props.demodulated_frame.Aeha
      } else if ('Nec' in props.demodulated_frame) {
        bitstream = props.demodulated_frame.Nec
      } else if ('Sirc' in props.demodulated_frame) {
        bitstream = props.demodulated_frame.Sirc
      } else if ('Unknown' in props.demodulated_frame) {
      } else {
        console.log(props.demodulated_frame)
        throw new Error('unimplemented')
      }
      setState(state => ({
        ...state,
        decorded_frame: wasm_decode_phase3(props.demodulated_frame),
        bitstream: bitstream,
      }));
    }
    , [props.demodulated_frame]);

  return (
    <>
      <Descriptions title={'Frame# ' + (1 + props.index)} >
        <Descriptions.Item key={state.bitstream.join('')} label={'Bitstream ' + state.bitstream.length + ' bits(with stop bit)'}>
          <Text>{state.bitstream.join('')}</Text>
        </Descriptions.Item>
      </Descriptions>
      <Descriptions
        layout='vertical'
        column={{ xxl: 12, xl: 8, lg: 6, md: 4, sm: 2, xs: 1 }}
        bordered>
        {state.decorded_frame && display_decorded_frame(props.msb_first, state.decorded_frame)}
      </Descriptions>
    </>
  )
}

export default IrDemodulatedFrame;