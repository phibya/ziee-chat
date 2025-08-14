import { useEffect } from 'react'
import { Card, Tag, Typography, Flex, Alert, Descriptions } from 'antd'
import { LinkOutlined } from '@ant-design/icons'
import { initializeEngines, Stores } from '../../../../store'
import { SettingsPageContainer } from '../common/SettingsPageContainer'

const { Text, Link } = Typography

export function EnginesSettings() {
    const { engines, loading, error, initialized } = Stores.AdminEngines

    useEffect(() => {
        if (!initialized) {
            initializeEngines().catch(console.error)
        }
    }, [initialized])


    const getEngineLink = (engineType: string) => {
        switch (engineType) {
            case 'mistralrs': return 'https://github.com/EricLBuehler/mistral.rs'
            case 'llamacpp': return 'https://github.com/ggerganov/llama.cpp'
            default: return null
        }
    }

    if (error) {
        return (
            <SettingsPageContainer
                title="Engine Management"
                subtitle="Manage local model execution engines"
            >
                <Alert
                    message="Error Loading Engines"
                    description={error}
                    type="error"
                    showIcon
                />
            </SettingsPageContainer>
        )
    }

    return (
        <SettingsPageContainer
            title="Engine Management"
            subtitle="Manage local model execution engines"
        >
            {loading ? (
                <Card loading />
            ) : (
                <Flex vertical gap={16}>
                    {engines?.map((engine) => (
                        <Card
                            key={engine.engine_type}
                            title={
                                <Text strong style={{ fontSize: '16px' }}>
                                    {engine.name}
                                </Text>
                            }
                        >
                            <Descriptions
                                column={1}
                                size="small"
                                items={[
                                    ...(engine.version ? [{
                                        key: 'version',
                                        label: 'Version',
                                        children: engine.version,
                                    }] : []),
                                    ...(getEngineLink(engine.engine_type) ? [{
                                        key: 'link',
                                        label: 'Link',
                                        children: (
                                            <Link 
                                                href={getEngineLink(engine.engine_type) || ''} 
                                                target="_blank" 
                                                rel="noopener noreferrer"
                                            >
                                                <LinkOutlined /> {getEngineLink(engine.engine_type)}
                                            </Link>
                                        ),
                                    }] : []),
                                    ...(engine.description ? [{
                                        key: 'description',
                                        label: 'Description',
                                        children: (
                                            <Text type="secondary">
                                                {engine.description}
                                            </Text>
                                        ),
                                    }] : []),
                                    ...(engine.supported_architectures && engine.supported_architectures.length > 0 ? [{
                                        key: 'architectures',
                                        label: 'Supported Architectures',
                                        children: (
                                            <Flex gap={4} wrap="wrap">
                                                {engine.supported_architectures.map((arch) => (
                                                    <Tag key={arch}>{arch}</Tag>
                                                ))}
                                            </Flex>
                                        ),
                                    }] : []),
                                ]}
                            />
                        </Card>
                    )) || []}
                </Flex>
            )}

            {engines?.length === 0 && !loading && (
                <Alert
                    message="No Engines Available"
                    description="No local execution engines are currently available on this system."
                    type="info"
                    showIcon
                    style={{ marginTop: '24px' }}
                />
            )}
        </SettingsPageContainer>
    )
}