import { UploadOutlined } from "@ant-design/icons";
import {
  App,
  Button,
  Card,
  Flex,
  Form,
  Input,
  List,
  Modal,
  Radio,
  Select,
  Tag,
  Typography,
  Upload,
} from "antd";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { useUpdate } from "react-use";
import { useShallow } from "zustand/react/shallow";
import { ApiClient } from "../../../../api/client";
import { LOCAL_FILE_TYPE_OPTIONS } from "../../../../constants/localModelTypes.ts";
import { useProvidersStore } from "../../../../store/providers";
import { useModelDownloadStore } from "../../../../store/modelDownload";
import { ProviderType } from "../../../../types/api/provider";
import { Repository } from "../../../../types/api/repository";
import { BASIC_MODEL_FIELDS, LOCAL_MODEL_FIELDS } from "./shared/constants";
import { ModelParametersSection } from "./shared/ModelParametersSection";
import { UploadProgress } from "./UploadProgress";

interface AddModelModalProps {
  open: boolean;
  providerId: string;
  providerType: ProviderType;
  onClose: () => void;
  onSubmit: (modelData: any) => void;
}

export function AddModelModal({
  open,
  providerId,
  providerType,
  onClose,
  onSubmit,
}: AddModelModalProps) {
  const { t } = useTranslation();
  const { message } = App.useApp();
  const [form] = Form.useForm();
  const [loading, setLoading] = useState(false);
  const [selectedFiles, setSelectedFiles] = useState<File[]>([]);
  const [filteredFiles, setFilteredFiles] = useState<
    { file: File; purpose: string; required: boolean }[]
  >([]);
  const [repositories, setRepositories] = useState<Repository[]>([]);
  const [loadingRepositories, setLoadingRepositories] = useState(false);
  const update = useUpdate();

  // Function to generate a unique model ID from display name
  const generateModelId = (displayName: string): string => {
    // Convert display name to a URL-friendly ID
    const baseId = displayName
      .toLowerCase()
      .replace(/[^a-z0-9\s-]/g, "") // Remove special characters except spaces and hyphens
      .replace(/\s+/g, "-") // Replace spaces with hyphens
      .replace(/-+/g, "-") // Replace multiple hyphens with single hyphen
      .replace(/^-|-$/g, "") // Remove leading/trailing hyphens
      .substring(0, 50); // Limit length

    // Add timestamp to ensure uniqueness
    const timestamp = Date.now().toString(36); // Base36 for shorter string
    return `${baseId}-${timestamp}`;
  };

  // Get values from form instead of separate state
  const selectedFileFormat =
    Form.useWatch("file_format", form) || "safetensors";
  const modelSource = Form.useWatch("model_source", form) || "upload";
  const selectedRepository = Form.useWatch("repository_id", form);

  // Load available repositories
  const loadRepositories = async () => {
    try {
      setLoadingRepositories(true);
      const response = await ApiClient.Repositories.list({});
      // Filter to only enabled repositories
      const enabledRepos = response.repositories.filter((repo) => repo.enabled);
      setRepositories(enabledRepos);
    } catch (error) {
      console.error("Failed to load repositories:", error);
      message.error(t("providers.failedToLoadRepositories"));
    } finally {
      setLoadingRepositories(false);
    }
  };

  const {
    uploadMultipleFilesAndCommit,
    uploading,
    uploadProgress,
    overallUploadProgress,
    clearError,
    cancelUpload,
  } = useProvidersStore(
    useShallow((state) => ({
      uploadMultipleFilesAndCommit: state.uploadMultipleFilesAndCommit,
      uploading: state.uploading,
      uploadProgress: state.uploadProgress,
      overallUploadProgress: state.overallUploadProgress,
      clearError: state.clearError,
      cancelUpload: state.cancelUpload,
    })),
  );

  const {
    downloading,
    downloadProgress,
    downloadFromRepository,
    clearError: clearDownloadError,
  } = useModelDownloadStore(
    useShallow((state) => ({
      downloading: state.downloading,
      downloadProgress: state.downloadProgress,
      downloadFromRepository: state.downloadFromRepository,
      clearError: state.clearError,
    })),
  );

  const handleSubmit = async () => {
    try {
      setLoading(true);
      clearError(); // Clear any previous errors
      const values = await form.validateFields();

      if (providerType === "local") {
        // Auto-generate model ID from display name for Local models
        const modelId = generateModelId(values.alias || "model");

        if (values.model_source === "upload") {
          // Step 1: Upload files using new workflow
          if (selectedFiles.length === 0) {
            message.error(t("providers.selectModelFolderRequired"));
            return;
          }

          if (!values.main_filename) {
            message.error(t("providers.localFilenameRequired"));
            return;
          }

          // Comprehensive validation of selected files
          const validation = validateModelFiles(
            selectedFiles,
            values.file_format,
          );

          if (!validation.isValid) {
            validation.errors.forEach((error) => {
              message.error(error);
            });
            return;
          }

          // Show warnings but allow upload to continue
          if (validation.warnings.length > 0) {
            validation.warnings.forEach((warning) => {
              message.warning(warning);
            });
          }

          // Validate that the specified main file exists in filtered files
          const filesToUpload = filteredFiles.map((item) => item.file);
          const mainFile = filesToUpload.find(
            (file) => file.name === values.main_filename,
          );
          if (!mainFile) {
            message.error(t("providers.mainFileNotFound"));
            return;
          }

          // Upload and auto-commit the files as a model in a single request
          await uploadMultipleFilesAndCommit({
            provider_id: providerId,
            files: filesToUpload,
            main_filename: values.main_filename,
            name: modelId, // Auto-generated model ID
            alias: values.alias, // Display name
            description: values.description,
            file_format: values.file_format,
            capabilities: values.capabilities, // Include capabilities from form
            settings: values.settings, // Include model settings from form
          });

          message.success(t("providers.modelFolderUploadedSuccessfully"));

          // Clear upload progress after successful upload
          cancelUpload();
        } else if (values.model_source === "repository") {
          // Repository-based download workflow
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

          // Call the repository download API through store
          try {
            await downloadFromRepository({
              provider_id: providerId,
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
            });

            // Update the providers store with the new model
            const { loadProviderModels } = useProvidersStore.getState();
            await loadProviderModels(providerId);

            message.success(
              t("providers.modelDownloadFromRepositoryCompleted"),
            );
          } catch (error) {
            console.error("Failed to download from repository:", error);
            message.error(t("providers.modelDownloadFromRepositoryFailed"));
            return;
          }
        }

        // Step 5: Close the modal for adding model
        form.resetFields();
        setSelectedFiles([]);
        setFilteredFiles([]);
        // No need to clear session since we auto-commit
        onClose();

        // Step 6: Trigger parent refresh (if needed)
        await onSubmit({ type: "local-upload", success: true });
      } else {
        // For other providers, use the existing workflow
        const modelData = {
          id: `model-${Date.now()}`,
          ...values,
          enabled: true,
          capabilities: {
            vision: values.vision || false,
            audio: values.audio || false,
            tools: values.tools || false,
            codeInterpreter: values.codeInterpreter || false,
          },
        };

        // Remove capability checkboxes from main data
        delete modelData.vision;
        delete modelData.audio;
        delete modelData.tools;
        delete modelData.codeInterpreter;

        await onSubmit(modelData);

        form.resetFields();
        setSelectedFiles([]);
        setFilteredFiles([]);
        onClose();
      }
    } catch (error) {
      console.error("Failed to add model:", error);
    } finally {
      setLoading(false);
    }
  };

  // Load repositories and pre-fill form when modal opens
  useEffect(() => {
    if (open && providerType === "local") {
      // Load available repositories
      loadRepositories();

      // Set form values for quick testing with a tiny chat model
      form.setFieldsValue({
        alias: "TinyLlama Chat Model", // Only display name for Local models
        description:
          "Small 1.1B parameter chat model for quick testing (~637MB)",
        file_format: "safetensors",
        model_source: "repository",
        repository_path: "meta-llama/Llama-3.1-8B-Instruct",
        main_filename: "model.safetensors",
        repository_branch: "main",
        settings: {},
      });
      update(); // Force re-render to update form watchers
    }
  }, [open, providerType, form, update]);

  // Clear errors when modal closes
  useEffect(() => {
    if (!open) {
      clearError();
      clearDownloadError();
    }
  }, [open, clearError, clearDownloadError]);

  const handleFolderSelect = (info: any) => {
    const fileList = info.fileList || [];
    const files = fileList.map(
      (file: any) => file.originFileObj || file.file || file,
    );

    if (files.length > 0) {
      // Get the common folder path from the first file
      const firstFile = files[0];
      let folderPath = "";

      if (firstFile.webkitRelativePath) {
        const pathParts = firstFile.webkitRelativePath.split("/");
        folderPath = pathParts.slice(0, -1).join("/");
      } else if (firstFile.path) {
        const pathParts = firstFile.path.split("/");
        folderPath = pathParts.slice(0, -1).join("/");
      }

      setSelectedFiles(files);
      form.setFieldsValue({
        local_folder_path: folderPath || "Selected folder",
      });

      // Categorize and filter files based on selected format
      const currentFormat = selectedFileFormat;
      const categorizedFiles = categorizeFiles(files, currentFormat);
      setFilteredFiles(categorizedFiles);

      // Validate the filtered files
      const validation = validateModelFiles(files, currentFormat);

      // Show validation errors
      if (validation.errors.length > 0) {
        validation.errors.forEach((error) => {
          message.error(error);
        });
      }

      // Show validation warnings
      if (validation.warnings.length > 0) {
        validation.warnings.forEach((warning) => {
          message.warning(warning);
        });
      }

      // Try to find the main model file using fuzzy matching
      const suggestedMainFile = findMainModelFile(files, currentFormat);

      if (suggestedMainFile) {
        form.setFieldsValue({
          main_filename: suggestedMainFile,
        });
        message.success(
          `Selected ${categorizedFiles.length} relevant files from folder. Suggested main file: ${suggestedMainFile}`,
        );
      } else {
        message.success(
          `Selected ${categorizedFiles.length} relevant files from folder`,
        );
      }
    }
  };

  const handleFileFormatChange = (value: string) => {
    // Clear the current filename when format changes to guide user
    form.setFieldsValue({
      main_filename: "",
    });

    // Recategorize files if we have selected files
    if (selectedFiles.length > 0) {
      const categorizedFiles = categorizeFiles(selectedFiles, value);
      setFilteredFiles(categorizedFiles);

      // Try to auto-fill with a new main file suggestion
      const suggestedMainFile = findMainModelFile(selectedFiles, value);
      if (suggestedMainFile) {
        form.setFieldsValue({
          main_filename: suggestedMainFile,
        });
      }
    }

    console.log(
      "File format changed to:",
      value,
      "Current format:",
      selectedFileFormat,
    );

    update(); // Force re-render to update form watchers
  };

  const getFilenamePlaceholder = (fileFormat: string) => {
    switch (fileFormat) {
      case "safetensors":
        return "model.safetensors";
      case "pytorch":
        return "pytorch_model.bin";
      case "gguf":
        return "model.gguf";
      default:
        return "pytorch_model.bin";
    }
  };

  const validateFilename = (filename: string, fileFormat: string) => {
    if (!filename) return false;

    const validExtensions = {
      safetensors: [".safetensors"],
      pytorch: [".bin", ".pt", ".pth"],
      gguf: [".gguf"],
    };

    const extensions = validExtensions[
      fileFormat as keyof typeof validExtensions
    ] || [".bin"];
    return extensions.some((ext) => filename.toLowerCase().endsWith(ext));
  };

  // File validation utilities for different model formats
  const validateModelFiles = (files: File[], fileFormat: string) => {
    const errors: string[] = [];
    const warnings: string[] = [];
    const fileNames = files.map((f) => f.name.toLowerCase());

    // Common required files across formats
    const hasConfigJson = fileNames.some(
      (name) =>
        name === "config.json" ||
        (name.includes("config") && name.endsWith(".json")),
    );
    const hasTokenizerJson = fileNames.some(
      (name) =>
        name === "tokenizer.json" ||
        (name.includes("tokenizer") && name.endsWith(".json")),
    );
    const hasTokenizerConfig = fileNames.some(
      (name) =>
        name === "tokenizer_config.json" || name.includes("tokenizer_config"),
    );

    // Format-specific validation - only check file extensions
    switch (fileFormat) {
      case "safetensors": {
        // Check for any SafeTensors file
        const safetensorsFiles = fileNames.filter((name) =>
          name.endsWith(".safetensors"),
        );
        if (safetensorsFiles.length === 0) {
          errors.push(
            "Missing SafeTensors model file (.safetensors extension required)",
          );
        } else if (safetensorsFiles.length > 1) {
          // Check for index file if multiple shards
          const hasIndex = fileNames.some(
            (name) =>
              name === "model.safetensors.index.json" ||
              name === "pytorch_model.safetensors.index.json",
          );
          if (!hasIndex) {
            warnings.push(
              "Multiple SafeTensors files found but no index file detected",
            );
          }
        }
        break;
      }

      case "pytorch": {
        // Check for any PyTorch model file
        const pytorchFiles = fileNames.filter(
          (name) =>
            name.endsWith(".bin") ||
            name.endsWith(".pt") ||
            name.endsWith(".pth"),
        );
        if (pytorchFiles.length === 0) {
          errors.push(
            "Missing PyTorch model file (.bin, .pt, or .pth extension required)",
          );
        } else if (pytorchFiles.length > 1) {
          // Check for index file if multiple shards
          const hasIndex = fileNames.some(
            (name) => name === "pytorch_model.bin.index.json",
          );
          if (!hasIndex) {
            warnings.push(
              "Multiple PyTorch files found but no index file detected",
            );
          }
        }
        break;
      }

      case "gguf": {
        // Check for any GGUF file
        const ggufFiles = fileNames.filter((name) => name.endsWith(".gguf"));
        if (ggufFiles.length === 0) {
          errors.push("Missing GGUF model file (.gguf extension required)");
        }

        // GGUF files are self-contained, so fewer requirements
        if (!hasConfigJson) {
          warnings.push(
            "config.json recommended for GGUF models but not strictly required",
          );
        }
        break;
      }

      default:
        errors.push(`Unsupported file format: ${fileFormat}`);
    }

    // Common file checks
    if (!hasConfigJson && fileFormat !== "gguf") {
      errors.push(
        "Missing config.json file (required for model configuration)",
      );
    }

    if (!hasTokenizerJson && !hasTokenizerConfig) {
      warnings.push(
        "Missing tokenizer files (tokenizer.json or tokenizer_config.json) - may affect text processing",
      );
    }

    // Check for other common files
    const hasVocab = fileNames.some(
      (name) =>
        name.includes("vocab") ||
        name.includes("merges") ||
        name === "special_tokens_map.json",
    );
    if (!hasVocab) {
      warnings.push(
        "Missing vocabulary files - tokenizer may not work correctly",
      );
    }

    return { errors, warnings, isValid: errors.length === 0 };
  };

  // Categorize files based on their purpose and format
  const categorizeFiles = (
    files: File[],
    fileFormat: string,
  ): { file: File; purpose: string; required: boolean }[] => {
    const categorized: { file: File; purpose: string; required: boolean }[] =
      [];

    for (const file of files) {
      const fileName = file.name.toLowerCase();
      let purpose = "";
      let required = false;
      let include = false;

      // Model files (format-specific)
      if (fileFormat === "safetensors" && fileName.endsWith(".safetensors")) {
        purpose = "Main model file (SafeTensors)";
        required = true;
        include = true;
      } else if (
        fileFormat === "pytorch" &&
        (fileName.endsWith(".bin") ||
          fileName.endsWith(".pt") ||
          fileName.endsWith(".pth"))
      ) {
        purpose = "Main model file (PyTorch)";
        required = true;
        include = true;
      } else if (fileFormat === "gguf" && fileName.endsWith(".gguf")) {
        purpose = "Main model file (GGUF)";
        required = true;
        include = true;
      }
      // Configuration files
      else if (fileName === "config.json") {
        purpose = "Model configuration";
        required = fileFormat !== "gguf";
        include = true;
      }
      // Tokenizer files
      else if (fileName === "tokenizer.json") {
        purpose = "Tokenizer configuration";
        required = false;
        include = true;
      } else if (fileName === "tokenizer_config.json") {
        purpose = "Tokenizer configuration";
        required = false;
        include = true;
      } else if (fileName === "special_tokens_map.json") {
        purpose = "Special tokens mapping";
        required = false;
        include = true;
      }
      // Vocabulary files
      else if (fileName.includes("vocab") && fileName.endsWith(".json")) {
        purpose = "Vocabulary file";
        required = false;
        include = true;
      } else if (fileName === "merges.txt") {
        purpose = "BPE merges file";
        required = false;
        include = true;
      }
      // Index files for sharded models
      else if (
        fileName === "model.safetensors.index.json" ||
        fileName === "pytorch_model.safetensors.index.json" ||
        fileName === "pytorch_model.bin.index.json"
      ) {
        purpose = "Model sharding index";
        required = false;
        include = true;
      }
      // README and other documentation
      else if (fileName === "readme.md" || fileName === "model_card.md") {
        purpose = "Documentation";
        required = false;
        include = true;
      }
      // Generation config
      else if (fileName === "generation_config.json") {
        purpose = "Generation configuration";
        required = false;
        include = true;
      }

      if (include) {
        categorized.push({ file, purpose, required });
      }
    }

    return categorized.sort((a, b) => {
      // Sort by: required first, then alphabetically by purpose
      if (a.required !== b.required) {
        return a.required ? -1 : 1;
      }
      return a.purpose.localeCompare(b.purpose);
    });
  };

  // Format file size to appropriate unit (B, KB, MB, GB)
  const formatFileSize = (bytes: number): string => {
    if (bytes === 0) return "0 B";

    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));

    if (i === 0) {
      return `${bytes} B`;
    } else if (i === 1) {
      return `${(bytes / k).toFixed(1)} KB`;
    } else {
      return `${(bytes / Math.pow(k, i)).toFixed(1)} ${sizes[i]}`;
    }
  };

  // Find main model file by extension - auto-fill the first matching file
  const findMainModelFile = (
    files: File[],
    fileFormat: string,
  ): string | null => {
    const validExtensions = {
      safetensors: [".safetensors"],
      pytorch: [".bin", ".pt", ".pth"],
      gguf: [".gguf"],
    };

    const extensions =
      validExtensions[fileFormat as keyof typeof validExtensions] || [];

    // Find the first file with a matching extension
    for (const file of files) {
      const fileName = file.name.toLowerCase();
      if (extensions.some((ext) => fileName.endsWith(ext))) {
        return file.name;
      }
    }

    return null;
  };

  return (
    <Modal
      title={t("providers.addModel")}
      open={open}
      onCancel={onClose}
      footer={[
        <Button key="cancel" onClick={onClose}>
          {t("buttons.cancel")}
        </Button>,
        <Button
          key="submit"
          type="primary"
          loading={loading}
          onClick={handleSubmit}
        >
          {t("providers.addModel")}
        </Button>,
      ]}
      width={600}
      maskClosable={false}
      destroyOnHidden={true}
    >
      <Form
        form={form}
        layout="vertical"
        initialValues={{
          file_format: "safetensors",
          model_source: "upload",
          local_folder_path: "",
          main_filename: "",
          settings: {},
        }}
      >
        <ModelParametersSection
          parameters={
            providerType === "local" ? LOCAL_MODEL_FIELDS : BASIC_MODEL_FIELDS
          }
        />

        <Form.Item
          name="file_format"
          label={t("providers.fileFormat")}
          rules={[
            {
              required: true,
              message: t("providers.fileFormatRequired"),
            },
          ]}
        >
          <Select
            placeholder={t("providers.selectFileFormat")}
            onChange={handleFileFormatChange}
            options={LOCAL_FILE_TYPE_OPTIONS.map((option) => ({
              value: option.value,
              label: option.label,
              description: option.description,
            }))}
            optionRender={(option) => (
              <div className={"flex flex-col"}>
                <Typography.Text>{option.label}</Typography.Text>
                <Typography.Text type="secondary">
                  {option.data.description}
                </Typography.Text>
              </div>
            )}
          />
        </Form.Item>

        {providerType === "local" && (
          <Form.Item
            name="model_source"
            label={t("providers.modelSource")}
            rules={[
              {
                required: true,
                message: t("providers.modelSourceRequired"),
              },
            ]}
          >
            <Radio.Group
              onChange={(e) => {
                form.setFieldValue("model_source", e.target.value);
                update(); // Force re-render to update form watchers
              }}
              value={modelSource}
            >
              <Radio value="upload">{t("providers.uploadLocal")}</Radio>
              <Radio value="repository">
                {t("providers.downloadFromRepository")}
              </Radio>
            </Radio.Group>
          </Form.Item>
        )}

        {providerType === "local" && modelSource === "upload" && (
          <>
            <Form.Item
              name="local_folder_path"
              label={t("providers.localFolderPath")}
              rules={[
                {
                  required: true,
                  message: t("providers.selectModelFolderRequired"),
                },
              ]}
            >
              <Input
                placeholder={t("providers.selectModelFolder")}
                addonBefore="📁"
                addonAfter={
                  <Upload
                    showUploadList={false}
                    beforeUpload={() => false}
                    onChange={handleFolderSelect}
                    directory
                    multiple
                  >
                    <Button
                      icon={<UploadOutlined />}
                      type={"text"}
                      size="small"
                    >
                      {t("providers.browse")}
                    </Button>
                  </Upload>
                }
              />
            </Form.Item>

            <Form.Item
              name="main_filename"
              label={t("providers.localFilename")}
              rules={[
                {
                  required: true,
                  message: t("providers.localFilenameRequired"),
                },
                {
                  validator: (_, value) => {
                    if (!value) return Promise.resolve();
                    if (validateFilename(value, selectedFileFormat)) {
                      return Promise.resolve();
                    }
                    const placeholder =
                      getFilenamePlaceholder(selectedFileFormat);
                    return Promise.reject(
                      new Error(
                        `Filename must match selected format (e.g., ${placeholder})`,
                      ),
                    );
                  },
                },
              ]}
              help={t("providers.localFilenameHelp")}
            >
              <Input placeholder={getFilenamePlaceholder(selectedFileFormat)} />
            </Form.Item>

            {/* File Preview Section */}
            {filteredFiles.length > 0 && (
              <Form.Item label="Files to Upload">
                <Card size="small">
                  <Typography.Text type="secondary">
                    {filteredFiles.length} file(s) will be uploaded:
                  </Typography.Text>
                  <List
                    size="small"
                    dataSource={filteredFiles}
                    className={"max-h-56 overflow-auto"}
                    renderItem={(item) => (
                      <List.Item>
                        <List.Item.Meta
                          title={
                            <Typography.Text ellipsis>
                              {item.file.name}
                            </Typography.Text>
                          }
                          description={
                            <Flex className={"gap-2"}>
                              <Typography.Text type="secondary">
                                {item.purpose}
                              </Typography.Text>
                              {item.required && <Tag color="red">Required</Tag>}
                            </Flex>
                          }
                        />
                        <Typography.Text type="secondary">
                          {formatFileSize(item.file.size)}
                        </Typography.Text>
                      </List.Item>
                    )}
                  />
                  <Typography.Text type="secondary">
                    Total size:{" "}
                    {formatFileSize(
                      filteredFiles.reduce(
                        (total, item) => total + item.file.size,
                        0,
                      ),
                    )}
                  </Typography.Text>
                </Card>
              </Form.Item>
            )}
          </>
        )}

        {providerType === "local" && modelSource === "repository" && (
          <>
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
                options={repositories.map((repo) => ({
                  value: repo.id,
                  label: repo.name,
                  description: repo.url,
                }))}
                optionRender={(option) => (
                  <div className="flex flex-col">
                    <Typography.Text>{option.label}</Typography.Text>
                    <Typography.Text type="secondary">
                      {option.data.description}
                    </Typography.Text>
                  </div>
                )}
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
                {
                  pattern: /^[a-zA-Z0-9_-]+\/[a-zA-Z0-9_.-]+$/,
                  message: t("providers.repositoryPathFormat"),
                },
              ]}
            >
              <Input
                placeholder="microsoft/DialoGPT-medium"
                addonBefore={
                  repositories.find((repo) => repo.id === selectedRepository)
                    ?.name === "Hugging Face Hub"
                    ? "🤗"
                    : "📁"
                }
              />
            </Form.Item>

            <Form.Item
              name="main_filename"
              label={t("providers.repositoryFilename")}
              rules={[
                {
                  required: true,
                  message: t("providers.repositoryFilenameRequired"),
                },
                {
                  validator: (_, value) => {
                    if (!value) return Promise.resolve();
                    if (validateFilename(value, selectedFileFormat)) {
                      return Promise.resolve();
                    }
                    const placeholder =
                      getFilenamePlaceholder(selectedFileFormat);
                    return Promise.reject(
                      new Error(
                        `Filename must match selected format (e.g., ${placeholder})`,
                      ),
                    );
                  },
                },
              ]}
            >
              <Input placeholder={getFilenamePlaceholder(selectedFileFormat)} />
            </Form.Item>

            <Form.Item
              name="repository_branch"
              label={t("providers.repositoryBranch")}
            >
              <Input placeholder="main" />
            </Form.Item>
          </>
        )}
      </Form>

      {/* Upload Progress */}
      {(uploading || uploadProgress.length > 0) && (
        <div className="mt-4">
          <UploadProgress
            files={uploadProgress.map((p) => ({
              filename: p.filename,
              progress: p.progress,
              status: p.status,
              error: p.error,
              size: p.size,
            }))}
            overallProgress={overallUploadProgress}
            isUploading={uploading}
            showDetails={true}
          />
        </div>
      )}

      {/* Download Progress */}
      {(downloading || downloadProgress) && (
        <div className="mt-4">
          <Card size="small">
            <Flex className="gap-2 items-center">
              <Typography.Text strong>
                Repository Download Progress
              </Typography.Text>
            </Flex>
            {downloadProgress && (
              <div className="mt-2 flex flex-col gap-1">
                <Typography.Text type="secondary">
                  {downloadProgress.message}
                </Typography.Text>
                <div className="w-full bg-gray-200 rounded-full h-2">
                  <div
                    className="bg-blue-600 h-2 rounded-full transition-all duration-300"
                    style={{
                      width: `${Math.round((downloadProgress.current / downloadProgress.total) * 100)}%`,
                    }}
                  />
                </div>
                {/*<Typography.Text type="secondary">*/}
                {/*  {downloadProgress.current}/{downloadProgress.total}*/}
                {/*</Typography.Text>*/}
              </div>
            )}
          </Card>
        </div>
      )}
    </Modal>
  );
}
