import { NavLink } from 'react-router-dom';
import React from 'react';
import { Card, Typography } from 'antd';
import ReactEmbedGist from 'react-embed-gist';
import { LoadingOutlined } from '@ant-design/icons';

const { Title, Text } = Typography;

//
const Home: React.FC = () => {
	return (
		<>
			<Title>ホーム</Title>
			<p>
				こんにちは。
			</p>
			<p>
				このウエブアプリケーションは赤外線リモコン信号の解析を行います。<br />
				解析対象のコードがあるなら <NavLink to="/ir-analyzer/">Infrared Remote Analyzer</NavLink> ページに移動して
				その入力フォームに入れてください。
			</p>
			<h2>ハードウェアの準備</h2>
			<p>
				このウエブアプリケーションは<a href="https://ssci.to/6262">ATOM Lite</a>に<a href="https://ssci.to/5699">M5Stack用赤外線送受信ユニット</a>を接続したハードウエアをPCにシリアル接続することで便利に使えるようになっています。
			</p>
			<h2>(ATOM Liteの)ファームウェアの準備</h2>
			<p> <a href="https://docs.m5stack.com/en/quick_start/atom/arduino">このページを参照して</a><br />
				Arduino IDEでATOM Liteのビルドができるように準備してください。
			</p>
			<p>続いてArduino IDEの「ツール」⇒「ライブラリを管理」を選択してライブラリマネージャから"IRremoteESP8266"をインストールしてください。</p>
			<Card>
				<ReactEmbedGist
					gist='ak1211/b87f4dac2cc6ca4c6a9fb9e50501973a'
					wrapperClass="gist__bash"
					titleClass="gist__title"
					loadingFallback={<LoadingOutlined />}
				/>
			</Card>
			<p>このファームウェアのソースコードをArduino IDEを使用してATOM Liteに書き込んでください</p>
			<p>ファームウェアが書き込めたATOM Liteが準備できたら、Arduino IDEは不要です。</p>
			<p><NavLink to="/settings/">Settings</NavLink> ページに移動してWeb Serial APIによるシリアル接続を行ってください。</p>
		</>
	)
}

export default Home