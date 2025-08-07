import { useEffect } from 'react'
import { BrowserRouter as Router, Route, Routes } from 'react-router-dom'
import { AuthGuard } from './components2/Auth'
import { ThemeProvider } from './components2/Providers/ThemeProvider'
import { AppLayout } from './components2/Layout/AppLayout'
import { App as AntdApp } from 'antd'
import { useTranslation } from 'react-i18next'
import { useUserAppearanceLanguage } from './store'
import { ProjectsPage } from './components2/Pages/Projects/ProjectsPage'
import { ProjectDetailsPage } from './components2/Pages/Projects/ProjectDetailsPage'
import './i18n'
import '@ant-design/v5-patch-for-react-19'
import './store/startup'

// Simple placeholder component for testing
function HomePage() {
  return (
    <div className="p-6">
      <h1 className="text-2xl font-bold mb-4">Welcome to Ziee</h1>
      <p className="text-gray-600">
        New macOS-style interface is working! This is a placeholder homepage.
      </p>
    </div>
  )
}

function App2() {
  const { i18n } = useTranslation()
  const language = useUserAppearanceLanguage()

  // Update language when settings change
  useEffect(() => {
    if (i18n.language !== language) {
      i18n.changeLanguage(language)
    }
  }, [language, i18n.language])

  return (
    <ThemeProvider>
      <AuthGuard>
        <Router>
          <AntdApp>
            <AppLayout>
              <Routes>
                <Route path="/" element={<HomePage />} />
                <Route path="/projects" element={<ProjectsPage />} />
                <Route
                  path="/projects/:projectId"
                  element={<ProjectDetailsPage />}
                />
              </Routes>
            </AppLayout>
          </AntdApp>
        </Router>
      </AuthGuard>
    </ThemeProvider>
  )
}

export default App2
