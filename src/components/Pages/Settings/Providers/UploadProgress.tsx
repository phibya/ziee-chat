import {
  CheckCircleOutlined,
  ExclamationCircleOutlined,
  LoadingOutlined,
} from "@ant-design/icons";
import { Alert, Card, List, Progress, Typography } from "antd";
import { useTranslation } from "react-i18next";

const { Text, Title } = Typography;

export interface FileUploadProgress {
  filename: string;
  progress: number;
  status: "pending" | "uploading" | "completed" | "error";
  error?: string;
  size?: number;
}

export interface UploadProgressProps {
  files: FileUploadProgress[];
  overallProgress: number;
  isUploading: boolean;
  showDetails?: boolean;
}

export function UploadProgress({
  files,
  overallProgress,
  isUploading,
  showDetails = true,
}: UploadProgressProps) {
  const { t } = useTranslation();

  const completedFiles = files.filter((f) => f.status === "completed").length;
  const errorFiles = files.filter((f) => f.status === "error").length;
  const totalFiles = files.length;

  const getStatusIcon = (status: FileUploadProgress["status"]) => {
    switch (status) {
      case "completed":
        return <CheckCircleOutlined style={{ color: "#52c41a" }} />;
      case "error":
        return <ExclamationCircleOutlined style={{ color: "#ff4d4f" }} />;
      case "uploading":
        return <LoadingOutlined style={{ color: "#1890ff" }} />;
      default:
        return null;
    }
  };

  const formatFileSize = (bytes?: number) => {
    if (!bytes) return "";
    const sizes = ["B", "KB", "MB", "GB"];
    const i = Math.floor(Math.log(bytes) / Math.log(1024));
    return `${(bytes / Math.pow(1024, i)).toFixed(1)} ${sizes[i]}`;
  };

  return (
    <Card title={t("providers.uploadProgress")} className="w-full">
      {/* Overall Progress */}
      <div className="mb-4">
        <div className="flex justify-between items-center mb-2">
          <Title level={5} className="mb-0">
            {isUploading
              ? t("providers.uploadingFiles")
              : completedFiles === totalFiles
                ? t("providers.uploadComplete")
                : t("providers.uploadStopped")}
          </Title>
        </div>

        <Progress
          percent={overallProgress}
          status={
            errorFiles > 0 ? "exception" : isUploading ? "active" : "success"
          }
          format={() => `${completedFiles}/${totalFiles} files`}
        />
      </div>

      {/* Error Summary */}
      {errorFiles > 0 && (
        <Alert
          type="warning"
          message={t("providers.uploadErrors")}
          description={t("providers.uploadErrorsDescription", {
            count: errorFiles,
          })}
          className="mb-4"
          showIcon
        />
      )}

      {/* File Details */}
      {showDetails && (
        <div>
          <Title level={5}>{t("providers.fileDetails")}</Title>
          <List
            size="small"
            dataSource={files}
            renderItem={(file) => (
              <List.Item>
                <div className="w-full">
                  <div className="flex justify-between items-center mb-1">
                    <div className="flex items-center gap-2">
                      {getStatusIcon(file.status)}
                      <Text strong className="truncate max-w-xs">
                        {file.filename}
                      </Text>
                      {file.size && (
                        <Text type="secondary" className="text-xs">
                          ({formatFileSize(file.size)})
                        </Text>
                      )}
                    </div>
                    <Text type="secondary" className="text-xs">
                      {file.status === "uploading"
                        ? `${file.progress}%`
                        : t(`providers.status.${file.status}`)}
                    </Text>
                  </div>

                  {file.status === "uploading" && (
                    <Progress
                      percent={file.progress}
                      size="small"
                      status="active"
                      showInfo={false}
                    />
                  )}

                  {file.error && (
                    <Text type="danger" className="text-xs block mt-1">
                      {file.error}
                    </Text>
                  )}
                </div>
              </List.Item>
            )}
          />
        </div>
      )}
    </Card>
  );
}
