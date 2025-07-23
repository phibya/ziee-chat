import { Button, Card, Flex, Form, Modal } from "antd";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { Stores, updateExistingModel } from "../../../../store";
import { closeEditModelModal } from "../../../../store/ui/modals";
import { BASIC_MODEL_FIELDS, LOCAL_PARAMETERS } from "./shared/constants";
import { DeviceSelectionSection } from "./shared/DeviceSelectionSection";
import { ModelCapabilitiesSection } from "./shared/ModelCapabilitiesSection";
import { ModelParametersSection } from "./shared/ModelParametersSection";
import { ModelSettingsSection } from "./shared/ModelSettingsSection";

export function EditModelModal() {
  const { t } = useTranslation();
  const [form] = Form.useForm();
  const [loading, setLoading] = useState(false);

  const { editModelModalOpen, editingModelId } = Stores.UI.Modals;
  const { providers, modelsByProvider } = Stores.Providers;

  // Find the current model and provider from the store
  const currentModel = editingModelId
    ? Object.values(modelsByProvider)
        .flat()
        .find((m) => m.id === editingModelId)
    : null;
  const currentProvider = currentModel
    ? providers.find((p) =>
        modelsByProvider[p.id]?.some((m) => m.id === editingModelId),
      )
    : null;

  useEffect(() => {
    if (currentModel && editModelModalOpen) {
      form.setFieldsValue({
        name: currentModel.name,
        alias: currentModel.alias,
        description: currentModel.description,
        capabilities: currentModel.capabilities || {},
        parameters: currentModel.parameters || {},
        settings: currentModel.settings || {},
      });
    }
  }, [currentModel, editModelModalOpen, form]);

  const handleSubmit = async () => {
    if (!currentModel) return;

    try {
      setLoading(true);
      const values = await form.validateFields();

      const modelData = {
        ...currentModel,
        ...values,
      };
      await updateExistingModel(modelData.id, modelData);
      closeEditModelModal();
    } catch (error) {
      console.error("Failed to update model:", error);
    } finally {
      setLoading(false);
    }
  };

  return (
    <Modal
      title={t("providers.editModel")}
      open={editModelModalOpen}
      onCancel={closeEditModelModal}
      footer={[
        <Button key="cancel" onClick={closeEditModelModal}>
          {t("buttons.cancel")}
        </Button>,
        <Button
          key="submit"
          type="primary"
          loading={loading}
          onClick={handleSubmit}
        >
          {t("buttons.saveChanges")}
        </Button>,
      ]}
      width={600}
      maskClosable={false}
    >
      <Form form={form} layout="vertical">
        <ModelParametersSection parameters={BASIC_MODEL_FIELDS} />

        <Flex className={`flex-col gap-3`}>
          <ModelCapabilitiesSection />

          {currentProvider?.type === "local" && <DeviceSelectionSection />}

          {currentProvider?.type === "local" && <ModelSettingsSection />}

          {currentProvider?.type === "local" && (
            <Card title={t("providers.parameters")} size={"small"}>
              <ModelParametersSection parameters={LOCAL_PARAMETERS} />
            </Card>
          )}
        </Flex>
      </Form>
    </Modal>
  );
}
