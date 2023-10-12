// Copyright (c) 2023 Akihiro Yamamoto.
// Licensed under the MIT License <https://spdx.org/licenses/MIT.html>
// See LICENSE file in the project root for full license information.
//
import React, { useEffect, useCallback, useRef } from 'react';
import { atom, useRecoilState, useSetRecoilState } from 'recoil';
import { available as wsa_available, Serial, SerialPort } from '../webserialapi';

//
const TIMEOUT_MILLISECONDS = 60;

//
export enum RequestMessages {
	Open = 'REQUEST_OPEN_SERIAL_PORT',
	Close = 'REQUEST_CLOSE_SERIAL_PORT',
	SoftwareReset = 'REQUEST_SOFTWARE_RESET_SERIAL_PORT',
};

//
export interface RxJsonSchema {
	timestamp: number,
	buffer_full: boolean,
	library: string,
	code: string,
	rawDataSize: number,
	rawData: number[],
	description: string,
};

//
export const rxJsonDataState = atom<RxJsonSchema[]>({
	key: 'rxJsonDataState',
	default: [],
});

//
export const rxTextState = atom<string>({
	key: 'rxTextState',
	default: '',
});

//
type SerialPortReader = ReadableStreamDefaultReader;
export const serialPortState = atom<{ port: SerialPort, reader: SerialPortReader } | undefined>({
	key: 'serialPortState',
	default: undefined,
});

//
const openSerialPort = (serial: Serial): Promise<SerialPort> => {
	// ユーザーにシリアルポートを選んでもらう
	return serial
		.requestPort()
		.then(
			async (port) => {
				await port.open({ baudRate: 115200, bufferSize: 8192 });
				return port;
			}
		);
}

//
const readSerialPort = (reader: SerialPortReader): Promise<string | undefined> => {
	return reader
		.read()
		.then(
			({ value, done }) => {
				if (done) {
					console.log("Canceled\n");
					return undefined;
				} else {
					return new TextDecoder().decode(value);
				}
			}
		)
}

//
const SerialConnectionEffect: React.FC = () => {
	//
	const timeoutRef = useRef<number | undefined>(undefined);
	//
	const [serialPort, setSerialPort] = useRecoilState(serialPortState);
	//
	const handleConnectEvent = useCallback(() => { console.log("serial device connected") }, []);
	//
	const handleDisconnectEvent = useCallback(() => {
		setSerialPort(undefined);
		console.log("serial device disconnected")
	}, [setSerialPort]);
	//
	const handleOpenSerialPort = useCallback(
		async () => {
			await openSerialPort(navigator.serial)
				.then(
					(port) => {
						const reader = port.readable.getReader();
						setSerialPort({ port: port, reader: reader });
					},
					(error) => {
						console.log(error);
						setSerialPort(undefined);
					}
				)
		}, [setSerialPort]);
	//
	const handleCloseSerialPort = useCallback(
		async () => {
			clearTimeout(timeoutRef.current);
			timeoutRef.current = undefined;
			setSerialPort((prev) => {
				prev && (async () => {
					prev.reader.releaseLock();
					await prev.port.close();
					return undefined;
				})();
				return undefined;
			});
		}, [setSerialPort]);
	//
	const handleSoftwareResetSerialPort = useCallback(
		async () => {
			if (serialPort?.port) {
				// M5Atomにシリアル通信でリセットを行う。
				// RTSをHiにする
				await serialPort.port.setSignals({
					dataTerminalReady: false,
					requestToSend: true,
				});
				//
				await new Promise(resolve => setTimeout(resolve, 125));
				// 一定時間後に
				// RTSをLoにする
				await serialPort.port.setSignals({
					requestToSend: false,
				});
				// 受信文をクリアする
				setRxText('');
			}
		}, [serialPort]);
	//
	const [rxText, setRxText] = useRecoilState(rxTextState);
	const setRxJsonData = useSetRecoilState(rxJsonDataState);
	//
	const handleReceiveSerialPort = useCallback(
		async () => {
			if (serialPort) {
				return await readSerialPort(serialPort.reader)
					.then(
						(text) => {
							if (text !== undefined) {
								setRxText((prev) => prev + text);
							}
							return true;
						},
						(error) => {
							console.log("Error: Read " + error + "\n");
							return false;
						}
					)
			} else {
				return false;
			}
		}, [serialPort?.reader]);
	//
	useEffect(function callToTimeoutLoop() {
		timeoutRef.current = setTimeout(async () => {
			if (await handleReceiveSerialPort()) {
				callToTimeoutLoop();
			}
		}, TIMEOUT_MILLISECONDS);
		return () => {
			clearTimeout(timeoutRef.current);
			timeoutRef.current = undefined;
		}
	}, [handleReceiveSerialPort]);
	//
	useEffect(() => {
		try {
			const lines = rxText.split(/\r?\n/);
			const last: string | undefined = lines.slice(-2)[0];
			if (last) {
				const rx_json: RxJsonSchema = JSON.parse(last);
				setRxJsonData((prev: RxJsonSchema[]) => [...prev, rx_json]);
				// JSON読み取りに成功したならバッファを消去する
				setRxText('');
			}
		} catch (e) { }
	}, [rxText]);
	//
	useEffect(() => {
		//
		window.addEventListener(RequestMessages.Open, handleOpenSerialPort, { passive: true });
		window.addEventListener(RequestMessages.Close, handleCloseSerialPort, { passive: true });
		window.addEventListener(RequestMessages.SoftwareReset, handleSoftwareResetSerialPort, { passive: true });
		if (wsa_available()) {
			navigator.serial.addEventListener("connect", handleConnectEvent, { passive: true });
			navigator.serial.addEventListener("disconnect", handleDisconnectEvent, { passive: true });
		}
		return () => {
			window.removeEventListener(RequestMessages.SoftwareReset, handleSoftwareResetSerialPort);
			window.removeEventListener(RequestMessages.Close, handleCloseSerialPort);
			window.removeEventListener(RequestMessages.Open, handleOpenSerialPort);
			if (wsa_available()) {
				navigator.serial.removeEventListener("connect", handleConnectEvent);
				navigator.serial.removeEventListener("disconnect", handleDisconnectEvent);
			}
		};
	}, [handleConnectEvent, handleDisconnectEvent, handleOpenSerialPort, handleCloseSerialPort, handleSoftwareResetSerialPort]);
	//
	return <></>;
}

export default SerialConnectionEffect 