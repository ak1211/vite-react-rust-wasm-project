// Copyright (c) 2023 Akihiro Yamamoto.
// Licensed under the MIT License <https://spdx.org/licenses/MIT.html>
// See LICENSE file in the project root for full license information.
//
import { MarkAndSpaceMicros } from '../wasm/pkg/wasm';
import { Line, Datum } from '@ant-design/charts';
import { Card, Divider, Typography, Table } from 'antd';
import 'antd/dist/antd.min.css';
import './App.css';

const { Title } = Typography;

type Props = {
  ir_mark_and_spaces: MarkAndSpaceMicros[],
}

//
const IrSignal = (props: Props): JSX.Element => {
  //
  const config = {
    data: conv_ir_control_signal(props.ir_mark_and_spaces),
    height: 200,
    xField: 'time',
    yField: 'bit',
    stepType: 'vh',
    animation: false,
    xAxis: {
      type: 'time',
      tickInterval: 1,
      label: {
        formatter: (_i: any, _j: any, index: number) => {
          return index
        }
      },
    },
    yAxis: {
      tickInterval: 1,
    },
    tooltip: {
      title: '赤外線リモコン信号',
      formatter: (datum: Datum) => {
        const micros = datum.time as number;
        const millis = Math.floor(micros / 1000.0);
        const label: string = micros + 'μs' + '(' + millis + 'ms' + ')';
        const description: string = datum.bit === 0 ? 'Lo' : 'Hi';
        return { name: label, value: description };
      },
    },
    slider: {
      start: 0.0,
      end: 1.0,
    },
  };

  const columns = [
    {
      title: 'Seqence Number',
      dataIndex: 'seq_num',
      key: 'seq_num',
    },
    {
      title: 'Start Time (μs)',
      dataIndex: 'start_time',
      key: 'start_time',
    },
    {
      title: 'Duration (μs)',
      dataIndex: 'duration',
      key: 'duration',
    },
    {
      title: 'Kinds',
      dataIndex: 'kinds',
      key: 'kinds',
    },
  ];

  return (
    <Card size='small' title={<Title level={4}>受信信号</Title>}>
      <Line {...config} />
      <Divider>マークアンドスペース</Divider>
      <Table
        dataSource={convert_for_list(props.ir_mark_and_spaces)}
        columns={columns}
        scroll={{ y: 240 }}
      />
    </Card>
  )
}

interface DatumForList { key: number, seq_num: number, start_time: number, kinds: string, duration: number }
const convert_for_list = (input: MarkAndSpaceMicros[]): DatumForList[] => {
  var start: number = 0;
  var output: DatumForList[] = [];
  var sequencenumber: number = 1;
  input.forEach(item => {
    output.push({ key: sequencenumber, seq_num: sequencenumber, start_time: start, kinds: 'Mark', duration: item.mark });
    start += item.mark;
    sequencenumber += 1;
    //
    output.push({ key: sequencenumber, seq_num: sequencenumber, start_time: start, kinds: 'Space', duration: item.space });
    start += item.space;
    sequencenumber += 1;
  })
  return output
}

//
interface DatumForGraph { time: number, bit: number }
const conv_ir_control_signal = (input: MarkAndSpaceMicros[]): Array<DatumForGraph> => {
  var output: DatumForGraph[] = [];
  if (input.length >= 1) {
    var sum = 0;
    output.push({ time: sum, bit: 0 });
    input.forEach(item => {
      sum += item.mark;
      output.push({ time: sum, bit: 1 });
      //
      sum += item.space;
      output.push({ time: sum, bit: 0 });
    })
  }
  return output;
}

export default IrSignal;