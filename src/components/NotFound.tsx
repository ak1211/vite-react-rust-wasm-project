// Copyright (c) 2023 Akihiro Yamamoto.
// Licensed under the MIT License <https://spdx.org/licenses/MIT.html>
// See LICENSE file in the project root for full license information.
//
import { Button, Result } from 'antd'
import { Link } from 'react-router-dom';

//
const Root: React.FC = () => {
  return (
    <Result status="404" title="404" subTitle='Sorry, the page you visited does not exist.'
      extra={<Button type='primary' ><Link to='/'>Back Home</Link></Button>} />
  );
}

export default Root