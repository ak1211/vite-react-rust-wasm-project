// Copyright (c) 2023 Akihiro Yamamoto.
// Licensed under the MIT License <https://spdx.org/licenses/MIT.html>
// See LICENSE file in the project root for full license information.
//
import { DecordedInfraredRemoteFrame } from '../wasm/pkg/wasm';
import { Statistic, Typography, Descriptions } from 'antd'
import 'antd/dist/antd.min.css'

const { Text, Title } = Typography

type Props = {
  msb_first: Boolean,
  index: number,
  decorded_frame: DecordedInfraredRemoteFrame,
}

// オクテット単位にまとめる
const to_octets = (bitstream: Uint8Array): Uint8Array[] => {
  var output: Uint8Array[] = []
  for (let i = 0; i < bitstream.length; i += 8) {
    output.push(bitstream.slice(i, i + 8))
  }
  return output
}

// 16進表記にする
const to_hex_string = (msb_first: Boolean, octets: Uint8Array): string => {
  let xs: Uint8Array = msb_first ? octets : octets.slice().reverse();
  return xs
    .reduce((acc, x) => 2 * acc + x, 0)
    .toString(16)
    .padStart(2, '0')
}

//
type IrFrameValue = {
  frame_label: string,
  items: {
    item_label: string,
    value: Uint8Array,
  }[],
};

//
const descriptions_item = (msb_first: Boolean, key: any, label: string, bitstream: Uint8Array): JSX.Element => {
  return (<Descriptions.Item key={key} label={label} style={{ textAlign: 'center' }} span={1}>
    <Text> {bitstream.join('')}</Text>
    <Statistic title='hex' value={to_hex_string(msb_first, bitstream)} />
  </Descriptions.Item>);
};

//
const ir_frame_value = (decorded_frame: DecordedInfraredRemoteFrame): IrFrameValue => {
  if ('Aeha' in decorded_frame) {
    const octets = to_octets(decorded_frame.Aeha);
    return {
      frame_label: 'AEHA',
      items: octets.map((x, index) => {
        let offset = 8 * index;
        let label = 'offset ' + offset;
        return {
          item_label: label,
          value: x,
        };
      })
    };
  } else if ('Nec' in decorded_frame) {
    return {
      frame_label: 'NEC',
      items: [
        { item_label: 'Address', value: decorded_frame.Nec.slice(0, 8) },
        { item_label: '(Logical Inverse) Address', value: decorded_frame.Nec.slice(8, 16) },
        { item_label: 'Command', value: decorded_frame.Nec.slice(16, 24) },
        { item_label: '(Logical Inverse) Command', value: decorded_frame.Nec.slice(24, 32) },
      ],
    };
  } else if ('Sirc' in decorded_frame) {
    return {
      frame_label: 'SIRC',
      items: [
        { item_label: 'command', value: decorded_frame.Sirc.slice(0, 5) },
        { item_label: 'address', value: decorded_frame.Sirc.slice(5) },
      ],
    };
  } else if ('Unknown' in decorded_frame) {
    return {
      frame_label: 'UNKNOWN',
      items: [],
    };
  } else {
    throw new Error('unimplemented')
  }
}

//
const display_decorded_frame = (msb_first: Boolean, frame_value: IrFrameValue): JSX.Element => {
  return (<>
    <Descriptions.Item key={frame_value.frame_label} label='Protocol' style={{ textAlign: 'center' }} span={1}>
      <Statistic value={frame_value.frame_label} />
    </Descriptions.Item>
    {frame_value.items.map((v, index) => { return descriptions_item(msb_first, v.item_label + index, v.item_label, v.value); })}
  </>);
}

//
const IrDecordedFrame = (props: Props): JSX.Element => {
  const value: IrFrameValue = ir_frame_value(props.decorded_frame);
  return (<>
    <Descriptions title={'Frame# ' + (1 + props.index)} >
      <Descriptions.Item >
        <Title level={5}>
          {value.items.map(v => { return to_hex_string(props.msb_first, v.value) })}
        </Title >
      </Descriptions.Item>
    </Descriptions>
    <Descriptions
      layout='vertical'
      column={{ xxl: 12, xl: 8, lg: 6, md: 4, sm: 2, xs: 1 }}
      bordered>
      {display_decorded_frame(props.msb_first, value)}
    </Descriptions>
  </>
  )
}

export default IrDecordedFrame;