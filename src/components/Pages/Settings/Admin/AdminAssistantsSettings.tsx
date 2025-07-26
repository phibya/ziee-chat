import {
  DeleteOutlined,
  EditOutlined,
  PlusOutlined,
  RobotOutlined,
} from "@ant-design/icons";
import {
  App,
  Button,
  Card,
  Flex,
  Popconfirm,
  Table,
  Tag,
  Tooltip,
  Typography,
} from "antd";
import React, { useEffect } from "react";
import { useTranslation } from "react-i18next";
import {
  clearSystemAdminError,
  deleteSystemAdminAssistant,
  loadSystemAdminAssistants,
  openAssistantDrawer,
  Stores,
} from "../../../../store";
import { Assistant } from "../../../../types/api/assistant";
import { SettingsPageContainer } from '../SettingsPageContainer';
import { AssistantFormDrawer } from "../../../shared/AssistantFormDrawer.tsx";

const { Text } = Typography;

export const AdminAssistantsSettings: React.FC = () => {
  const { t } = useTranslation();
  const { message } = App.useApp();

  // Admin store
  const { assistants, loading, error } = Stores.Admin;

  useEffect(() => {
    loadSystemAdminAssistants();
  }, []);

  // Show errors
  useEffect(() => {
    if (error) {
      message.error(error);
      clearSystemAdminError();
    }
  }, [error, message]);

  const handleDelete = async (assistant: Assistant) => {
    try {
      await deleteSystemAdminAssistant(assistant.id);
      message.success("Assistant deleted successfully");
    } catch (error) {
      console.error("Failed to delete assistant:", error);
      // Error is handled by the store
    }
  };

  const handleEdit = (assistant: Assistant) => {
    openAssistantDrawer(assistant, true);
  };

  const handleCreate = () => {
    openAssistantDrawer(undefined, true);
  };

  const columns = [
    {
      title: t("labels.name"),
      dataIndex: "name",
      key: "name",
      render: (text: string, record: Assistant) => (
        <Flex className="gap-2">
          <RobotOutlined />
          <Text strong>{text}</Text>
          {record.is_default && <Tag color="green">Default</Tag>}
          {!record.is_active && <Tag color="red">Inactive</Tag>}
        </Flex>
      ),
    },
    {
      title: t("labels.description"),
      dataIndex: "description",
      key: "description",
      render: (text: string) => (
        <Text type="secondary">{text || "No description"}</Text>
      ),
    },
    {
      title: t("admin.assistants.createdBy"),
      dataIndex: "created_by",
      key: "created_by",
      render: (userId: string) => (
        <Text type="secondary">{userId ? "User" : "System"}</Text>
      ),
    },
    {
      title: t("labels.created"),
      dataIndex: "created_at",
      key: "created_at",
      render: (date: string) => new Date(date).toLocaleDateString(),
    },
    {
      title: t("labels.actions"),
      key: "actions",
      render: (_: any, record: Assistant) => (
        <Flex className="gap-2">
          <Tooltip title={t("buttons.edit")}>
            <Button
              type="text"
              icon={<EditOutlined />}
              onClick={() => handleEdit(record)}
            />
          </Tooltip>
          <Popconfirm
            title={t("assistants.deleteAssistant")}
            description={t("assistants.deleteConfirm")}
            onConfirm={() => handleDelete(record)}
            okText="Yes"
            cancelText="No"
          >
            <Tooltip title={t("buttons.delete")}>
              <Button type="text" danger icon={<DeleteOutlined />} />
            </Tooltip>
          </Popconfirm>
        </Flex>
      ),
    },
  ];

  return (
    <SettingsPageContainer 
      title="Assistants"
      subtitle="Manage template assistants. Default assistants are automatically cloned for new users."
    >
      <div>
        <div className="flex justify-end items-center mb-6">
          <Button type="primary" icon={<PlusOutlined />} onClick={handleCreate}>
            Create Assistant
          </Button>
        </div>

        <Card>
          <Table
            columns={columns}
            dataSource={assistants}
            loading={loading}
            rowKey="id"
            pagination={{ pageSize: 10 }}
          />
        </Card>

        <AssistantFormDrawer />
      </div>
    </SettingsPageContainer>
  );
};
