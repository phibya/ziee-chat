import {
  AppstoreOutlined,
  ClearOutlined,
  DownloadOutlined,
  EyeOutlined,
  FileTextOutlined,
  LockOutlined,
  SearchOutlined,
  ToolOutlined,
  UnlockOutlined,
} from "@ant-design/icons";
import { App, Button, Card, Flex, Input, Select, Tag, Typography } from "antd";
import { useEffect, useMemo, useState } from "react";
import { searchModels, useHubStore } from "../../../store/hub";
import type { HubModel } from "../../../types/api/hub";
import { openUrl } from "@tauri-apps/plugin-opener";
import { isDesktopApp } from "../../../api/core.ts";
import { loadAllModelRepositories, Stores } from "../../../store";
import { repositoryHasCredentials } from "../../../store/repositories.ts";
import { openRepositoryDrawer } from "../../../store/ui";
import { RepositoryDrawer } from "../Settings/ModelRepositorySettings/RepositoryDrawer.tsx";

const { Title, Text } = Typography;

export function ModelsTab() {
  const { models } = useHubStore();
  const { message } = App.useApp();
  const [searchTerm, setSearchTerm] = useState("");
  const [selectedTags, setSelectedTags] = useState<string[]>([]);
  const [selectedCapabilities, setSelectedCapabilities] = useState<string[]>(
    [],
  );
  const [sortBy, setSortBy] = useState("popular");
  const { repositories } = Stores.Repositories;

  useEffect(() => {
    loadAllModelRepositories();
  }, []);

  const handleDownload = async (model: HubModel) => {
    console.log("Downloading model:", model.id);
    const repo = repositories.find((repo) => repo.url === model.repository_url);
    console.log({ repo });
    if (!repo) {
      message.error(
        `Repository not found for model ${model.alias}. Please check the repository configuration.`,
      );
      return;
    }

    if (!model.public && !repositoryHasCredentials(repo)) {
      message.info(
        `Model ${model.alias} is private and requires credentials. Please configure the repository with valid credentials.`,
      );

      openRepositoryDrawer(repo);

      return;
    }
  };

  const handleViewReadme = (model: HubModel) => {
    // Construct the README URL based on repository type
    const constructReadmeUrl = (model: HubModel): string => {
      const baseUrl = model.repository_url.replace(/\/$/, "");
      const repoPath = model.repository_path;

      if (baseUrl.startsWith("https://github.com")) {
        return `${baseUrl}/${repoPath}/blob/main/README.md`;
      } else if (baseUrl.startsWith("https://huggingface.co")) {
        return `${baseUrl}/${repoPath}/blob/main/README.md`;
      } else {
        // Fallback to the repository URL itself
        return `${baseUrl}/${repoPath}`;
      }
    };

    const readmeUrl = constructReadmeUrl(model);
    if (isDesktopApp) {
      openUrl(readmeUrl).catch((err) => {
        console.error(`Failed to open ${readmeUrl}:`, err);
        message.error(`Failed to open ${readmeUrl}`);
      });
    } else {
      window.open(readmeUrl, "_blank", "noopener,noreferrer");
    }
  };

  const clearAllFilters = () => {
    setSearchTerm("");
    setSelectedTags([]);
    setSelectedCapabilities([]);
  };

  // Get unique tags and capabilities for filters
  const modelTags = useMemo(() => {
    const allTags = new Set<string>();
    models.forEach((model) => {
      model.tags.forEach((tag) => allTags.add(tag));
    });
    return Array.from(allTags).sort();
  }, [models]);

  const modelCapabilities = useMemo(() => {
    const capabilities: { key: string; label: string }[] = [];
    const hasVision = models.some((m) => m.capabilities?.vision);
    const hasTools = models.some((m) => m.capabilities?.tools);
    const hasCode = models.some((m) => m.capabilities?.code_interpreter);
    const hasAudio = models.some((m) => m.capabilities?.audio);

    if (hasVision) capabilities.push({ key: "vision", label: "Vision" });
    if (hasTools) capabilities.push({ key: "tools", label: "Tools" });
    if (hasCode)
      capabilities.push({ key: "code_interpreter", label: "Code Interpreter" });
    if (hasAudio) capabilities.push({ key: "audio", label: "Audio" });

    return capabilities;
  }, [models]);

  const filteredModels = useMemo(() => {
    let filtered = searchModels(models, searchTerm);

    // Filter by tags
    if (selectedTags.length > 0) {
      filtered = filtered.filter((model) =>
        selectedTags.some((tag) => model.tags.includes(tag)),
      );
    }

    // Filter by capabilities
    if (selectedCapabilities.length > 0) {
      filtered = filtered.filter((model) => {
        if (!model.capabilities) return false;
        return selectedCapabilities.some((capability) => {
          switch (capability) {
            case "vision":
              return model.capabilities?.vision || false;
            case "tools":
              return model.capabilities?.tools || false;
            case "code_interpreter":
              return model.capabilities?.code_interpreter || false;
            case "audio":
              return model.capabilities?.audio || false;
            default:
              return false;
          }
        });
      });
    }

    // Sort models
    switch (sortBy) {
      case "popular":
        filtered.sort(
          (a, b) => (b.popularity_score || 0) - (a.popularity_score || 0),
        );
        break;
      case "name":
        filtered.sort((a, b) => a.name.localeCompare(b.name));
        break;
      case "size":
        filtered.sort((a, b) => a.size_gb - b.size_gb);
        break;
      default:
        break;
    }

    return filtered;
  }, [models, searchTerm, selectedTags, selectedCapabilities, sortBy]);

  return (
    <>
      {/* Search and Filters */}
      <div className="mb-6">
        <Flex wrap gap={16} className="mb-4">
          <div className="flex-1 min-w-[200px] basis-[300px]">
            <Input
              placeholder="Search models..."
              prefix={<SearchOutlined />}
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
              allowClear
            />
          </div>
          <div className="flex-1 min-w-[150px] basis-[200px]">
            <Select
              mode="multiple"
              placeholder="Filter by tags"
              value={selectedTags}
              onChange={setSelectedTags}
              className="w-full"
              allowClear
              maxTagCount="responsive"
            >
              {modelTags.map((tag) => (
                <Select.Option key={tag} value={tag}>
                  {tag}
                </Select.Option>
              ))}
            </Select>
          </div>
          <div className="flex-1 min-w-[150px] basis-[200px]">
            <Select
              mode="multiple"
              placeholder="Capabilities"
              value={selectedCapabilities}
              onChange={setSelectedCapabilities}
              className="w-full"
              allowClear
              maxTagCount="responsive"
            >
              {modelCapabilities.map((capability) => (
                <Select.Option key={capability.key} value={capability.key}>
                  {capability.label}
                </Select.Option>
              ))}
            </Select>
          </div>
          <div className="flex-1 min-w-[120px] basis-[150px]">
            <Select
              placeholder="Sort by"
              value={sortBy}
              onChange={setSortBy}
              className="w-full"
            >
              <Select.Option value="popular">Popular</Select.Option>
              <Select.Option value="name">Name</Select.Option>
              <Select.Option value="size">Size</Select.Option>
            </Select>
          </div>
        </Flex>
        {(searchTerm ||
          selectedTags.length > 0 ||
          selectedCapabilities.length > 0) && (
          <Flex align="center" gap={8}>
            <Text type="secondary" className="text-xs">
              Filters active:{" "}
              {[
                searchTerm && "search",
                selectedTags.length > 0 && `${selectedTags.length} tags`,
                selectedCapabilities.length > 0 &&
                  `${selectedCapabilities.length} capabilities`,
              ]
                .filter(Boolean)
                .join(", ")}
            </Text>
            <Button
              size="small"
              type="text"
              icon={<ClearOutlined />}
              onClick={clearAllFilters}
            >
              Clear all
            </Button>
          </Flex>
        )}
      </div>

      {/* Models Grid */}
      <div className="grid grid-cols-[repeat(auto-fill,minmax(300px,1fr))] gap-4">
        {filteredModels.map((model) => (
          <Card
            key={model.id}
            hoverable
            className="h-full"
            styles={{ body: { padding: "16px" } }}
          >
            <div className="mb-3">
              <Flex justify="space-between" align="start" className="mb-2">
                <Title level={4} className="m-0">
                  {model.alias}
                </Title>
                {model.public ? (
                  <UnlockOutlined className="text-green-500" />
                ) : (
                  <LockOutlined className="text-red-500" />
                )}
              </Flex>
              <Text type="secondary" className="text-xs">
                {model.description}
              </Text>
            </div>

            {/* Tags */}
            <div className="mb-3">
              <Flex wrap className="gap-1">
                {model.tags.slice(0, 3).map((tag) => (
                  <Tag key={tag} color="default" className="text-xs">
                    {tag}
                  </Tag>
                ))}
                {model.tags.length > 3 && (
                  <Tag color="default" className="text-xs">
                    +{model.tags.length - 3}
                  </Tag>
                )}
              </Flex>
            </div>

            {/* Capabilities */}
            {model.capabilities && (
              <div className="mb-3">
                <Flex wrap className="gap-1">
                  {model.capabilities.vision && (
                    <Tag
                      color="purple"
                      icon={<EyeOutlined />}
                      className="text-xs"
                    >
                      Vision
                    </Tag>
                  )}
                  {model.capabilities.tools && (
                    <Tag
                      color="blue"
                      icon={<ToolOutlined />}
                      className="text-xs"
                    >
                      Tools
                    </Tag>
                  )}
                  {model.capabilities.code_interpreter && (
                    <Tag
                      color="orange"
                      icon={<AppstoreOutlined />}
                      className="text-xs"
                    >
                      Code
                    </Tag>
                  )}
                </Flex>
              </div>
            )}

            {/* Stats */}
            <div className="mb-3">
              <Flex justify="space-between" align="center" className="mb-1">
                <Text type="secondary" className="text-xs">
                  Size: {model.size_gb}GB
                </Text>
                <Text type="secondary" className="text-xs">
                  {model.file_format.toUpperCase()}
                </Text>
              </Flex>
              {model.license && (
                <Text type="secondary" className="text-xs">
                  License: {model.license}
                </Text>
              )}
            </div>

            {/* Action Buttons */}
            <div className="mt-auto flex gap-2">
              <Button
                size="small"
                icon={<FileTextOutlined />}
                onClick={() => handleViewReadme(model)}
                className="flex-1"
              >
                README
              </Button>
              <Button
                type="primary"
                size="small"
                icon={<DownloadOutlined />}
                onClick={() => handleDownload(model)}
                className="flex-[2]"
              >
                Download
              </Button>
            </div>
          </Card>
        ))}
      </div>

      {filteredModels.length === 0 && (
        <div className="text-center py-12">
          <Text type="secondary">No models found</Text>
        </div>
      )}

      <RepositoryDrawer />
    </>
  );
}
