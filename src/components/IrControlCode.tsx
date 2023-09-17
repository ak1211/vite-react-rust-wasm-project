// Copyright (c) 2023 Akihiro Yamamoto.
// Licensed under the MIT License <https://spdx.org/licenses/MIT.html>
// See LICENSE file in the project root for full license information.
//
import { InfraredRemoteControlCode } from '../../wasm/pkg/wasm'
import { Space, Empty, Descriptions, Card, Statistic, Typography } from 'antd'
import 'antd/dist/antd.min.css'

const { Title } = Typography

type Props = {
  ir_control_codes: InfraredRemoteControlCode[],
}

//
const IrControlCode = (props: Props): JSX.Element => {
  const empty = <Empty image={Empty.PRESENTED_IMAGE_SIMPLE} />;
  return (
    <Card size='small' title={<Title level={4}>赤外線リモコンコード</Title>}>
      {props.ir_control_codes.length == 0 ? empty : props.ir_control_codes.map((v, idx) => display_remocon_code(idx, v))}
    </Card>
  )
}

//
const display_remocon_code = (index: number, ir_code: InfraredRemoteControlCode): JSX.Element => {
  const title = 'Frame# ' + (1 + index);
  return <Space key={title} direction='vertical'>
    <Descriptions title={title}></Descriptions>
    <Descriptions bordered>{display_ir_code(ir_code)}</Descriptions>
  </Space>;
}

//
const descriptions_item = (key: any, label: string, value: string) => {
  return (<Descriptions.Item key={key} label={label} style={{ textAlign: 'center' }} span={1}>
    <Statistic value={value} />
  </Descriptions.Item >);
};

//
const display_ir_code = (ir_code: InfraredRemoteControlCode): JSX.Element[] => {
  var list: JSX.Element[] = [];
  ir_code.forEach((value, key) => {
    list.push(descriptions_item("", key, value))
  })
  return list;
}

export default IrControlCode;