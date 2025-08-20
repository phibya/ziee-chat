import { theme } from 'antd'
import { useEffect } from 'react'

export const useSetBackgroundColor = () => {
  const { token } = theme.useToken()
  useEffect(() => {
    document.documentElement.style.backgroundColor = token.colorBgContainer
  })
}
