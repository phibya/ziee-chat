import {
  TestProxyConnectionRequest,
  TestProxyConnectionResponse,
} from '../types'
import { ApiClient } from './client'

/**
 * Tests a proxy configuration and returns detailed response
 * @param proxySettings The proxy settings to test
 * @returns Promise that resolves to the full test response
 */
export async function testProxyDetailed(
  proxySettings: TestProxyConnectionRequest,
): Promise<TestProxyConnectionResponse> {
  try {
    return await ApiClient.Utils.testProxy(proxySettings)
  } catch (error) {
    console.error('Proxy test failed:', error)
    return {
      success: false,
      message: error instanceof Error ? error.message : 'Proxy test failed',
    }
  }
}
