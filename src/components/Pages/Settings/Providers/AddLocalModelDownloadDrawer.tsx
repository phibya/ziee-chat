import {
  App,
  Button,
  Card,
  Form,
  Input,
  Progress,
  Select,
  Typography,
} from "antd";
import { Drawer } from "../../../common/Drawer.tsx";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { ApiClient } from "../../../../api/client";
import {
  cancelModelDownload,
  clearProvidersError,
  closeAddLocalModelDownloadDrawer,
  closeViewDownloadModal,
  downloadModelFromRepository,
  openViewDownloadModal,
  Stores,
} from "../../../../store";
import { Repository } from "../../../../types/api/repository";
import { LocalModelCommonFields } from "./shared/LocalModelCommonFields";

const { Text } = Typography;

export function AddLocalModelDownloadDrawer() {
  const { t } = useTranslation();
  const { message } = App.useApp();
  const [form] = Form.useForm();
  const [loading, setLoading] = useState(false);
  const [repositories, setRepositories] = useState<Repository[]>([]);
  const [loadingRepositories, setLoadingRepositories] = useState(false);
  const { downloads } = Stores.ModelDownload;

  // Function to generate a unique model ID from display name
  const generateModelId = (displayName: string): string => {
    const baseId = displayName
      .toLowerCase()
      .replace(/[^a-z0-9\s-]/g, "")
      .replace(/\s+/g, "-")
      .replace(/-+/g, "-")
      .replace(/^-|-$/g, "")
      .substring(0, 50);

    const timestamp = Date.now().toString(36);
    return `${baseId}-${timestamp}`;
  };

  // Get values from form
  const selectedRepository = Form.useWatch("repository_id", form);

  // Load available repositories
  const loadRepositories = async () => {
    try {
      setLoadingRepositories(true);
      const response = await ApiClient.Admin.listRepositories({});
      const enabledRepos = response.repositories.filter(
        (repo: Repository) => repo.enabled,
      );
      setRepositories(enabledRepos);
    } catch (error) {
      console.error("Failed to load repositories:", error);
      message.error(t("providers.failedToLoadRepositories"));
    } finally {
      setLoadingRepositories(false);
    }
  };

  const { open: addMode, providerId } = Stores.UI.AddLocalModelDownloadModal;
  const { open: viewMode, downloadId } = Stores.UI.ViewDownloadModal;

  const open = viewMode || addMode;

  // Helper function to close the appropriate modal
  const handleCloseModal = () => {
    closeAddLocalModelDownloadDrawer();
    closeViewDownloadModal();
    setLoading(false);
  };

  // Get download instance from store
  const viewDownload = Object.values(Stores.ModelDownload.downloads).find(
    (d) => d.id === downloadId,
  );

  const handleSubmit = async () => {
    try {
      setLoading(true);
      clearProvidersError();
      const values = await form.validateFields();

      // Auto-generate model ID from display name
      const modelId = generateModelId(values.alias || "model");

      if (!values.repository_id) {
        message.error(t("providers.repositoryRequired"));
        return;
      }

      if (!values.repository_path) {
        message.error(t("providers.repositoryPathRequired"));
        return;
      }

      // Get the selected repository details
      const selectedRepo = repositories.find(
        (repo) => repo.id === values.repository_id,
      );
      if (!selectedRepo) {
        message.error(t("providers.repositoryNotFound"));
        return;
      }

      const isAnotherDownloadInProgress = Object.values(downloads).some(
        (download) =>
          download.provider_id === providerId &&
          download.repository_id === values.repository_id &&
          download.request_data.repository_path === values.repository_path &&
          (download.status === "downloading" || download.status === "pending"),
      );

      if (isAnotherDownloadInProgress) {
        message.error(
          "Another download with the same repository is already in progress. Please wait for it to complete.",
        );
        return;
      }

      // Call the repository download API through store
      try {
        await downloadModelFromRepository(
          {
            provider_id: providerId!,
            repository_id: values.repository_id,
            repository_path: values.repository_path,
            main_filename: values.main_filename,
            repository_branch: values.repository_branch,
            name: modelId,
            alias: values.alias,
            description: values.description,
            file_format: values.file_format,
            capabilities: values.capabilities || {},
            settings: values.settings || {},
          },
          openViewDownloadModal,
        );

        message.success(t("providers.downloadStarted"));
      } catch (error) {
        console.error("Failed to start download:", error);
        message.error(t("providers.downloadStartFailed"));
      }
    } catch (error) {
      console.error("Failed to create model:", error);
      message.error(t("providers.failedToCreateModel"));
    } finally {
      setLoading(false);
    }
  };

  const handleCancel = () => {
    handleCloseModal();
  };

  // Load repositories and pre-fill form when modal opens
  useEffect(() => {
    if (open) {
      loadRepositories();
      if (viewDownload) {
        // In view mode, populate form with download data from request_data
        const requestData = viewDownload.request_data;
        form.setFieldsValue({
          alias: requestData.alias,
          description: requestData.description || "",
          file_format: requestData.file_format,
          repository_id: viewDownload.repository_id, // Get from download instance, not request_data
          repository_path: requestData.repository_path,
          main_filename: requestData.main_filename,
          repository_branch: requestData.revision || "main", // Use revision instead of repository_branch
          capabilities: requestData.capabilities || {},
          settings: requestData.settings || {},
        });
      } else if (!viewMode) {
        // In add mode, set default values
        form.setFieldsValue({
          alias: "TinyLlama Chat Model",
          description:
            "Small 1.1B parameter chat model for quick testing (~637MB)",
          file_format: "safetensors",
          repository_path: "meta-llama/Llama-3.1-8B-Instruct",
          main_filename: "model.safetensors",
          repository_branch: "main",
          settings: {},
        });
      }
    }
  }, [open, viewMode, viewDownload, form]);

  return (
    <Drawer
      title={
        viewMode ? "View Download Details" : t("providers.downloadLocalModel")
      }
      open={open}
      onClose={handleCloseModal}
      footer={
        viewMode
          ? [
              <Button key="close" onClick={handleCloseModal}>
                {t("buttons.close")}
              </Button>,
              viewDownload &&
                (viewDownload.status === "downloading" ||
                  viewDownload.status === "pending") && (
                  <Button
                    key="cancel-download"
                    danger
                    onClick={async () => {
                      try {
                        await cancelModelDownload(viewDownload.id);
                        message.success("Download cancelled successfully");
                      } catch (error: any) {
                        console.error("Failed to cancel download:", error);
                        message.error(
                          `Failed to cancel download: ${error.message}`,
                        );
                      }
                    }}
                  >
                    {t("buttons.cancel")} Download
                  </Button>
                ),
            ].filter(Boolean)
          : [
              <Button key="cancel" onClick={handleCancel}>
                {t("buttons.cancel")}
              </Button>,
              <Button
                key="submit"
                type="primary"
                loading={loading}
                onClick={handleSubmit}
              >
                {t("buttons.startDownload")}
              </Button>,
            ]
      }
      width={600}
      maskClosable={false}
    >
      <div>
        {viewDownload && viewDownload.progress_data && (
          <Card
            title={t("providers.downloadProgress")}
            size="small"
            style={{ marginBottom: 16 }}
          >
            <Text>
              {viewDownload.progress_data?.phase || viewDownload.status}
            </Text>
            <Progress
              percent={Math.round(
                ((viewDownload.progress_data.current || 0) /
                  (viewDownload.progress_data.total || 1)) *
                  100,
              )}
              status={
                viewDownload.status === "downloading"
                  ? "active"
                  : viewDownload.status === "completed"
                    ? "success"
                    : viewDownload.status === "failed"
                      ? "exception"
                      : "normal"
              }
              format={(percent) => `${percent}%`}
            />
            <Text type="secondary" style={{ fontSize: "12px" }}>
              {viewDownload.progress_data.message ||
                viewDownload.error_message ||
                ""}
            </Text>
            {viewDownload.progress_data.download_speed && (
              <div style={{ marginTop: 8 }}>
                <Text type="secondary" style={{ fontSize: "12px" }}>
                  Speed:{" "}
                  {Math.round(
                    (viewDownload.progress_data.download_speed / 1024 / 1024) *
                      10,
                  ) / 10}{" "}
                  MB/s
                  {viewDownload.progress_data.eta_seconds && (
                    <>
                      {" "}
                      • ETA:{" "}
                      {Math.round(
                        viewDownload.progress_data.eta_seconds / 60,
                      )}{" "}
                      minutes
                    </>
                  )}
                </Text>
              </div>
            )}
          </Card>
        )}

        <Form
          form={form}
          layout="vertical"
          disabled={viewMode}
          initialValues={{
            file_format: "safetensors",
            main_filename: "",
            repository_branch: "main",
            settings: {},
          }}
        >
          <LocalModelCommonFields />

          <Form.Item
            name="repository_id"
            label={t("providers.selectRepository")}
            rules={[
              {
                required: true,
                message: t("providers.repositoryRequired"),
              },
            ]}
          >
            <Select
              placeholder={t("providers.selectRepositoryPlaceholder")}
              loading={loadingRepositories}
              showSearch
              optionFilterProp="children"
              options={repositories.map((repo) => ({
                value: repo.id,
                label: `${repo.name} (${repo.url})`,
              }))}
            />
          </Form.Item>

          <Form.Item
            name="repository_path"
            label={t("providers.repositoryPath")}
            rules={[
              {
                required: true,
                message: t("providers.repositoryPathRequired"),
              },
            ]}
          >
            <Input
              placeholder="microsoft/DialoGPT-medium"
              addonBefore={
                selectedRepository
                  ? repositories.find((r) => r.id === selectedRepository)
                      ?.url || "Repository"
                  : "Repository"
              }
            />
          </Form.Item>

          <Form.Item
            name="main_filename"
            label={t("providers.mainFilename")}
            rules={[
              {
                required: true,
                message: t("providers.localFilenameRequired"),
              },
            ]}
          >
            <Input placeholder="model.safetensors" />
          </Form.Item>

          <Form.Item
            name="repository_branch"
            label={t("providers.repositoryBranch")}
          >
            <Input placeholder="main" />
          </Form.Item>
        </Form>
      </div>
    </Drawer>
  );
}
