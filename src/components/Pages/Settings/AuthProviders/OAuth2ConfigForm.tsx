import { Form, Input, Switch, Typography } from 'antd'

const { TextArea } = Input
const { Text } = Typography

export function OAuth2ConfigForm() {
  return (
    <>
      <Form.Item
        name={['config', 'client_id']}
        label="Client ID"
        rules={[{ required: true, message: 'Please enter client ID' }]}
        tooltip="The OAuth2 client ID provided by the identity provider"
      >
        <Input placeholder="Enter client ID" />
      </Form.Item>

      <Form.Item
        name={['config', 'client_secret']}
        label="Client Secret"
        rules={[{ required: true, message: 'Please enter client secret' }]}
        tooltip="The OAuth2 client secret provided by the identity provider"
      >
        <Input.Password placeholder="Enter client secret" />
      </Form.Item>

      <Form.Item
        name={['config', 'authorization_url']}
        label="Authorization URL"
        rules={[
          { required: true, message: 'Please enter authorization URL' },
          { type: 'url', message: 'Please enter a valid URL' },
        ]}
        tooltip="The OAuth2 authorization endpoint URL"
      >
        <Input placeholder="https://provider.com/oauth2/authorize" />
      </Form.Item>

      <Form.Item
        name={['config', 'token_url']}
        label="Token URL"
        rules={[
          { required: true, message: 'Please enter token URL' },
          { type: 'url', message: 'Please enter a valid URL' },
        ]}
        tooltip="The OAuth2 token endpoint URL"
      >
        <Input placeholder="https://provider.com/oauth2/token" />
      </Form.Item>

      <Form.Item
        name={['config', 'userinfo_url']}
        label="User Info URL"
        rules={[{ type: 'url', message: 'Please enter a valid URL' }]}
        tooltip="The OAuth2/OIDC user info endpoint URL (optional for OAuth2, required for OIDC)"
      >
        <Input placeholder="https://provider.com/oauth2/userinfo" />
      </Form.Item>

      <Form.Item
        name={['config', 'redirect_uri']}
        label="Redirect URI"
        tooltip="The callback URL to be configured in your OAuth provider. This is typically: https://yourdomain.com/auth/callback"
      >
        <Input placeholder="https://yourdomain.com/auth/callback" disabled />
      </Form.Item>

      <Form.Item
        name={['config', 'scopes']}
        label="Scopes"
        rules={[{ required: true, message: 'Please enter scopes' }]}
        tooltip="OAuth2 scopes to request, separated by spaces (e.g., 'openid profile email')"
      >
        <Input placeholder="openid profile email" />
      </Form.Item>

      <Form.Item
        name={['config', 'pkce_enabled']}
        label="PKCE Enabled"
        valuePropName="checked"
        tooltip="Enable Proof Key for Code Exchange (PKCE) for enhanced security"
      >
        <Switch />
      </Form.Item>

      <Form.Item
        name={['config', 'additional_params']}
        label="Additional Parameters"
        tooltip="Additional query parameters to include in authorization requests (JSON format)"
        rules={[
          {
            validator: (_, value) => {
              if (!value) return Promise.resolve()
              try {
                JSON.parse(value)
                return Promise.resolve()
              } catch {
                return Promise.reject('Invalid JSON format')
              }
            },
          },
        ]}
      >
        <TextArea
          rows={3}
          placeholder='{"prompt": "consent", "access_type": "offline"}'
        />
      </Form.Item>

      <Form.Item
        label="Attribute Mapping"
        tooltip="Configure how user attributes from the provider map to local user fields"
      >
        <Text type="secondary" className="text-xs">
          Configure attribute mapping rules below
        </Text>
      </Form.Item>

      <Form.Item
        name={['mapping_rules', 'username']}
        label="Username Attribute"
        tooltip="The attribute from the provider to use as username (e.g., 'preferred_username', 'email', 'sub')"
      >
        <Input placeholder="preferred_username" />
      </Form.Item>

      <Form.Item
        name={['mapping_rules', 'email']}
        label="Email Attribute"
        tooltip="The attribute from the provider to use as email"
      >
        <Input placeholder="email" />
      </Form.Item>

      <Form.Item
        name={['mapping_rules', 'display_name']}
        label="Display Name Attribute"
        tooltip="The attribute from the provider to use as display name"
      >
        <Input placeholder="name" />
      </Form.Item>

      <Form.Item
        name={['mapping_rules', 'groups']}
        label="Groups Attribute"
        tooltip="The attribute from the provider containing user groups"
      >
        <Input placeholder="groups" />
      </Form.Item>
    </>
  )
}
