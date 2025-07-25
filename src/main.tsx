import { App as AntdApp } from 'antd'
// import React from "react";
import ReactDOM from 'react-dom/client'
import App from './App'
import './styles/global.css'
import './starup/desktop'

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  // <React.StrictMode>
  <AntdApp>
    <App />
  </AntdApp>,
  // </React.StrictMode>,
)
