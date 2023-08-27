// Copyright (c) 2023 Akihiro Yamamoto.
// Licensed under the MIT License <https://spdx.org/licenses/MIT.html>
// See LICENSE file in the project root for full license information.
//
import { InfraredRemoteDemodulatedFrame } from './types'
import { Typography, Descriptions } from 'antd'
import 'antd/dist/antd.min.css'

const { Text } = Typography

type Props = {
  msb_first: Boolean,
  ir_demodulated_frame: InfraredRemoteDemodulatedFrame,
  index: number,
}

//
const IrBitStream = (props: Props): JSX.Element => {
  const bitstream: Uint8Array = ((demodulated_frame: InfraredRemoteDemodulatedFrame): Uint8Array => {
    if ('Aeha' in demodulated_frame) {
      return demodulated_frame.Aeha;
    } else if ('Nec' in demodulated_frame) {
      return demodulated_frame.Nec;
    } else if ('Sirc' in demodulated_frame) {
      return demodulated_frame.Sirc;
    } else if ('Unknown' in demodulated_frame) {
      return new Uint8Array();
    } else {
      console.log(demodulated_frame);
      throw new Error('unimplemented')
    }
  })(props.ir_demodulated_frame);

  return display_bitstream(bitstream, props.index);
}

const display_bitstream = (bitstream: Uint8Array, index: number): JSX.Element => {
  return (
    <Descriptions title={'Frame# ' + (1 + index)} >
      <Descriptions.Item key={bitstream.join('')} label={'Bitstream ' + bitstream.length + ' bits(with stop bit)'}>
        <Text>{bitstream.join('')}</Text>
      </Descriptions.Item>
    </Descriptions>
  );
}

export default IrBitStream;
