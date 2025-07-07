import {Button, Switch, Typography} from 'antd'
import {CopyOutlined} from '@ant-design/icons'

const {Title, Text} = Typography

export function AppearanceSettings() {
    return (
        <div className="space-y-6">
            <div className="mb-6">
                <Title level={3} className="mb-0">Appearance</Title>
            </div>

            <div className="border-b pb-4">
                <div className="flex justify-between items-center">
                    <div>
                        <Text strong>Theme</Text>
                        <div className="text-sm text-gray-500">Match the OS theme.</div>
                    </div>
                    <Button type="default" className="bg-gray-100">System</Button>
                </div>
            </div>

            <div className="border-b pb-4">
                <div className="flex justify-between items-center">
                    <div>
                        <Text strong>Font Size</Text>
                        <div className="text-sm text-gray-500">Adjust the app's font size.</div>
                    </div>
                    <Button type="default" className="bg-gray-100">Medium</Button>
                </div>
            </div>

            <div className="border-b pb-4">
                <div className="flex justify-between items-center">
                    <div>
                        <Text strong>Window Background</Text>
                        <div className="text-sm text-gray-500">Set the app window's background color.</div>
                    </div>
                    <div className="flex space-x-2">
                        <div className="w-6 h-6 rounded-full bg-red-500 cursor-pointer border-2 border-gray-300"></div>
                        <div className="w-6 h-6 rounded-full bg-blue-500 cursor-pointer border-2 border-gray-300"></div>
                        <div
                            className="w-6 h-6 rounded-full bg-purple-500 cursor-pointer border-2 border-gray-300"></div>
                        <div
                            className="w-6 h-6 rounded-full bg-yellow-500 cursor-pointer border-2 border-gray-300"></div>
                        <div
                            className="w-6 h-6 rounded-full bg-green-500 cursor-pointer border-2 border-gray-300"></div>
                        <div
                            className="w-6 h-6 rounded-full bg-orange-500 cursor-pointer border-2 border-gray-300"></div>
                        <div
                            className="w-6 h-6 rounded-sm bg-gray-200 cursor-pointer border-2 border-gray-300 flex items-center justify-center">
                            <span className="text-xs">✎</span>
                        </div>
                    </div>
                </div>
            </div>

            <div className="border-b pb-4">
                <div className="flex justify-between items-center">
                    <div>
                        <Text strong>App Main View</Text>
                        <div className="text-sm text-gray-500">Set the main content area's background color.</div>
                    </div>
                    <div className="flex space-x-2">
                        <div className="w-6 h-6 rounded-full bg-white cursor-pointer border-2 border-gray-300"></div>
                        <div className="w-6 h-6 rounded-full bg-gray-800 cursor-pointer border-2 border-gray-300"></div>
                        <div
                            className="w-6 h-6 rounded-sm bg-gray-200 cursor-pointer border-2 border-gray-300 flex items-center justify-center">
                            <span className="text-xs">✎</span>
                        </div>
                    </div>
                </div>
            </div>

            <div className="border-b pb-4">
                <div className="flex justify-between items-center">
                    <div>
                        <Text strong>Primary</Text>
                        <div className="text-sm text-gray-500">Set the primary color for UI components.</div>
                    </div>
                    <div className="flex space-x-2">
                        <div
                            className="w-6 h-6 rounded-full bg-orange-500 cursor-pointer border-2 border-gray-300"></div>
                        <div className="w-6 h-6 rounded-full bg-blue-500 cursor-pointer border-2 border-gray-300"></div>
                        <div
                            className="w-6 h-6 rounded-full bg-green-500 cursor-pointer border-2 border-gray-300"></div>
                        <div
                            className="w-6 h-6 rounded-full bg-purple-500 cursor-pointer border-2 border-gray-300"></div>
                        <div
                            className="w-6 h-6 rounded-sm bg-gray-200 cursor-pointer border-2 border-gray-300 flex items-center justify-center">
                            <span className="text-xs">✎</span>
                        </div>
                    </div>
                </div>
            </div>

            <div className="border-b pb-4">
                <div className="flex justify-between items-center">
                    <div>
                        <Text strong>Accent</Text>
                        <div className="text-sm text-gray-500">Set the accent color for UI highlights.</div>
                    </div>
                    <div className="flex space-x-2">
                        <div className="w-6 h-6 rounded-full bg-blue-500 cursor-pointer border-2 border-gray-300"></div>
                        <div className="w-6 h-6 rounded-full bg-red-500 cursor-pointer border-2 border-gray-300"></div>
                        <div
                            className="w-6 h-6 rounded-full bg-green-500 cursor-pointer border-2 border-gray-300"></div>
                        <div
                            className="w-6 h-6 rounded-sm bg-gray-200 cursor-pointer border-2 border-gray-300 flex items-center justify-center">
                            <span className="text-xs">✎</span>
                        </div>
                    </div>
                </div>
            </div>

            <div className="border-b pb-4">
                <div className="flex justify-between items-center">
                    <div>
                        <Text strong>Destructive</Text>
                        <div className="text-sm text-gray-500">Set the color for destructive actions.</div>
                    </div>
                    <div className="flex space-x-2">
                        <div className="w-6 h-6 rounded-full bg-red-500 cursor-pointer border-2 border-gray-300"></div>
                        <div className="w-6 h-6 rounded-full bg-red-600 cursor-pointer border-2 border-gray-300"></div>
                        <div
                            className="w-6 h-6 rounded-full bg-purple-500 cursor-pointer border-2 border-gray-300"></div>
                        <div className="w-6 h-6 rounded-full bg-pink-500 cursor-pointer border-2 border-gray-300"></div>
                        <div
                            className="w-6 h-6 rounded-sm bg-gray-200 cursor-pointer border-2 border-gray-300 flex items-center justify-center">
                            <span className="text-xs">✎</span>
                        </div>
                    </div>
                </div>
            </div>

            <div className="border-b pb-4">
                <div className="flex justify-between items-center">
                    <div>
                        <Text strong>Reset to Default</Text>
                        <div className="text-sm text-gray-500">Reset all appearance settings to default.</div>
                    </div>
                    <Button type="primary" danger>Reset</Button>
                </div>
            </div>

            <div className="mt-6">
                <div className="border-b pb-4">
                    <div>
                        <Text strong>Chat Width</Text>
                        <div className="text-sm text-gray-500 mb-4">Customize the width of the chat view.</div>
                    </div>
                    <div className="grid grid-cols-2 gap-4">
                        <div className="border-2 border-pink-500 rounded-lg p-4 bg-gray-50">
                            <div className="text-sm font-medium mb-2">Compact Width</div>
                            <div className="bg-gray-200 rounded p-2 space-y-1">
                                <div className="h-2 bg-gray-300 rounded"></div>
                                <div className="h-2 bg-gray-300 rounded"></div>
                                <div className="h-2 bg-gray-300 rounded"></div>
                                <div className="h-2 bg-gray-300 rounded"></div>
                                <div className="h-2 bg-gray-300 rounded"></div>
                            </div>
                            <div className="mt-2 p-2 bg-gray-300 rounded text-xs text-center">Ask me anything...</div>
                        </div>
                        <div className="border-2 border-gray-300 rounded-lg p-4 bg-gray-50">
                            <div className="text-sm font-medium mb-2">Full Width</div>
                            <div className="bg-gray-200 rounded p-2 space-y-1">
                                <div className="h-2 bg-gray-300 rounded"></div>
                                <div className="h-2 bg-gray-300 rounded"></div>
                                <div className="h-2 bg-gray-300 rounded"></div>
                                <div className="h-2 bg-gray-300 rounded"></div>
                                <div className="h-2 bg-gray-300 rounded"></div>
                            </div>
                            <div className="mt-2 p-2 bg-gray-300 rounded text-xs text-center">Ask me anything...</div>
                        </div>
                    </div>
                </div>
            </div>

            <div className="mt-6">
                <div className="border-b pb-4">
                    <div className="flex justify-between items-center mb-4">
                        <div>
                            <Text strong>Code Block</Text>
                            <div className="text-sm text-gray-500">Choose a syntax highlighting style.</div>
                        </div>
                        <Button type="default" className="bg-gray-100">VSCode Dark+</Button>
                    </div>
                    <div className="bg-gray-900 rounded-lg p-4">
                        <div className="flex justify-between items-center mb-2">
                            <span className="text-gray-400 text-sm">Preview</span>
                            <div className="flex items-center space-x-2">
                                <span className="text-gray-400 text-sm">Typescript</span>
                                <Button size="small" type="text" className="text-gray-400" icon={<CopyOutlined/>}>
                                    Copy
                                </Button>
                            </div>
                        </div>
                        <div className="text-sm font-mono">
                            <div className="text-gray-500">1 <span
                                className="text-gray-400">// Example code for preview</span></div>
                            <div className="text-gray-500">2 <span className="text-blue-400">function</span>
                                <span
                                    className="text-yellow-400">greeting</span>(<span
                                    className="text-orange-400">name</span>: <span
                                    className="text-blue-300">string</span></div>
                            <div className="text-gray-500">3 <span className="text-blue-400">return</span> <span
                                className="text-green-400">`Hello, ${"{name}"}!`
                            </span>
                            </div>
                            <div className="text-gray-500">4</div>
                            <div className="text-gray-500">5</div>
                            <div className="text-gray-500">6 <span className="text-gray-400">// Call the function</span>
                            </div>
                            <div className="text-gray-500">7 <span className="text-blue-400">const</span> <span
                                className="text-white">message</span> = <span
                                className="text-yellow-400">greeting</span>(<span
                                className="text-green-400">'Jan'</span>);
                            </div>
                            <div className="text-gray-500">8 <span className="text-blue-400">console</span>.<span
                                className="text-yellow-400">log</span>(<span
                                className="text-white">message</span>); <span
                                className="text-gray-400">// Outputs: Hello, Jan!</span></div>
                        </div>
                    </div>
                </div>
            </div>

            <div className="mt-6">
                <div className="border-b pb-4">
                    <div className="flex justify-between items-center">
                        <div>
                            <Text strong>Show Line Numbers</Text>
                            <div className="text-sm text-gray-500">Display line numbers in code blocks.</div>
                        </div>
                        <Switch size="small" defaultChecked/>
                    </div>
                </div>
            </div>

            <div className="mt-6">
                <div className="border-b pb-4">
                    <div className="flex justify-between items-center">
                        <div>
                            <Text strong>Reset Code Block Style</Text>
                            <div className="text-sm text-gray-500">Reset code block style to default.</div>
                        </div>
                        <Button type="primary" danger>Reset</Button>
                    </div>
                </div>
            </div>
        </div>
    )
}