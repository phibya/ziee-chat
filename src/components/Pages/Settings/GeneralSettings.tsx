import {Button, Switch, Typography} from 'antd'
import {FolderOpenOutlined, FileTextOutlined} from '@ant-design/icons'

const {Title, Text} = Typography

export function GeneralSettings() {
    return (
        <div className="space-y-6">
            <div className="mb-6">
                <Title level={3} className="mb-0">General</Title>
            </div>

            <div className="border-b pb-4">
                <div className="flex justify-between items-center">
                    <div>
                        <Text strong>App Version</Text>
                        <div className="text-sm text-gray-500">v0.6.4</div>
                    </div>
                    <div className="text-right">
                        <Text className="text-sm text-gray-500">v0.6.4</Text>
                    </div>
                </div>
            </div>

            <div className="border-b pb-4">
                <div className="flex justify-between items-center">
                    <div>
                        <Text strong>Check for Updates</Text>
                        <div className="text-sm text-gray-500">Check if a newer version of Jan is available.</div>
                    </div>
                    <Button type="default">Check for Updates</Button>
                </div>
            </div>

            <div className="mt-6">
                <Title level={4} className="mb-4">Advanced</Title>
                <div className="border-b pb-4">
                    <div className="flex justify-between items-center">
                        <div>
                            <Text strong>Experimental Features</Text>
                            <div className="text-sm text-gray-500">Enable experimental features. They may be unstable or
                                change at any time.
                            </div>
                        </div>
                        <Switch size="small"/>
                    </div>
                </div>
            </div>

            <div className="mt-6">
                <Title level={4} className="mb-4">Data Folder</Title>
                <div className="border-b pb-4">
                    <div className="flex justify-between items-center">
                        <div>
                            <Text strong>App Data</Text>
                            <div className="text-sm text-gray-500">Default location for messages and other user data.
                            </div>
                            <div className="text-xs text-gray-400">/Users/royal/Library/Application Support/Jan/data
                            </div>
                        </div>
                        <Button type="default" icon={<FolderOpenOutlined/>}>Change Location</Button>
                    </div>
                </div>
                <div className="border-b pb-4 mt-4">
                    <div className="flex justify-between items-center">
                        <div>
                            <Text strong>App Logs</Text>
                            <div className="text-sm text-gray-500">View detailed logs of the App.</div>
                        </div>
                        <div className="space-x-2">
                            <Button type="default" icon={<FileTextOutlined/>}>Open Logs</Button>
                            <Button type="default" icon={<FolderOpenOutlined/>}>Show in Finder</Button>
                        </div>
                    </div>
                </div>
            </div>

            <div className="mt-6">
                <Title level={4} className="mb-4">Other</Title>
                <div className="border-b pb-4">
                    <div className="flex justify-between items-center">
                        <div>
                            <Text strong>Spell Check</Text>
                            <div className="text-sm text-gray-500">Enable spell check for your threads.</div>
                        </div>
                        <Switch size="small" defaultChecked/>
                    </div>
                </div>
                <div className="border-b pb-4 mt-4">
                    <div className="flex justify-between items-center">
                        <div>
                            <Text strong>Reset To Factory Settings</Text>
                            <div className="text-sm text-gray-500">Restore application to its initial state, erasing all
                                models and chat history. This action is irreversible and recommended only if the
                                application is corrupted.
                            </div>
                        </div>
                        <Button type="primary" danger>Reset</Button>
                    </div>
                </div>
            </div>

            <div className="mt-6">
                <Title level={4} className="mb-4">Resources</Title>
                <div className="border-b pb-4">
                    <div className="flex justify-between items-center">
                        <div>
                            <Text strong>Documentation</Text>
                            <div className="text-sm text-gray-500">Learn how to use Jan and explore its features.</div>
                        </div>
                        <Button type="link" className="text-pink-500">View Docs</Button>
                    </div>
                </div>
                <div className="border-b pb-4 mt-4">
                    <div className="flex justify-between items-center">
                        <div>
                            <Text strong>Release Notes</Text>
                            <div className="text-sm text-gray-500">See what's new in the latest version of Jan.</div>
                        </div>
                        <Button type="link" className="text-pink-500">View Releases</Button>
                    </div>
                </div>
            </div>

            <div className="mt-6">
                <Title level={4} className="mb-4">Community</Title>
                <div className="border-b pb-4">
                    <div className="flex justify-between items-center">
                        <div>
                            <Text strong>GitHub</Text>
                            <div className="text-sm text-gray-500">Contribute to Jan's development.</div>
                        </div>
                        <Button type="text" className="text-gray-500">
                            <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
                                <path
                                    d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.013 8.013 0 0016 8c0-4.42-3.58-8-8-8z"/>
                            </svg>
                        </Button>
                    </div>
                </div>
                <div className="border-b pb-4 mt-4">
                    <div className="flex justify-between items-center">
                        <div>
                            <Text strong>Discord</Text>
                            <div className="text-sm text-gray-500">Join our community for support and discussions.</div>
                        </div>
                        <Button type="text" className="text-gray-500">
                            <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
                                <path
                                    d="M13.545 2.907a13.227 13.227 0 0 0-3.257-1.011.05.05 0 0 0-.052.025c-.141.25-.297.577-.406.833a12.19 12.19 0 0 0-3.658 0 8.258 8.258 0 0 0-.412-.833.051.051 0 0 0-.052-.025c-1.125.194-2.22.534-3.257 1.011a.041.041 0 0 0-.021.018C.356 6.024-.213 9.047.066 12.032c.001.014.01.028.021.037a13.276 13.276 0 0 0 3.995 2.02.05.05 0 0 0 .056-.019c.308-.42.582-.863.818-1.329a.05.05 0 0 0-.01-.059.051.051 0 0 0-.018-.011 8.875 8.875 0 0 1-1.248-.595.05.05 0 0 1-.02-.066.051.051 0 0 1 .015-.019c.084-.063.168-.129.248-.195a.05.05 0 0 1 .051-.007c2.619 1.196 5.454 1.196 8.041 0a.052.052 0 0 1 .053.007c.08.066.164.132.248.195a.051.051 0 0 1-.004.085 8.254 8.254 0 0 1-1.249.594.05.05 0 0 0-.03.03.052.052 0 0 0 .003.041c.24.465.515.909.817 1.329a.05.05 0 0 0 .056.019 13.235 13.235 0 0 0 4.001-2.02.049.049 0 0 0 .021-.037c.334-3.451-.559-6.449-2.366-9.106a.034.034 0 0 0-.02-.019z"/>
                            </svg>
                        </Button>
                    </div>
                </div>
            </div>

            <div className="mt-6">
                <Title level={4} className="mb-4">Support</Title>
                <div className="border-b pb-4">
                    <div className="flex justify-between items-center">
                        <div>
                            <Text strong>Report an Issue</Text>
                            <div className="text-sm text-gray-500">Found a bug? Help us out by filing an issue on
                                GitHub.
                            </div>
                        </div>
                        <Button type="link" className="text-pink-500">Report Issue</Button>
                    </div>
                </div>
            </div>
        </div>
    )
}