import { Form, Input, Switch } from 'antd'

const { TextArea } = Input

export function SAMLConfigForm() {
  return (
    <>
      <Form.Item
        name={['config', 'entity_id']}
        label="Entity ID"
        rules={[{ required: true, message: 'Please enter entity ID' }]}
        tooltip="The unique identifier for your service provider (SP)"
      >
        <Input placeholder="https://yourdomain.com/saml/metadata" />
      </Form.Item>

      <Form.Item
        name={['config', 'sso_url']}
        label="SSO URL"
        rules={[
          { required: true, message: 'Please enter SSO URL' },
          { type: 'url', message: 'Please enter a valid URL' },
        ]}
        tooltip="The Identity Provider's Single Sign-On URL"
      >
        <Input placeholder="https://idp.example.com/saml2/sso" />
      </Form.Item>

      <Form.Item
        name={['config', 'slo_url']}
        label="SLO URL"
        rules={[{ type: 'url', message: 'Please enter a valid URL' }]}
        tooltip="The Identity Provider's Single Logout URL (optional)"
      >
        <Input placeholder="https://idp.example.com/saml2/slo" />
      </Form.Item>

      <Form.Item
        name={['config', 'idp_cert']}
        label="IdP Certificate"
        rules={[{ required: true, message: 'Please enter IdP certificate' }]}
        tooltip="The Identity Provider's X.509 certificate for signature verification"
      >
        <TextArea
          rows={6}
          placeholder="-----BEGIN CERTIFICATE-----&#10;...&#10;-----END CERTIFICATE-----"
        />
      </Form.Item>

      <Form.Item
        name={['config', 'sp_cert']}
        label="SP Certificate"
        tooltip="Your Service Provider's X.509 certificate (optional, required for encryption)"
      >
        <TextArea
          rows={6}
          placeholder="-----BEGIN CERTIFICATE-----&#10;...&#10;-----END CERTIFICATE-----"
        />
      </Form.Item>

      <Form.Item
        name={['config', 'sp_private_key']}
        label="SP Private Key"
        tooltip="Your Service Provider's private key (optional, required for encryption)"
      >
        <TextArea
          rows={6}
          placeholder="-----BEGIN PRIVATE KEY-----&#10;...&#10;-----END PRIVATE KEY-----"
        />
      </Form.Item>

      <Form.Item
        name={['config', 'name_id_format']}
        label="NameID Format"
        tooltip="The SAML NameID format to use"
      >
        <Input placeholder="urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress" />
      </Form.Item>

      <Form.Item
        name={['config', 'want_assertions_signed']}
        label="Require Signed Assertions"
        valuePropName="checked"
        tooltip="Require that SAML assertions are signed"
      >
        <Switch />
      </Form.Item>

      <Form.Item
        name={['config', 'want_messages_signed']}
        label="Require Signed Messages"
        valuePropName="checked"
        tooltip="Require that SAML messages are signed"
      >
        <Switch />
      </Form.Item>

      <Form.Item
        name={['config', 'authn_context']}
        label="Authentication Context"
        tooltip="The authentication context class to request"
      >
        <Input placeholder="urn:oasis:names:tc:SAML:2.0:ac:classes:PasswordProtectedTransport" />
      </Form.Item>

      {/* Attribute Mapping */}
      <Form.Item
        label="Attribute Mapping"
        tooltip="Map SAML attributes to user fields"
      >
        <div className="text-xs text-gray-500">
          Configure how SAML attributes map to local user fields
        </div>
      </Form.Item>

      <Form.Item
        name={['mapping_rules', 'username']}
        label="Username Attribute"
        tooltip="SAML attribute to use as username (e.g., uid, username)"
      >
        <Input placeholder="uid" />
      </Form.Item>

      <Form.Item
        name={['mapping_rules', 'email']}
        label="Email Attribute"
        tooltip="SAML attribute containing email address"
      >
        <Input placeholder="email" />
      </Form.Item>

      <Form.Item
        name={['mapping_rules', 'display_name']}
        label="Display Name Attribute"
        tooltip="SAML attribute for display name"
      >
        <Input placeholder="displayName" />
      </Form.Item>

      <Form.Item
        name={['mapping_rules', 'groups']}
        label="Groups Attribute"
        tooltip="SAML attribute containing group memberships"
      >
        <Input placeholder="memberOf" />
      </Form.Item>
    </>
  )
}
