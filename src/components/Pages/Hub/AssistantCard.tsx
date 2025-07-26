import { InfoCircleOutlined, RobotOutlined } from "@ant-design/icons";
import { App, Button, Card, Flex, Tag, Typography } from "antd";
import { useState } from "react";
import { useNavigate } from "react-router-dom";
import type { HubAssistant } from "../../../types/api/hub";
import { createUserAssistant } from "../../../store/assistants";
import { AssistantDetailsDrawer } from "./AssistantDetailsDrawer";

const { Title, Text } = Typography;

interface AssistantCardProps {
  assistant: HubAssistant;
}

export function AssistantCard({ assistant }: AssistantCardProps) {
  const { message } = App.useApp();
  const [showDetails, setShowDetails] = useState(false);
  const [isCreating, setIsCreating] = useState(false);
  const navigate = useNavigate();

  const handleUseAssistant = async (assistant: HubAssistant) => {
    setIsCreating(true);
    try {
      // Create a user assistant based on the hub assistant
      await createUserAssistant({
        name: assistant.name,
        description: assistant.description,
        instructions: assistant.instructions,
        parameters: assistant.parameters || { stream: true },
        is_active: true,
      });

      message.success(`Assistant "${assistant.name}" created successfully!`);

      // Navigate to assistants page to show the newly created assistant
      navigate("/assistants");
    } catch (error: any) {
      console.error("Failed to create assistant:", error);
      message.error(
        `Failed to create assistant: ${error.message || "Unknown error"}`,
      );
    } finally {
      setIsCreating(false);
    }
  };

  const handleShowDetails = () => {
    setShowDetails(true);
  };

  const handleCloseDetails = () => {
    setShowDetails(false);
  };

  return (
    <>
      <Card
        key={assistant.id}
        hoverable
        className="h-full"
        styles={{ body: { padding: "16px" } }}
        onClick={handleShowDetails}
      >
        <div className="mb-3">
          <Title level={4} className="m-0 mb-1">
            {assistant.name}
          </Title>
          <Text type="secondary" className="text-xs">
            {assistant.description}
          </Text>
        </div>

        {/* Category & Author */}
        <div className="mb-3">
          <Flex justify="space-between" align="center">
            <Tag color="geekblue" className="text-xs">
              {assistant.category}
            </Tag>
            {assistant.author && (
              <Text type="secondary" className="text-xs">
                by {assistant.author}
              </Text>
            )}
          </Flex>
        </div>

        {/* Tags */}
        <div className="mb-3">
          <Flex wrap className="gap-1">
            {assistant.tags.slice(0, 3).map((tag) => (
              <Tag key={tag} color="default" className="text-xs">
                {tag}
              </Tag>
            ))}
            {assistant.tags.length > 3 && (
              <Tag color="default" className="text-xs">
                +{assistant.tags.length - 3}
              </Tag>
            )}
          </Flex>
        </div>

        {/* Recommended Models */}
        {assistant.recommended_models.length > 0 && (
          <div className="mb-3">
            <Text type="secondary" className="text-xs">
              Works best with:{" "}
              {assistant.recommended_models.slice(0, 2).join(", ")}
              {assistant.recommended_models.length > 2 && "..."}
            </Text>
          </div>
        )}

        {/* Action Buttons */}
        <div className="mt-auto flex gap-2">
          <Button
            size="small"
            icon={<InfoCircleOutlined />}
            onClick={(e) => {
              e.stopPropagation();
              handleShowDetails();
            }}
            className="flex-1"
          >
            Details
          </Button>
          <Button
            type="primary"
            size="small"
            icon={<RobotOutlined />}
            onClick={(e) => {
              e.stopPropagation();
              handleUseAssistant(assistant);
            }}
            className="flex-[2]"
            loading={isCreating}
            disabled={isCreating}
          >
            {isCreating ? "Creating..." : "Use Assistant"}
          </Button>
        </div>
      </Card>

      <AssistantDetailsDrawer
        assistant={assistant}
        open={showDetails}
        onClose={handleCloseDetails}
      />
    </>
  );
}
