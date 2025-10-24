import {
  App,
  Button,
  Card,
  Collapse,
  Flex,
  Form,
  Input,
  InputNumber,
  Select,
  Space,
  Switch,
  Typography,
} from 'antd'
import { useEffect, useState } from 'react'
import { Drawer } from '../../../common/Drawer.tsx'
import {
  createAuthProvider,
  getAuthProvider,
  updateAuthProvider,
} from '../../../../store'
import type { AuthProvider } from '../../../../types'
import { OAuth2ConfigForm } from './OAuth2ConfigForm.tsx'
import { LDAPConfigForm } from './LDAPConfigForm.tsx'
import { SAMLConfigForm } from './SAMLConfigForm.tsx'

const { Text } = Typography

interface ProviderTemplate {
  name: string
  description: string
  provider_type: 'oauth2' | 'oidc' | 'ldap' | 'saml'
  config: Record<string, unknown>
  mapping_rules: {
    username: string
    email: string
    display_name: string
    groups?: string
  }
}

const PROVIDER_TEMPLATES: ProviderTemplate[] = [
  // OAuth2 Templates
  {
    name: 'Google OAuth 2.0',
    description: 'Configure Google Sign-In',
    provider_type: 'oauth2',
    config: {
      client_id: 'YOUR_CLIENT_ID.apps.googleusercontent.com',
      authorization_url: 'https://accounts.google.com/o/oauth2/v2/auth',
      token_url: 'https://oauth2.googleapis.com/token',
      userinfo_url: 'https://www.googleapis.com/oauth2/v2/userinfo',
      scopes: 'openid profile email',
      pkce_enabled: true,
    },
    mapping_rules: {
      username: 'email',
      email: 'email',
      display_name: 'name',
    },
  },
  {
    name: 'Microsoft Azure AD',
    description: 'Configure Microsoft/Azure Active Directory',
    provider_type: 'oidc',
    config: {
      client_id: 'YOUR_APPLICATION_CLIENT_ID',
      authorization_url:
        'https://login.microsoftonline.com/common/oauth2/v2.0/authorize',
      token_url: 'https://login.microsoftonline.com/common/oauth2/v2.0/token',
      userinfo_url: 'https://graph.microsoft.com/v1.0/me',
      scopes: 'openid profile email',
      pkce_enabled: true,
    },
    mapping_rules: {
      username: 'userPrincipalName',
      email: 'mail',
      display_name: 'displayName',
    },
  },
  {
    name: 'GitHub OAuth',
    description: 'Configure GitHub OAuth App',
    provider_type: 'oauth2',
    config: {
      client_id: 'YOUR_GITHUB_CLIENT_ID',
      authorization_url: 'https://github.com/login/oauth/authorize',
      token_url: 'https://github.com/login/oauth/access_token',
      userinfo_url: 'https://api.github.com/user',
      scopes: 'read:user user:email',
      pkce_enabled: false,
    },
    mapping_rules: {
      username: 'login',
      email: 'email',
      display_name: 'name',
    },
  },
  // LDAP Templates
  {
    name: 'Active Directory LDAP',
    description: 'Configure Microsoft Active Directory LDAP',
    provider_type: 'ldap',
    config: {
      server_url: 'ldap://dc.example.com:389',
      bind_dn: 'CN=Service Account,CN=Users,DC=example,DC=com',
      user_search_base: 'CN=Users,DC=example,DC=com',
      user_search_filter: '(sAMAccountName={username})',
      group_search_base: 'CN=Groups,DC=example,DC=com',
      group_search_filter: '(member={userdn})',
      timeout: 10,
      use_tls: true,
      skip_verify: false,
    },
    mapping_rules: {
      username: 'sAMAccountName',
      email: 'mail',
      display_name: 'displayName',
      groups: 'memberOf',
    },
  },
  {
    name: 'OpenLDAP',
    description: 'Configure OpenLDAP server',
    provider_type: 'ldap',
    config: {
      server_url: 'ldap://ldap.example.com:389',
      bind_dn: 'cn=admin,dc=example,dc=com',
      user_search_base: 'ou=users,dc=example,dc=com',
      user_search_filter: '(uid={username})',
      group_search_base: 'ou=groups,dc=example,dc=com',
      group_search_filter: '(memberUid={username})',
      timeout: 10,
      use_tls: false,
      skip_verify: false,
    },
    mapping_rules: {
      username: 'uid',
      email: 'mail',
      display_name: 'cn',
      groups: 'memberOf',
    },
  },
  // SAML Templates
  {
    name: 'Okta SAML',
    description: 'Configure Okta SAML integration',
    provider_type: 'saml',
    config: {
      entity_id: 'https://yourdomain.com/saml/metadata',
      sso_url: 'https://yourorg.okta.com/app/yourapp/sso/saml',
      name_id_format: 'urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress',
      want_assertions_signed: true,
      want_messages_signed: false,
    },
    mapping_rules: {
      username: 'uid',
      email: 'email',
      display_name: 'displayName',
      groups: 'groups',
    },
  },
  {
    name: 'Azure AD SAML',
    description: 'Configure Azure Active Directory SAML',
    provider_type: 'saml',
    config: {
      entity_id: 'https://yourdomain.com/saml/metadata',
      sso_url: 'https://login.microsoftonline.com/your-tenant-id/saml2',
      name_id_format: 'urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress',
      want_assertions_signed: true,
      want_messages_signed: true,
    },
    mapping_rules: {
      username: 'http://schemas.xmlsoap.org/ws/2005/05/identity/claims/name',
      email:
        'http://schemas.xmlsoap.org/ws/2005/05/identity/claims/emailaddress',
      display_name:
        'http://schemas.xmlsoap.org/ws/2005/05/identity/claims/givenname',
      groups: 'http://schemas.microsoft.com/ws/2008/06/identity/claims/groups',
    },
  },
]

interface AuthProviderModalProps {
  open: boolean
  onClose: () => void
  onSuccess: () => void
  provider?: AuthProvider | null
}

export function AuthProviderModal({
  open,
  onClose,
  onSuccess,
  provider,
}: AuthProviderModalProps) {
  const { message } = App.useApp()
  const [form] = Form.useForm()
  const [loading, setLoading] = useState(false)

  const isEditMode = !!provider

  // Load provider data when editing
  useEffect(() => {
    if (open && provider) {
      // Load full provider data with config
      getAuthProvider(provider.id)
        .then(fullProvider => {
          if (fullProvider) {
            form.setFieldsValue({
              name: fullProvider.name,
              provider_type: fullProvider.provider_type,
              enabled: fullProvider.enabled,
              priority: fullProvider.priority,
              config: fullProvider.config || {},
              mapping_rules: fullProvider.mapping_rules || {},
            })
          }
        })
        .catch(error => {
          console.error('Failed to load provider:', error)
          message.error('Failed to load provider details')
        })
    } else if (open && !provider) {
      // Reset form for create mode
      form.resetFields()

      // Set default values based on provider type
      const redirectUri = `${window.location.origin}/auth/callback`
      form.setFieldsValue({
        enabled: true,
        priority: 0,
        provider_type: 'oauth2',
        config: {
          redirect_uri: redirectUri,
          pkce_enabled: true,
          scopes: 'openid profile email',
        },
        mapping_rules: {
          username: 'preferred_username',
          email: 'email',
          display_name: 'name',
        },
      })
    }
  }, [open, provider, form, message])

  const handleSubmit = async () => {
    try {
      setLoading(true)
      const values = await form.validateFields()

      // Prepare the data
      const data: any = {
        name: values.name,
        enabled: values.enabled,
        priority: values.priority || 0,
        config: values.config || {},
        mapping_rules: values.mapping_rules || {},
      }

      // Parse additional_params if present
      if (values.config?.additional_params) {
        try {
          data.config.additional_params = JSON.parse(
            values.config.additional_params,
          )
        } catch (error) {
          message.error('Invalid JSON in additional parameters')
          setLoading(false)
          return
        }
      }

      if (isEditMode && provider) {
        // Update existing provider
        await updateAuthProvider(provider.id, data)
        message.success('Provider updated successfully')
      } else {
        // Create new provider
        data.provider_type = values.provider_type
        await createAuthProvider(data)
        message.success('Provider created successfully')
      }

      form.resetFields()
      onSuccess()
      onClose()
    } catch (error) {
      console.error('Failed to save provider:', error)
    } finally {
      setLoading(false)
    }
  }

  const handleClose = () => {
    form.resetFields()
    onClose()
  }

  const applyTemplate = (template: ProviderTemplate) => {
    // Set redirect URI for OAuth2/OIDC templates
    const config = { ...template.config }
    if (
      template.provider_type === 'oauth2' ||
      template.provider_type === 'oidc'
    ) {
      config.redirect_uri = `${window.location.origin}/auth/callback`
    }

    form.setFieldsValue({
      provider_type: template.provider_type,
      config,
      mapping_rules: template.mapping_rules,
    })
    message.success(`Applied ${template.name} template`)
  }

  // Render appropriate config form based on provider type
  const renderConfigForm = (providerType: string) => {
    switch (providerType) {
      case 'oauth2':
      case 'oidc':
        return <OAuth2ConfigForm />
      case 'ldap':
        return <LDAPConfigForm />
      case 'saml':
        return <SAMLConfigForm />
      default:
        return null
    }
  }

  return (
    <Drawer
      title={isEditMode ? 'Edit Auth Provider' : 'Create Auth Provider'}
      open={open}
      onClose={handleClose}
      width={700}
      maskClosable={false}
      footer={
        <Flex className="gap-2 justify-end">
          <Button onClick={handleClose}>Cancel</Button>
          <Button type="primary" onClick={handleSubmit} loading={loading}>
            {isEditMode ? 'Update' : 'Create'}
          </Button>
        </Flex>
      }
    >
      <Form form={form} layout="vertical">
        {/* Common Provider Templates */}
        {!isEditMode && (
          <Collapse
            className="!mb-4"
            items={[
              {
                key: 'templates',
                label: 'Common Provider Templates',
                classNames: {
                  body: '!p-3',
                },
                children: (
                  <div className="w-full flex flex-col gap-3">
                    {PROVIDER_TEMPLATES.map(template => (
                      <Card key={template.name} className="w-full">
                        <Flex justify="space-between" align="center">
                          <div className="font-medium">{template.name}</div>
                          <Button
                            size="small"
                            onClick={() => applyTemplate(template)}
                          >
                            Use Template
                          </Button>
                        </Flex>
                        <Text type="secondary" className="text-sm">
                          {template.description}
                        </Text>
                      </Card>
                    ))}
                  </div>
                ),
              },
            ]}
          />
        )}

        {/* Basic Information */}
        <Form.Item
          name="name"
          label="Provider Name"
          rules={[{ required: true, message: 'Please enter provider name' }]}
        >
          <Input placeholder="e.g., Google OAuth, Azure AD, Company LDAP" />
        </Form.Item>

        <Form.Item
          name="provider_type"
          label="Provider Type"
          rules={[{ required: true, message: 'Please select provider type' }]}
        >
          <Select
            placeholder="Select provider type"
            disabled={isEditMode}
            options={[
              { value: 'oauth2', label: 'OAuth 2.0' },
              { value: 'oidc', label: 'OpenID Connect (OIDC)' },
              { value: 'ldap', label: 'LDAP' },
              { value: 'saml', label: 'SAML' },
            ]}
          />
        </Form.Item>

        <Form.Item
          name="priority"
          label="Priority"
          tooltip="Lower numbers have higher priority (0 = highest)"
        >
          <InputNumber min={0} placeholder="0" className="w-full" />
        </Form.Item>

        <Form.Item
          name="enabled"
          label="Enabled"
          valuePropName="checked"
          initialValue={true}
        >
          <Switch />
        </Form.Item>

        {/* Provider-specific Configuration */}
        <Form.Item
          noStyle
          shouldUpdate={(prev, curr) =>
            prev.provider_type !== curr.provider_type
          }
        >
          {({ getFieldValue }) => {
            const providerType = getFieldValue('provider_type')
            return providerType ? renderConfigForm(providerType) : null
          }}
        </Form.Item>
      </Form>
    </Drawer>
  )
}
