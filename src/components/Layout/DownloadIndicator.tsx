import { DownloadOutlined } from "@ant-design/icons";
import {
  Badge,
  Button,
  Flex,
  List,
  Popover,
  Progress,
  theme,
  Typography,
} from "antd";
import { useTranslation } from "react-i18next";
import { Link } from "react-router-dom";
import { Stores } from "../../store";
import type { DownloadInstance } from "../../types/api/modelDownloads";

const { Text } = Typography;

export function DownloadIndicator() {
  const { t } = useTranslation();
  const { token } = theme.useToken();

  // Get active downloads from store
  const downloads = Object.values(Stores.ModelDownload.downloads);

  // Don't render if no active downloads
  if (downloads.length === 0) {
    return null;
  }

  const getProgressPercent = (download: DownloadInstance): number => {
    if (!download.progress_data?.current || !download.progress_data?.total) {
      return 0;
    }
    return Math.round(
      (download.progress_data.current / download.progress_data.total) * 100,
    );
  };

  // Get the first download for the button display
  const firstDownload = downloads[0];
  const firstDownloadPercent = getProgressPercent(firstDownload);

  // Create the popover content
  const popoverContent = (
    <div className="w-60">
      <Text strong className="block mb-2">
        {t("downloads.activeDownloads", {
          count: downloads.length,
          defaultValue: `${downloads.length} Active Downloads`,
        })}
      </Text>
      <List
        size="small"
        dataSource={downloads}
        className={"!p-0 !m-0"}
        renderItem={(download: DownloadInstance) => {
          const percent = getProgressPercent(download);
          return (
            <List.Item className="flex flex-col !px-0 py-2">
              <Link
                to={`/settings/providers/${download.provider_id}`}
                className="w-full cursor-pointer"
              >
                <div className="w-full">
                  <Flex justify="space-between" align="center">
                    <Text
                      ellipsis
                      className={`text-sm font-medium`}
                      style={{ maxWidth: "70%" }}
                    >
                      {download.request_data.alias}
                    </Text>
                    <Text className="text-sm font-medium">{percent}%</Text>
                  </Flex>

                  <Progress
                    percent={percent}
                    size="small"
                    status="active"
                    strokeColor={token.colorPrimary}
                    showInfo={false}
                    className={"mt-1 !leading-0"}
                  />
                </div>
              </Link>
            </List.Item>
          );
        }}
      />
    </div>
  );

  return (
    <Popover
      content={popoverContent}
      title={null}
      trigger="click"
      placement="rightTop"
    >
      <Button
        type="text"
        className={"w-full flex items-center justify-between px-3 !py-5"}
        style={{
          border: `1px solid ${token.colorBorder}`,
        }}
      >
        <Flex vertical className="w-full" gap={4}>
          <Flex align="center" justify="space-between">
            <Flex align="center" gap={8}>
              <Badge
                count={downloads.length}
                size="small"
                style={{ backgroundColor: token.colorPrimary }}
              >
                <DownloadOutlined />
              </Badge>
              <Text className="text-xs" type="secondary">
                Downloading
              </Text>
            </Flex>
            <Text className="text-xs" type="secondary">
              {firstDownloadPercent}%
            </Text>
          </Flex>

          <Progress
            percent={firstDownloadPercent}
            size="small"
            status="active"
            strokeColor={token.colorPrimary}
            showInfo={false}
            strokeWidth={3}
            className={"!leading-0"}
          />
        </Flex>
      </Button>
    </Popover>
  );
}
