import { App, Card, Form, Switch, Typography } from "antd";
import { useEffect } from "react";
import { useTranslation } from "react-i18next";
import { Permission, usePermissions } from "../../../../permissions";
import {
  clearSystemAdminError,
  loadSystemUserRegistrationSettings,
  Stores,
  updateSystemUserRegistrationSettings,
} from "../../../../store";

const { Text } = Typography;

export function UserRegistrationSettings() {
  const { t } = useTranslation();
  const { message } = App.useApp();
  const [form] = Form.useForm();
  const { hasPermission } = usePermissions();

  // Admin store
  const { userRegistrationEnabled, loading, error } = Stores.Admin;

  const canRead = hasPermission(Permission.config.userRegistration.read);
  const canEdit = hasPermission(Permission.config.userRegistration.edit);

  useEffect(() => {
    if (canRead) {
      loadSystemUserRegistrationSettings();
    }
  }, [canRead]);

  // Show errors
  useEffect(() => {
    if (error) {
      message.error(error);
      clearSystemAdminError();
    }
  }, [error, message]);

  // Update form when registration status changes
  useEffect(() => {
    form.setFieldsValue({ enabled: userRegistrationEnabled });
  }, [userRegistrationEnabled, form]);

  const handleFormChange = async (changedValues: any) => {
    if (!canEdit) {
      message.error(t("admin.noPermissionEditSetting"));
      return;
    }
    if ("enabled" in changedValues) {
      const newValue = changedValues.enabled;

      try {
        await updateSystemUserRegistrationSettings(newValue);
        message.success(
          `User registration ${newValue ? "enabled" : "disabled"} successfully`,
        );
      } catch (error) {
        console.error("Failed to update registration status:", error);
        // Error is handled by the store
      }
    }
  };

  if (!canRead) {
    return null;
  }

  return (
    <Card title={t("admin.userRegistration")}>
      <Form
        form={form}
        onValuesChange={handleFormChange}
        initialValues={{ enabled: userRegistrationEnabled }}
      >
        <div className="flex justify-between items-center">
          <div>
            <Text strong>Enable User Registration</Text>
            <div>
              <Text type="secondary">
                Allow new users to register for accounts
              </Text>
            </div>
          </div>
          <Form.Item name="enabled" valuePropName="checked" className="mb-0">
            <Switch loading={loading} size="default" />
          </Form.Item>
        </div>
      </Form>
    </Card>
  );
}
