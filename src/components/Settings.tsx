// Copyright (c) 2023 Akihiro Yamamoto.
// Licensed under the MIT License <https://spdx.org/licenses/MIT.html>
// See LICENSE file in the project root for full license information.
//
import { useRecoilValue } from 'recoil';
import { Divider, Button, Space, Switch, Result, Typography, List, Card } from 'antd'
import * as SC from './SerialConnectionEffect';
import { available as wsa_available } from '../webserialapi';
import { CheckOutlined, CloseOutlined } from '@ant-design/icons';

const { Title, Paragraph } = Typography;

//
const Settings: React.FC = () => {
	const serialport = useRecoilValue(SC.serialPortState);
	const rxText = useRecoilValue(SC.rxTextState);
	const rxJsonData = useRecoilValue(SC.rxJsonDataState);

	return (
		<>
			<Title>シリアルポート設定</Title>
			{wsa_available() ? (
				<Result
					status="success"
					title="Web Serial APIが使用できます。"
					extra={
						<Paragraph>M5Atom Light をコンピューターに接続して、シリアルポートを開いてください。</Paragraph>
					}
				/>) : (
				<Result
					status="warning"
					title="現在使用中のブラウザではWebSerialAPIを利用できないためにシリアルポートに接続できません。"
					extra={
						<>
							<Paragraph>Web Serial APIに対応するブラウザを使用してください。</Paragraph>
							<Button type='primary' href="https://developer.mozilla.org/ja/docs/Web/API/Web_Serial_API">
								MDNの Web Serial API ページに移動する
							</Button>
						</>
					}
				/>)}
			<Space size='large'>
				シリアルポート接続
				<Switch
					title={"serial port connection"}
					checkedChildren={<CheckOutlined />}
					unCheckedChildren={<CloseOutlined />}
					disabled={!wsa_available()}
					onChange={(checked) => {
						window.dispatchEvent(
							new CustomEvent(checked ? SC.RequestMessages.Open : SC.RequestMessages.Close)
						);
					}}
					checked={serialport !== undefined}
				/>
				<Button type='default' shape='round' onClick={() => {
					window.dispatchEvent(
						new CustomEvent(SC.RequestMessages.SoftwareReset)
					)
				}}>ソフトウエアリセットを発行する</Button>
			</Space>
			<Divider></Divider >
			<Card title='Serial Monitor'>
				<pre>
					{rxText}
				</pre>
			</Card>
			<List<SC.RxJsonSchema>
				header={<span>Received Infrared Remote Code</span>}
				bordered
				dataSource={rxJsonData}
				renderItem={(item) =>
					<List.Item key={item.timestamp}>
						<pre>{JSON.stringify(item)}</pre>
					</List.Item>
				}
			/>
		</>
	)
}

export default Settings