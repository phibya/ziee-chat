import {Card, List, Button, Tag, Typography, Space, Avatar, Progress} from 'antd'
import {DownloadOutlined, DatabaseOutlined, CheckCircleOutlined, LoadingOutlined} from '@ant-design/icons'

const {Title, Text} = Typography

const mockLocalModels = [
    {
        id: 'llama-2-7b-chat',
        name: 'Llama 2 7B Chat',
        description: 'Meta\'s conversational AI model',
        provider: 'Meta',
        size: '3.8 GB',
        status: 'downloaded',
        downloadProgress: 100,
        capabilities: ['Chat', 'Instruction Following'],
    },
    {
        id: 'mistral-7b-instruct',
        name: 'Mistral 7B Instruct',
        description: 'High-performance instruction-following model',
        provider: 'Mistral AI',
        size: '4.1 GB',
        status: 'downloading',
        downloadProgress: 65,
        capabilities: ['Chat', 'Code Generation'],
    },
    {
        id: 'codellama-7b',
        name: 'Code Llama 7B',
        description: 'Specialized model for code generation',
        provider: 'Meta',
        size: '3.9 GB',
        status: 'available',
        downloadProgress: 0,
        capabilities: ['Code Generation', 'Code Completion'],
    },
    {
        id: 'phi-3-mini',
        name: 'Phi-3 Mini',
        description: 'Small but capable model from Microsoft',
        provider: 'Microsoft',
        size: '2.2 GB',
        status: 'downloaded',
        downloadProgress: 100,
        capabilities: ['Chat', 'Reasoning'],
    },
]

export function ModelsPage() {

    const handleDownload = (modelId: string) => {
        console.log('Downloading model:', modelId)
        // In a real app, this would trigger model download
    }

    const handleRemove = (modelId: string) => {
        console.log('Removing model:', modelId)
        // In a real app, this would remove the downloaded model
    }

    const getStatusColor = (status: string) => {
        switch (status) {
            case 'downloaded':
                return 'green'
            case 'downloading':
                return 'orange'
            case 'available':
                return 'blue'
            case 'error':
                return 'red'
            default:
                return 'default'
        }
    }

    const getStatusIcon = (status: string) => {
        switch (status) {
            case 'downloaded':
                return <CheckCircleOutlined/>
            case 'downloading':
                return <LoadingOutlined spin/>
            case 'available':
                return <DownloadOutlined/>
            default:
                return <DownloadOutlined/>
        }
    }

    const renderModelActions = (model: any) => {
        if (model.status === 'downloaded') {
            return (
                <Space>
                    <Button type="primary" size="small">
                        Use Model
                    </Button>
                    <Button size="small" onClick={() => handleRemove(model.id)}>
                        Remove
                    </Button>
                </Space>
            )
        } else if (model.status === 'downloading') {
            return (
                <div style={{width: '120px'}}>
                    <Progress
                        percent={model.downloadProgress}
                        size="small"
                        status="active"
                    />
                    <Text type="secondary" style={{fontSize: '12px'}}>
                        {model.downloadProgress}% complete
                    </Text>
                </div>
            )
        } else {
            return (
                <Button
                    type="primary"
                    icon={<DownloadOutlined/>}
                    size="small"
                    onClick={() => handleDownload(model.id)}
                >
                    Download
                </Button>
            )
        }
    }

    return (
        <div style={{padding: '24px', height: '100%', overflow: 'auto'}}>
            <div style={{marginBottom: '24px'}}>
                <Title level={2}>
                    <DatabaseOutlined style={{marginRight: '8px'}}/>
                    Local Models
                </Title>
                <Text type="secondary">
                    Manage your downloaded AI models for local inference
                </Text>
            </div>

            <div style={{marginBottom: '16px'}}>
                <Space>
                    <Text strong>Storage Used: </Text>
                    <Text>14.0 GB / 50.0 GB</Text>
                    <Progress percent={28} showInfo={false} style={{width: '200px'}}/>
                </Space>
            </div>

            <List
                grid={{gutter: 16, xs: 1, sm: 1, md: 2, lg: 2, xl: 3}}
                dataSource={mockLocalModels}
                renderItem={(model) => (
                    <List.Item>
                        <Card
                            hoverable
                            style={{height: '100%'}}
                            actions={[renderModelActions(model)]}
                        >
                            <Card.Meta
                                avatar={<Avatar size="large" icon={<DatabaseOutlined/>}/>}
                                title={
                                    <Space>
                                        {model.name}
                                        <Tag
                                            color={getStatusColor(model.status)}
                                            icon={getStatusIcon(model.status)}
                                        >
                                            {model.status}
                                        </Tag>
                                    </Space>
                                }
                                description={
                                    <div>
                                        <Text style={{marginBottom: '8px', display: 'block'}}>
                                            {model.description}
                                        </Text>
                                        <div style={{marginBottom: '8px'}}>
                                            <Text type="secondary">Provider: </Text>
                                            <Text strong>{model.provider}</Text>
                                        </div>
                                        <div style={{marginBottom: '8px'}}>
                                            <Text type="secondary">Size: </Text>
                                            <Text code>{model.size}</Text>
                                        </div>
                                        <div>
                                            <Text type="secondary">Capabilities: </Text>
                                            <Space wrap>
                                                {model.capabilities.map((capability) => (
                                                    <Tag key={capability}>
                                                        {capability}
                                                    </Tag>
                                                ))}
                                            </Space>
                                        </div>
                                    </div>
                                }
                            />
                        </Card>
                    </List.Item>
                )}
            />
        </div>
    )
}