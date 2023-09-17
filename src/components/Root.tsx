// Copyright (c) 2023 Akihiro Yamamoto.
// Licensed under the MIT License <https://spdx.org/licenses/MIT.html>
// See LICENSE file in the project root for full license information.
//
import React, { useEffect, useState } from 'react';
import { useLocation, Outlet, NavLink } from 'react-router-dom';
import { Layout, Menu } from 'antd';
import 'antd/dist/antd.min.css';
import './root.css';
import init from '../../wasm/pkg/wasm';
import SerialConnectionEffect from './SerialConnectionEffect';

//
const nav_items = [
  { key: 'item 1', label: 'HOME', path: '/' },
  { key: 'item 2', label: 'Infrared Remote Analyzer', path: '/ir-analyzer/' },
  { key: 'item 3', label: 'Settings', path: '/settings/' },
]

//
const Root: React.FC = () => {
  const location = useLocation();
  const [selectedKey, setSelectedKey] = useState<string | undefined>([...nav_items].reverse().find(_item => location.pathname.includes(_item.path))?.key)

  useEffect(() => { init() }, []);
  useEffect(() => {
    setSelectedKey([...nav_items].reverse().find(_item => location.pathname.includes(_item.path))?.key)
  }, [location])
  //
  return (
    <Layout>
      <Layout.Header>
        <Menu
          theme='dark'
          mode='horizontal'
          selectedKeys={selectedKey ? [selectedKey] : undefined}
          items={nav_items.map(item => ({
            key: item.key,
            label: (<NavLink to={item.path}>{item.label}</NavLink>),
          }))}
        />
      </Layout.Header>
      <Layout.Content>
        <SerialConnectionEffect />
        <div className='content'>
          <Outlet />
        </div>
      </Layout.Content>
      <Layout.Footer>vite-react-rust-wasm-project &copy;2023 Akihiro Yamamoto
      </Layout.Footer>
    </Layout >
  );
}

export default Root;