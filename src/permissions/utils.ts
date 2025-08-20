import React from 'react'
import { Permission, User, UserGroup } from '../types'
import { Stores } from '../store'

export const hasPermission = (
  permissions: Permission[],
  user?: User,
): boolean => {
  // testing
  if (permissions.includes(Permission.ChatSearch)) return false

  if (!user) {
    user = Stores.Auth.__state.user || undefined
  }
  if (!user?.groups) {
    return false
  }

  if (permissions.length === 0) {
    return true
  }

  const permissionsSet = new Set<Permission>(permissions)
  const permissionCategoriesSet = new Set<string>(
    permissions.map(permission => permission.replace(/::[^:]*$/, '')),
  )

  const groupPermissionsSet = user.groups.reduce(
    (acc: Set<Permission>, group: UserGroup) => {
      if (!group.is_active || !group.permissions) {
        return acc
      }

      return new Set([...acc, ...group.permissions]) as Set<Permission>
    },
    new Set<Permission>(),
  )

  if (groupPermissionsSet.has(Permission.All)) return true

  for (const groupPermission of groupPermissionsSet) {
    const isWildcard = groupPermission.endsWith('::*')
    if (isWildcard) {
      const category = groupPermission.replace(/::[^:]*$/, '')
      if (!permissionCategoriesSet.has(category)) {
        return false
      }
    }
    if (!permissionsSet.has(groupPermission)) {
      return false
    }
  }

  return true
}

/**
 * Properly disable React children by cloning with appropriate disabled props
 */
export const disableChildren = (children: React.ReactNode): React.ReactNode => {
  return React.Children.map(children, child => {
    if (!React.isValidElement(child)) {
      return child
    }

    // Handle different component types that can be disabled
    const childType = child.type
    const childProps = child.props

    // For Ant Design components and native HTML elements
    const disabledProps: Record<string, any> = {
      disabled: true,
    }

    // Handle specific component types
    if (typeof childType === 'string') {
      // Native HTML elements
      switch (childType) {
        case 'button':
        case 'input':
        case 'textarea':
        case 'select':
          disabledProps.disabled = true
          disabledProps.style = {
            ...childProps.style,
            pointerEvents: 'none',
            cursor: 'not-allowed',
          }
          break
        case 'a':
          disabledProps.onClick = (e: React.MouseEvent) => e.preventDefault()
          disabledProps.style = {
            ...childProps.style,
            pointerEvents: 'none',
            cursor: 'not-allowed',
          }
          break
        case 'div':
        case 'span':
          disabledProps.style = {
            ...childProps.style,
            pointerEvents: 'none',
            cursor: 'not-allowed',
          }
          break
        default:
          disabledProps.style = {
            ...childProps.style,
            pointerEvents: 'none',
            cursor: 'not-allowed',
          }
      }
    } else if (
      typeof childType === 'function' ||
      typeof childType === 'object'
    ) {
      // React components (including Ant Design components)
      disabledProps.disabled = true

      // Handle onClick and other event handlers
      if (childProps.onClick) {
        disabledProps.onClick = (e: React.MouseEvent) => {
          e.preventDefault()
          e.stopPropagation()
        }
      }

      // Add visual disabled styling
      disabledProps.style = {
        ...childProps.style,
        pointerEvents: 'none',
        cursor: 'not-allowed',
      }
    }

    // Clone the element with disabled props, and recursively disable children
    return React.cloneElement(
      child,
      disabledProps,
      child.props.children
        ? disableChildren(child.props.children)
        : child.props.children,
    )
  })
}

/**
 * Higher-Order Component for permission-based rendering
 * Works as both HOC function and can be used for component wrapping
 *
 * Usage:
 * ```tsx
 * // Method 1: Direct HOC usage
 * const ProtectedComponent = withPermission([Permission.HubAccess])(HubPage)
 *
 * // Method 2: Export wrapped component
 * export const HubPage = withPermission([Permission.HubAccess])(() => {
 *   return <div>Hub Content</div>
 * })
 *
 * // Method 3: With fallback
 * const ProtectedComponent = withPermission(
 *   [Permission.UsersRead],
 *   () => <div>Access Denied</div>
 * )(UserComponent)
 *
 * // Method 4: Disable instead of hide
 * const DisabledComponent = withPermission(
 *   [Permission.UsersEdit],
 *   null,
 *   true // disableInsteadOfHide
 * )(EditButton)
 * ```
 */
export const withPermission = (
  permissions: Permission[],
  fallback?: React.ComponentType<any> | React.ReactElement | null,
  disableInsteadOfHide?: boolean,
) => {
  return <T extends React.ComponentType<any>>(
    Component: T,
  ): React.ComponentType<React.ComponentProps<T>> => {
    const WrappedComponent = React.forwardRef<any, React.ComponentProps<T>>(
      (props, ref) => {
        if (hasPermission(permissions)) {
          return React.createElement(Component, { ...props, ref })
        }

        // If we should disable instead of hide
        if (disableInsteadOfHide) {
          const componentElement = React.createElement(Component, {
            ...props,
            ref,
          })
          return React.createElement(
            React.Fragment,
            {},
            disableChildren(componentElement),
          )
        }

        // Render fallback component or null
        if (fallback) {
          if (React.isValidElement(fallback)) {
            return fallback
          } else {
            return React.createElement(
              fallback as React.ComponentType<any>,
              props,
            )
          }
        }

        return null
      },
    )

    // Set display name for debugging
    WrappedComponent.displayName = `withPermission(${Component.displayName || Component.name})`

    return WrappedComponent
  }
}

/**
 * React hook for permission checking with disable support
 *
 * Usage:
 * ```tsx
 * const MyComponent = () => {
 *   const canEdit = usePermission([Permission.UsersEdit])
 *
 *   return (
 *     <Button disabled={!canEdit}>
 *       Edit User
 *     </Button>
 *   )
 * }
 * ```
 */
export const usePermission = (permissions: Permission[]): boolean => {
  return hasPermission(permissions)
}
