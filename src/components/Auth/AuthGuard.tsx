import { Layout, Spin } from "antd";
import React, { useEffect } from "react";
import {
  checkApplicationInitializationStatus,
  fetchCurrentUserProfile,
  Stores,
} from "../../store";
import { initializeUserSettingsOnStartup } from "../../store/settings";
import { AuthPage } from "./AuthPage";

const { Content } = Layout;

interface AuthGuardProps {
  children: React.ReactNode;
}

export const AuthGuard: React.FC<AuthGuardProps> = ({ children }) => {
  const { isAuthenticated, isLoading, token } = Stores.Auth;

  useEffect(() => {
    // Check initialization status on mount
    checkApplicationInitializationStatus();
  }, []);

  useEffect(() => {
    // If we have a token, fetch the current user
    if (token) {
      fetchCurrentUserProfile();
    }
  }, [token]);

  useEffect(() => {
    // Initialize user settings after authentication
    if (isAuthenticated && !isLoading) {
      initializeUserSettingsOnStartup().catch((error: unknown) => {
        console.error("Failed to initialize user settings:", error);
      });
    }
  }, [isAuthenticated, isLoading]);

  // Show loading spinner while checking auth status
  if (isLoading) {
    return (
      <Layout className="min-h-screen">
        <Content className="flex items-center justify-center">
          <Spin size="large" />
        </Content>
      </Layout>
    );
  }

  // Show authentication page if not authenticated
  if (!isAuthenticated) {
    return <AuthPage />;
  }

  // Show the protected content
  return <>{children}</>;
};
