import { InfoCircleOutlined, RobotOutlined } from "@ant-design/icons";
import { App, Button, Card, Flex, Tag, Typography } from "antd";
import { useState } from "react";
import { useNavigate } from "react-router-dom";
import type { HubAssistant } from "../../../types/api/hub";
import { createUserAssistant } from "../../../store/assistants";
import { Drawer } from "../../common/Drawer";

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
      const newAssistant = await createUserAssistant({
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
      message.error(`Failed to create assistant: ${error.message || "Unknown error"}`);
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

      {/* Assistant Details Drawer */}
      <Drawer
        title={assistant.name}
        open={showDetails}
        onClose={handleCloseDetails}
        width={600}
        footer={[
          <Button key="close" onClick={handleCloseDetails}>
            Close
          </Button>,
          <Button
            key="use"
            type="primary"
            icon={<RobotOutlined />}
            onClick={() => {
              handleUseAssistant(assistant);
              handleCloseDetails();
            }}
            loading={isCreating}
            disabled={isCreating}
          >
            {isCreating ? "Creating..." : "Use Assistant"}
          </Button>,
        ]}
      >
        <Flex vertical className="gap-4">
          {/* Basic Info */}
          <div>
            <Title level={5}>Description</Title>
            <Text>{assistant.description || "No description available"}</Text>
          </div>

          {/* Instructions */}
          {assistant.instructions && (
            <div>
              <Title level={5}>Instructions</Title>
              <Text className="whitespace-pre-wrap">{assistant.instructions}</Text>
            </div>
          )}

          {/* Category & Author */}
          <div>
            <Title level={5}>Details</Title>
            <Flex vertical className="gap-2">
              <Flex justify="space-between">
                <Text type="secondary">Category:</Text>
                <Tag color="geekblue">{assistant.category}</Tag>
              </Flex>
              {assistant.author && (
                <Flex justify="space-between">
                  <Text type="secondary">Author:</Text>
                  <Text>{assistant.author}</Text>
                </Flex>
              )}
              {assistant.popularity_score && (
                <Flex justify="space-between">
                  <Text type="secondary">Popularity:</Text>
                  <Text>{assistant.popularity_score}</Text>
                </Flex>
              )}
            </Flex>
          </div>

          {/* Tags */}
          <div>
            <Title level={5}>Tags</Title>
            <Flex wrap className="gap-1">
              {assistant.tags.map((tag) => (
                <Tag key={tag} color="default">
                  {tag}
                </Tag>
              ))}
            </Flex>
          </div>

          {/* Recommended Models */}
          {assistant.recommended_models.length > 0 && (
            <div>
              <Title level={5}>Recommended Models</Title>
              <Flex wrap className="gap-1">
                {assistant.recommended_models.map((model) => (
                  <Tag key={model} color="green">
                    {model}
                  </Tag>
                ))}
              </Flex>
            </div>
          )}

          {/* Required Capabilities */}
          {assistant.capabilities_required.length > 0 && (
            <div>
              <Title level={5}>Required Capabilities</Title>
              <Flex wrap className="gap-1">
                {assistant.capabilities_required.map((capability) => (
                  <Tag key={capability} color="orange">
                    {capability}
                  </Tag>
                ))}
              </Flex>
            </div>
          )}

          {/* Use Cases */}
          {assistant.use_cases && assistant.use_cases.length > 0 && (
            <div>
              <Title level={5}>Use Cases</Title>
              <ul className="ml-4">
                {assistant.use_cases.map((useCase, index) => (
                  <li key={index}>
                    <Text>{useCase}</Text>
                  </li>
                ))}
              </ul>
            </div>
          )}

          {/* Example Prompts */}
          {assistant.example_prompts && assistant.example_prompts.length > 0 && (
            <div>
              <Title level={5}>Example Prompts</Title>
              <Flex vertical className="gap-2">
                {assistant.example_prompts.map((prompt, index) => (
                  <Card key={index} size="small" className="bg-gray-50">
                    <Text className="text-sm italic">"{prompt}"</Text>
                  </Card>
                ))}
              </Flex>
            </div>
          )}

          {/* Parameters */}
          {assistant.parameters && Object.keys(assistant.parameters).length > 0 && (
            <div>
              <Title level={5}>Parameters</Title>
              <Card size="small" className="bg-gray-50">
                <pre className="text-xs overflow-auto">
                  {JSON.stringify(assistant.parameters, null, 2)}
                </pre>
              </Card>
            </div>
          )}
        </Flex>
      </Drawer>
    </>
  );
}