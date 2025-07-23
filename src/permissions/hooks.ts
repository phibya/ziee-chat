/**
 * Permission-related React hooks
 */

import { useMemo } from "react";
import { useShallow } from "zustand/react/shallow";
import { useAuthStore } from "../store";
import type { PermissionKey } from "./constants";
import type { PermissionContext } from "./types";
import {
  getUserEffectivePermissions,
  hasAllPermissions,
  hasAnyPermission,
  hasPermission,
} from "./utils";

const useUserId = () => {
  return useAuthStore(useShallow((state) => state.user?.id));
};

/**
 * Main permission hook providing all permission-related functionality
 */
export function usePermissions(): PermissionContext {
  useUserId() // Ensure user ID is fetched to trigger reactivity
  const user = useAuthStore.getState().user

  const effectivePermissions = useMemo(
    () => getUserEffectivePermissions(user),
    [user],
  )

  return {
    user,
    effectivePermissions,
    hasPermission: (permission: PermissionKey) =>
      hasPermission(user, permission),
    hasAnyPermission: (permissions: PermissionKey[]) =>
      hasAnyPermission(user, permissions),
    hasAllPermissions: (permissions: PermissionKey[]) =>
      hasAllPermissions(user, permissions),
  }
}
