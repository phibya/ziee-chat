import { Form, Input, InputNumber, Switch } from 'antd'

export function LDAPConfigForm() {
  return (
    <>
      <Form.Item
        name={['config', 'server_url']}
        label="LDAP Server URL"
        rules={[{ required: true, message: 'Please enter LDAP server URL' }]}
        tooltip="LDAP server address (e.g., ldap://ldap.example.com:389 or ldaps://ldap.example.com:636)"
      >
        <Input placeholder="ldap://ldap.example.com:389" />
      </Form.Item>

      <Form.Item
        name={['config', 'bind_dn']}
        label="Bind DN"
        tooltip="Distinguished Name for binding to LDAP (e.g., cn=admin,dc=example,dc=com)"
      >
        <Input placeholder="cn=admin,dc=example,dc=com" />
      </Form.Item>

      <Form.Item
        name={['config', 'bind_password']}
        label="Bind Password"
        tooltip="Password for the bind DN"
      >
        <Input.Password placeholder="Enter bind password" />
      </Form.Item>

      <Form.Item
        name={['config', 'user_search_base']}
        label="User Search Base"
        rules={[{ required: true, message: 'Please enter user search base' }]}
        tooltip="Base DN for user searches (e.g., ou=users,dc=example,dc=com)"
      >
        <Input placeholder="ou=users,dc=example,dc=com" />
      </Form.Item>

      <Form.Item
        name={['config', 'user_search_filter']}
        label="User Search Filter"
        tooltip="LDAP filter for finding users (e.g., (uid={username}) or (mail={username}))"
      >
        <Input placeholder="(uid={username})" />
      </Form.Item>

      <Form.Item
        name={['config', 'group_search_base']}
        label="Group Search Base"
        tooltip="Base DN for group searches (optional)"
      >
        <Input placeholder="ou=groups,dc=example,dc=com" />
      </Form.Item>

      <Form.Item
        name={['config', 'group_search_filter']}
        label="Group Search Filter"
        tooltip="LDAP filter for finding groups (optional)"
      >
        <Input placeholder="(member={userdn})" />
      </Form.Item>

      <Form.Item
        name={['config', 'timeout']}
        label="Connection Timeout (seconds)"
        tooltip="Timeout for LDAP connections"
      >
        <InputNumber min={1} placeholder="10" className="w-full" />
      </Form.Item>

      <Form.Item
        name={['config', 'use_tls']}
        label="Use TLS/SSL"
        valuePropName="checked"
        tooltip="Enable TLS/SSL for secure connections"
      >
        <Switch />
      </Form.Item>

      <Form.Item
        name={['config', 'skip_verify']}
        label="Skip TLS Verification"
        valuePropName="checked"
        tooltip="Skip TLS certificate verification (not recommended for production)"
      >
        <Switch />
      </Form.Item>

      {/* Attribute Mapping */}
      <Form.Item
        label="Attribute Mapping"
        tooltip="Map LDAP attributes to user fields"
      >
        <div className="text-xs text-gray-500">
          Configure how LDAP attributes map to local user fields
        </div>
      </Form.Item>

      <Form.Item
        name={['mapping_rules', 'username']}
        label="Username Attribute"
        tooltip="LDAP attribute to use as username (e.g., uid, sAMAccountName)"
      >
        <Input placeholder="uid" />
      </Form.Item>

      <Form.Item
        name={['mapping_rules', 'email']}
        label="Email Attribute"
        tooltip="LDAP attribute containing email address"
      >
        <Input placeholder="mail" />
      </Form.Item>

      <Form.Item
        name={['mapping_rules', 'display_name']}
        label="Display Name Attribute"
        tooltip="LDAP attribute for display name"
      >
        <Input placeholder="cn" />
      </Form.Item>

      <Form.Item
        name={['mapping_rules', 'groups']}
        label="Groups Attribute"
        tooltip="LDAP attribute containing group memberships"
      >
        <Input placeholder="memberOf" />
      </Form.Item>
    </>
  )
}
