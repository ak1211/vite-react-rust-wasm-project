import React from 'react';
import ReactDOM from 'react-dom/client'
import {
  createBrowserRouter,
  RouterProvider,
} from "react-router-dom";
import { RecoilRoot } from 'recoil';
import Root from './components/Root';
import Home from './components/Home';
import NotFound from './components/NotFound';
import InfraredRemoteAnalyzer from './components/InfraredRemoteAnalyzer.tsx';
import ErrorPage from './error-page';
import Settings from './components/Settings.tsx';
import './index.css';

const router = createBrowserRouter([
  {
    path: "/",
    element: <Root />,
    errorElement: <ErrorPage />,
    children: [
      {
        path: "/",
        element: <Home />
      },
      {
        path: "ir-analyzer",
        element: <InfraredRemoteAnalyzer />
      },
      {
        path: "settings",
        element: <Settings />
      },
      {
        path: "*",
        element: <NotFound />
      },
    ]
  }
], { basename: "/vite-react-rust-wasm-project", });

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <RecoilRoot>
      <RouterProvider router={router} />
    </RecoilRoot>
  </React.StrictMode>,
)
