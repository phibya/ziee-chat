import {useEffect} from 'react'
import {Card, Form, Input, Select, Switch, Button, Typography, InputNumber, message} from 'antd'
import {SettingOutlined, SaveOutlined} from '@ant-design/icons'
import {useTranslation} from 'react-i18next'
import {useSettingsStore} from '../../store/settings'
import {useTheme} from '../../hooks/useTheme'

const {Title, Text} = Typography
const {Option} = Select

export function SettingsPage() {
    const {t} = useTranslation()
    const appTheme = useTheme()
    const [form] = Form.useForm()
    const {
        theme,
        language,
        componentSize,
        autoSave,
        showTimestamps,
        maxTokens,
        temperature,
        openaiApiKey,
        anthropicApiKey,
        customEndpoint,
        requestTimeout,
        enableStreaming,
        enableFunctionCalling,
        debugMode,
        systemPrompt,
        defaultModel,
        updateSettings
    } = useSettingsStore()

    // Initialize form with current settings
    useEffect(() => {
        form.setFieldsValue({
            theme,
            language,
            componentSize,
            autoSave,
            showTimestamps,
            maxTokens,
            temperature,
            openaiApiKey,
            anthropicApiKey,
            customEndpoint,
            requestTimeout,
            enableStreaming,
            enableFunctionCalling,
            debugMode,
            systemPrompt,
            defaultModel,
        })
    }, [form, theme, language, componentSize, autoSave, showTimestamps, maxTokens, temperature, openaiApiKey,
        anthropicApiKey, customEndpoint, requestTimeout, enableStreaming, enableFunctionCalling, debugMode, systemPrompt, defaultModel])

    const handleSave = () => {
        form.validateFields().then((values) => {
            updateSettings(values)
            message.success(t('common.success'))
        })
    }

    return (
        <div className="p-4 sm:p-6 h-full overflow-auto">
            <div className="mb-4 sm:mb-6">
                <Title level={2}>
                    <SettingOutlined className="mr-2"/>
                    {t('settings.title')}
                </Title>
                <Text type="secondary">{t('settings.description')}</Text>
            </div>

            <Form form={form} layout="vertical" className="flex flex-col gap-3 sm:gap-4">
                <Card title={t('settings.general.title')}>
                    <Form.Item
                        label={t('settings.general.defaultModel')}
                        name="defaultModel"
                    >
                        <Select>
                            <Option value="gpt-3.5-turbo">GPT-3.5 Turbo</Option>
                            <Option value="gpt-4">GPT-4</Option>
                            <Option value="claude-3-sonnet">Claude 3 Sonnet</Option>
                            <Option value="llama-2-7b">Llama 2 7B</Option>
                        </Select>
                    </Form.Item>

                    <Form.Item
                        label={t('settings.general.theme')}
                        name="theme"
                    >
                        <Select>
                            <Option value="light">{t('settings.general.themes.light')}</Option>
                            <Option value="dark">{t('settings.general.themes.dark')}</Option>
                            <Option value="auto">{t('settings.general.themes.auto')}</Option>
                        </Select>
                    </Form.Item>

                    <Form.Item
                        label={t('settings.general.language')}
                        name="language"
                    >
                        <Select>
                            <Option value="en">{t('settings.general.languages.en')}</Option>
                            <Option value="vi">{t('settings.general.languages.vi')}</Option>
                        </Select>
                    </Form.Item>

                    <Form.Item
                        label={t('settings.general.componentSize')}
                        name="componentSize"
                    >
                        <Select>
                            <Option value="small">{t('settings.general.componentSizes.small')}</Option>
                            <Option value="middle">{t('settings.general.componentSizes.middle')}</Option>
                            <Option value="large">{t('settings.general.componentSizes.large')}</Option>
                        </Select>
                    </Form.Item>
                </Card>

                <Card title={t('settings.chat.title')} className="mb-4">
                    <Form.Item
                        label={t('settings.chat.autoSave')}
                        name="autoSave"
                        valuePropName="checked"
                    >
                        <Switch/>
                    </Form.Item>

                    <Form.Item
                        label={t('settings.chat.showTimestamps')}
                        name="showTimestamps"
                        valuePropName="checked"
                    >
                        <Switch/>
                    </Form.Item>

                    <Form.Item
                        label={t('settings.chat.maxTokens')}
                        name="maxTokens"
                    >
                        <InputNumber
                            min={100}
                            max={4096}
                            className="w-full"
                            placeholder={t('settings.chat.maxTokensPlaceholder')}
                        />
                    </Form.Item>

                    <Form.Item
                        label={t('settings.chat.temperature')}
                        name="temperature"
                    >
                        <InputNumber
                            min={0}
                            max={2}
                            step={0.1}
                            className="w-full"
                            placeholder={t('settings.chat.temperaturePlaceholder')}
                        />
                    </Form.Item>
                </Card>

                <Card title="API Settings" className="mb-4">
                    <Form.Item
                        label="OpenAI API Key"
                        name="openaiApiKey"
                    >
                        <Input.Password placeholder="Enter your OpenAI API key"/>
                    </Form.Item>

                    <Form.Item
                        label="Anthropic API Key"
                        name="anthropicApiKey"
                    >
                        <Input.Password placeholder="Enter your Anthropic API key"/>
                    </Form.Item>

                    <Form.Item
                        label="Custom API Endpoint"
                        name="customEndpoint"
                    >
                        <Input placeholder="https://api.example.com/v1"/>
                    </Form.Item>

                    <Form.Item
                        label="Request Timeout (seconds)"
                        name="requestTimeout"
                        initialValue={30}
                    >
                        <InputNumber
                            min={5}
                            max={300}
                            className="w-full"
                        />
                    </Form.Item>
                </Card>

                <Card title="Advanced Settings" className="mb-4">
                    <Form.Item
                        label="Enable Streaming"
                        name="enableStreaming"
                        valuePropName="checked"
                        initialValue={true}
                    >
                        <Switch/>
                    </Form.Item>

                    <Form.Item
                        label="Enable Function Calling"
                        name="enableFunctionCalling"
                        valuePropName="checked"
                        initialValue={true}
                    >
                        <Switch/>
                    </Form.Item>

                    <Form.Item
                        label="Debug Mode"
                        name="debugMode"
                        valuePropName="checked"
                        initialValue={false}
                    >
                        <Switch/>
                    </Form.Item>

                    <Form.Item
                        label="System Prompt"
                        name="systemPrompt"
                    >
                        <Input.TextArea
                            rows={4}
                            placeholder="Enter custom system prompt (optional)"
                        />
                    </Form.Item>
                </Card>

                <div className="text-right">
                    <Button type="primary" icon={<SaveOutlined/>} onClick={handleSave}>
                        {t('settings.save')}
                    </Button>
                </div>
            </Form>
        </div>
    )
}