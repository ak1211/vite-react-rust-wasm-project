// Copyright (c) 2023 Akihiro Yamamoto.
// Licensed under the MIT License <https://spdx.org/licenses/MIT.html>
// See LICENSE file in the project root for full license information.
//
import { InfraredRemoteControlCode } from './types'
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
  interface T { label: string, value: string };
  if ('Sirc' in ir_code) {
    const list: T[] = [
      { label: 'manufacturer', value: 'ソニー' },
      { label: 'device', value: ir_code.Sirc.device },
      { label: 'command', value: ir_code.Sirc.command },
    ];
    return list.map((v) => { return descriptions_item(v.label, v.label, v.value); });
  } else if ('PanasonicHvac' in ir_code) {
    const list: T[] = [
      { label: 'manufacturer', value: 'パナソニック' },
      { label: 'device', value: 'エアコン' },
      { label: '温度', value: ir_code.PanasonicHvac.temperature.toString() },
      { label: 'モード', value: ir_code.PanasonicHvac.mode },
      { label: '電源', value: ir_code.PanasonicHvac.switch ? "ON" : "OFF" },
      { label: 'swing', value: ir_code.PanasonicHvac.swing },
      { label: '風量', value: ir_code.PanasonicHvac.fan },
      { label: 'チェックサム', value: ir_code.PanasonicHvac.checksum.toString() },
    ];
    return list.map((v) => { return descriptions_item(v.label, v.label, v.value); });
  } else if ('DaikinHvac' in ir_code) {
    const list: T[] = [
      { label: 'manufacturer', value: 'ダイキン' },
      { label: 'device', value: 'エアコン' },
      { label: '温度', value: ir_code.DaikinHvac.temperature.toString() },
      { label: 'モード', value: ir_code.DaikinHvac.mode },
      { label: 'オンタイマー', value: ir_code.DaikinHvac.on_timer.toString() },
      { label: 'オン継続時間', value: ir_code.DaikinHvac.on_timer_duration_hour.toString() },
      { label: 'オフタイマー', value: ir_code.DaikinHvac.off_timer.toString() },
      { label: 'オフ継続時間', value: ir_code.DaikinHvac.off_timer_duration_hour.toString() },
      { label: '電源', value: ir_code.DaikinHvac.switch ? "ON" : "OFF" },
      { label: 'swing', value: ir_code.DaikinHvac.swing },
      { label: '風量', value: ir_code.DaikinHvac.fan },
      { label: 'チェックサム', value: ir_code.DaikinHvac.checksum.toString() },
    ];
    return list.map((v) => { return descriptions_item(v.label, v.label, v.value); });
  } else if ('HitachiHvac' in ir_code) {
    const list: T[] = [
      { label: 'manufacturer', value: '日立' },
      { label: 'device', value: 'エアコン' },
      { label: '温度', value: ir_code.HitachiHvac.temperature.toString() },
      { label: 'モード', value: ir_code.HitachiHvac.mode },
      { label: '電源', value: ir_code.HitachiHvac.switch ? "ON" : "OFF" },
      { label: '風量', value: ir_code.HitachiHvac.fan },
    ];
    return list.map((v) => { return descriptions_item(v.label, v.label, v.value); });
  } else if ('MitsubishiElectricHvac' in ir_code) {
    const list: T[] = [
      { label: 'manufacturer', value: '三菱電機' },
      { label: 'device', value: 'エアコン' },
      { label: '温度', value: ir_code.MitsubishiElectricHvac.temperature.toString() },
      { label: 'モード1', value: ir_code.MitsubishiElectricHvac.mode1 },
      { label: '電源', value: ir_code.MitsubishiElectricHvac.switch ? "ON" : "OFF" },
      { label: 'チェックサム', value: ir_code.MitsubishiElectricHvac.checksum.toString() },
    ];
    return list.map((v) => { return descriptions_item(v.label, v.label, v.value); });
  } else if ('Unknown' in ir_code) {
    const list: T[] = [
      { label: 'manufacturer', value: '不明' },
      { label: 'device', value: '不明' },
    ];
    return list.map((v) => { return descriptions_item(v.label, v.label, v.value); });
  } else {
    throw new Error('unimplemented')
  }
}

export default IrControlCode;