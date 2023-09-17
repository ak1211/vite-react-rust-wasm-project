// Web Serial API 
//
// Copyright (c) 2023 Akihiro Yamamoto.
// Licensed under the MIT License <https://spdx.org/licenses/MIT.html>
// See LICENSE file in the project root for full license information.
//
//
export interface OpenOptions {
	baudRate: number;
	bufferSize?: number;
	dataBits?: 7 | 8;
	flowControl?: 'none' | 'hardware';
	parity?: 'none' | 'even' | 'odd';
	stopBits?: 1 | 2;
};

//
export interface SetSignalsOptions {
	dataTerminalReady?: boolean;
	requestToSend?: boolean;
	break?: boolean;
};

//
export interface Signals {
	clearToSend: boolean;
	dataCarrierDetect: boolean;
	dataSetReady: boolean;
	ringIndicator: boolean;
};

//
export interface SerialPort extends EventTarget {
	readonly readable: ReadableStream;
	readonly writable: WritableStream;
	//
	close(): Promise<undefined>;
	forget(): Promise<undefined>;
	getInfo(): { usbVendorId: number | undefined, usbProductId: number | undefined };
	getSignals(): Promise<Signals>;
	open(options: OpenOptions): Promise<undefined>;
	setSignals(options?: SetSignalsOptions): Promise<undefined>;
};

//
export interface Serial extends EventTarget {
	getPorts(): Promise<SerialPort[]>;
	requestPort(): Promise<SerialPort>;
};

//
declare global {
	interface Navigator {
		readonly serial: Serial;
	}
}

export const available = (): boolean => { return "serial" in navigator; };

export const getSerial = (): Serial | undefined => {
	return available() ? navigator.serial : undefined;
};

