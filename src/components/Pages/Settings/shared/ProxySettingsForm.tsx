import {
  Button,
  Card,
  Col,
  Divider,
  Flex,
  Form,
  Input,
  message,
  Row,
  Space,
  Switch,
  Typography,
} from "antd";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { testProxyDetailed } from "../../../../api/proxy";
import { isProxyValid, type ProxySettings } from "./";

const { Text, Title } = Typography;

export interface ProxySettingsFormProps {
  initialSettings: ProxySettings | null;
  onSave: (values: ProxySettings) => Promise<void> | void;
  disabled?: boolean;
}

export function ProxySettingsForm({
  initialSettings,
  onSave,
  disabled = false,
}: ProxySettingsFormProps) {
  const { t } = useTranslation();
  const [form] = Form.useForm();
  const [loading, setLoading] = useState(false);
  const [testingProxy, setTestingProxy] = useState(false);

  // Update form when initial settings change
  useEffect(() => {
    if (initialSettings) {
      form.setFieldsValue(initialSettings);
    }
  }, [initialSettings, form]);

  const handleSave = async () => {
    try {
      setLoading(true);
      const values = await form.validateFields();

      await onSave(values);
      message.success(t("proxy.settingsSaved"));
    } catch (error) {
      console.error("Failed to save proxy settings:", error);
      message.error(t("proxy.saveFailed"));
    } finally {
      setLoading(false);
    }
  };

  const handleReset = () => {
    if (initialSettings) {
      form.setFieldsValue(initialSettings);
    }
  };

  const handleTestProxy = async () => {
    try {
      setTestingProxy(true);
      const values = form.getFieldsValue();

      // Only test if URL is provided
      if (!values.url || values.url.trim() === "") {
        message.warning(t("proxy.enterValidUrl"));
        return;
      }

      // Test the proxy using the API client
      const result = await testProxyDetailed(values);
      const success = result.success;

      if (success) {
        message.success(t("proxy.testSuccessful"));
      } else {
        message.error(result.message || t("proxy.testFailed"));
      }
    } catch (error) {
      console.error("Proxy test failed:", error);
      message.error(t("proxy.testFailed"));
    } finally {
      setTestingProxy(false);
    }
  };

  return (
    <Card title={"Proxy"}>
      <Form form={form} layout="vertical" onFinish={handleSave}>
        <Flex className={"flex-col"}>
          <Flex className={"flex-col gap-3"}>
            {/* Enable Proxy Toggle */}
            <div>
              <div className={"flex justify-between items-center"}>
                <div style={{ flex: 1, marginRight: 16 }}>
                  <Text strong>{t("proxy.enableProxy")}</Text>
                  <br />
                  <Text type="secondary">{t("proxy.enableProxyDesc")}</Text>
                </div>
                <Form.Item
                  name="enabled"
                  valuePropName="checked"
                  style={{ margin: 0 }}
                >
                  <Switch disabled={disabled} />
                </Form.Item>
              </div>
            </div>

            {/* Proxy URL */}
            <div>
              <Text strong>{t("proxy.proxyUrl")}</Text>
              <br />
              <Text type="secondary">{t("proxy.proxyUrlDesc")}</Text>
              <Form.Item
                name="url"
                style={{ marginTop: 8 }}
                dependencies={["enabled"]}
                validateTrigger={["onChange", "onBlur"]}
                rules={[
                  () => ({
                    validator(_, value) {
                      if (value && value.trim() !== "") {
                        try {
                          const url = new URL(value);
                          const allowedProtocols = ['http:', 'https:', 'socks5:'];
                          if (!allowedProtocols.includes(url.protocol)) {
                            return Promise.reject(
                              new Error("URL must start with http://, https://, or socks5://"),
                            );
                          }
                          return Promise.resolve();
                        } catch {
                          return Promise.reject(
                            new Error("Invalid URL format"),
                          );
                        }
                      }
                      return Promise.resolve();
                    },
                  }),
                ]}
              >
                <Input
                  placeholder={t("proxy.proxyUrlPlaceholder")}
                  disabled={disabled}
                />
              </Form.Item>
            </div>

            {/* Authentication */}
            <div>
              <Text strong>{t("proxy.authentication")}</Text>
              <br />
              <Text type="secondary">{t("proxy.authDesc")}</Text>
              <Row gutter={8} style={{ marginTop: 8 }}>
                <Col span={12}>
                  <Form.Item name="username">
                    <Input
                      placeholder={t("proxy.usernamePlaceholder")}
                      disabled={disabled}
                    />
                  </Form.Item>
                </Col>
                <Col span={12}>
                  <Form.Item name="password">
                    <Input.Password
                      placeholder={t("proxy.passwordPlaceholder")}
                      disabled={disabled}
                    />
                  </Form.Item>
                </Col>
              </Row>
            </div>

            {/* No Proxy */}
            <div>
              <Text strong>{t("proxy.noProxy")}</Text>
              <br />
              <Text type="secondary">{t("proxy.noProxyDesc")}</Text>
              <Form.Item name="no_proxy" style={{ marginTop: 8 }}>
                <Input
                  placeholder={t("proxy.noProxyPlaceholder")}
                  disabled={disabled}
                />
              </Form.Item>
            </div>
          </Flex>

          <Divider />

          {/* SSL Configuration */}
          <div className={"pt-2"}>
            <Title level={4} className={"pb-2"}>
              {t("proxy.sslVerification")}
            </Title>

            {/* Ignore SSL Certificates */}
            <div style={{ marginBottom: 24 }}>
              <div
                style={{
                  display: "flex",
                  justifyContent: "space-between",
                  alignItems: "center",
                }}
              >
                <div style={{ flex: 1, marginRight: 16 }}>
                  <Text strong>{t("proxy.ignoreSslCerts")}</Text>
                  <br />
                  <Text type="secondary">{t("proxy.ignoreSslCertsDesc")}</Text>
                </div>
                <Form.Item
                  name="ignore_ssl_certificates"
                  valuePropName="checked"
                  style={{ margin: 0 }}
                >
                  <Switch disabled={disabled} />
                </Form.Item>
              </div>
            </div>

            {/* Proxy SSL */}
            <div style={{ marginBottom: 24 }}>
              <div
                style={{
                  display: "flex",
                  justifyContent: "space-between",
                  alignItems: "center",
                }}
              >
                <div style={{ flex: 1, marginRight: 16 }}>
                  <Text strong>{t("proxy.proxySsl")}</Text>
                  <br />
                  <Text type="secondary">{t("proxy.proxySslDesc")}</Text>
                </div>
                <Form.Item
                  name="proxy_ssl"
                  valuePropName="checked"
                  style={{ margin: 0 }}
                >
                  <Switch disabled={disabled} />
                </Form.Item>
              </div>
            </div>

            {/* Proxy Host SSL */}
            <div style={{ marginBottom: 24 }}>
              <div
                style={{
                  display: "flex",
                  justifyContent: "space-between",
                  alignItems: "center",
                }}
              >
                <div style={{ flex: 1, marginRight: 16 }}>
                  <Text strong>{t("proxy.proxyHostSsl")}</Text>
                  <br />
                  <Text type="secondary">{t("proxy.proxyHostSslDesc")}</Text>
                </div>
                <Form.Item
                  name="proxy_host_ssl"
                  valuePropName="checked"
                  style={{ margin: 0 }}
                >
                  <Switch disabled={disabled} />
                </Form.Item>
              </div>
            </div>

            {/* Peer SSL */}
            <div style={{ marginBottom: 24 }}>
              <div
                style={{
                  display: "flex",
                  justifyContent: "space-between",
                  alignItems: "center",
                }}
              >
                <div style={{ flex: 1, marginRight: 16 }}>
                  <Text strong>{t("proxy.peerSsl")}</Text>
                  <br />
                  <Text type="secondary">{t("proxy.peerSslDesc")}</Text>
                </div>
                <Form.Item
                  name="peer_ssl"
                  valuePropName="checked"
                  style={{ margin: 0 }}
                >
                  <Switch disabled={disabled} />
                </Form.Item>
              </div>
            </div>

            {/* Host SSL */}
            <div>
              <div
                style={{
                  display: "flex",
                  justifyContent: "space-between",
                  alignItems: "center",
                }}
              >
                <div style={{ flex: 1, marginRight: 16 }}>
                  <Text strong>{t("proxy.hostSsl")}</Text>
                  <br />
                  <Text type="secondary">{t("proxy.hostSslDesc")}</Text>
                </div>
                <Form.Item
                  name="host_ssl"
                  valuePropName="checked"
                  style={{ margin: 0 }}
                >
                  <Switch disabled={disabled} />
                </Form.Item>
              </div>
            </div>
          </div>
        </Flex>

        <Divider />

        <div className={"flex justify-end"}>
          <Form.Item
            noStyle
            shouldUpdate={(prev, curr) => prev.url !== curr.url}
          >
            {({ getFieldsValue }) => {
              const values = getFieldsValue();
              const canTest = isProxyValid(values);

              return (
                <Space>
                  <Button onClick={handleReset} disabled={disabled}>
                    {t("proxy.reset")}
                  </Button>
                  <Button
                    onClick={handleTestProxy}
                    loading={testingProxy}
                    disabled={!canTest || disabled}
                  >
                    {t("proxy.testProxy")}
                  </Button>
                  <Button
                    type="primary"
                    htmlType="submit"
                    loading={loading}
                    disabled={disabled}
                  >
                    {t("common.save")}
                  </Button>
                </Space>
              );
            }}
          </Form.Item>
        </div>
      </Form>
    </Card>
  );
}
