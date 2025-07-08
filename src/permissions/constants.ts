/**
 * AWS-style permission constants for fine-grained access control
 * This file contains all permission constants used throughout the application
 */

// The original permission description object
export const PermissionDescription = {
  '*': 'Grants all permissions in the system',
  // User management permissions
  'users::*': 'Grants all user management permissions',
  'users::read': 'Allows viewing user lists and user details',
  'users::edit':
    'Allows editing user information, resetting passwords, and managing user status',
  'users::delete': 'Allows deleting user accounts',
  'users::create': 'Allows creating new user accounts',
  // Group management permissions
  'groups::*': 'Grants all group management permissions',
  'groups::read': 'Allows viewing group lists and group details',
  'groups::edit':
    'Allows editing group information and managing group memberships',
  'groups::delete': 'Allows deleting user groups',
  'groups::create': 'Allows creating new user groups',
  // Configuration permissions
  'config::user-registration::*':
    'Grants all user registration configuration permissions',
  'config::user-registration::read':
    'Allows viewing user registration settings',
  'config::user-registration::edit':
    'Allows modifying user registration settings',
  // Advanced configuration permissions (admin-only)
  'config::updates::*': 'Grants all update configuration permissions',
  'config::updates::read': 'Allows viewing update settings',
  'config::updates::edit':
    'Allows checking for updates and configuring update settings',
  'config::experimental::*': 'Grants all experimental features permissions',
  'config::experimental::read': 'Allows viewing experimental features settings',
  'config::experimental::edit':
    'Allows enabling/disabling experimental features',
  'config::data-folder::*': 'Grants all data folder configuration permissions',
  'config::data-folder::read': 'Allows viewing data folder location',
  'config::data-folder::edit': 'Allows changing data folder location',
  'config::factory-reset::*': 'Grants all factory reset permissions',
  'config::factory-reset::read': 'Allows viewing factory reset options',
  'config::factory-reset::edit': 'Allows performing factory reset operations',
  // Chat permissions
  'chat::use': 'Allows using chat functionality',
  // Profile permissions
  'profile::edit': 'Allows editing own profile information',
} as const // 'as const' is important for TypeScript to infer literal types
export const PermissionKeys = Object.keys(
  PermissionDescription,
) as Array<PermissionKey>

// A union of all the keys from PermissionDescription
export type PermissionKey = keyof typeof PermissionDescription

// Utility type to convert kebab-case strings to camelCase
type KebabToCamel<S extends string> = S extends `${infer T}-${infer U}`
  ? `${T}${Capitalize<KebabToCamel<U>>}`
  : S

// Utility type to transform a single part of a path (e.g., 'user-registration' -> 'userRegistration', '*' -> 'all')
type TransformPathPart<T extends string> = T extends '*'
  ? 'all'
  : KebabToCamel<T>

// Recursively builds a nested object type from a permission string (e.g., 'a::b::c' -> { a: { b: { c: 'a::b::c' } } })
type BuildPath<
  T extends string,
  TOriginal extends string = T,
> = T extends `${infer THead}::${infer TTail}`
  ? { readonly [K in TransformPathPart<THead>]: BuildPath<TTail, TOriginal> }
  : { readonly [K in TransformPathPart<T>]: TOriginal }

// Utility type to merge a union of objects into a single intersection (e.g., {a:1}|{b:2} -> {a:1}&{b:2})
type UnionToIntersection<U> = (U extends any ? (k: U) => void : never) extends (
  k: infer I,
) => void
  ? I
  : never

// The final, deeply nested Permission type with full IntelliSense support.
// It maps over each permission key, builds its path, and then merges all paths together.
export type PermissionType = UnionToIntersection<
  { [K in PermissionKey]: BuildPath<K> }[PermissionKey]
>

/**
 * Creates a deeply nested, dot-accessible object from the PermissionDescription.
 * The resulting object is fully typed by `PermissionType` for IntelliSense.
 */
function createPermissionsObject(
  description: Record<string, string>,
): PermissionType {
  const permissions: any = {}

  const kebabToCamel = (s: string): string =>
    s.replace(/-./g, x => x[1].toUpperCase())

  for (const key in description) {
    let currentLevel = permissions
    const parts = key.split('::')

    parts.forEach((part, index) => {
      // Transform part name: '*' -> 'all', 'kebab-case' -> 'camelCase'
      const transformedPart = part === '*' ? 'all' : kebabToCamel(part)

      // If it's the last part, assign the original key as the value
      if (index === parts.length - 1) {
        currentLevel[transformedPart] = key
      } else {
        // Otherwise, traverse deeper, creating new objects if they don't exist
        if (!currentLevel[transformedPart]) {
          currentLevel[transformedPart] = {}
        }
        currentLevel = currentLevel[transformedPart]
      }
    })
  }
  return permissions as PermissionType
}

export const Permission = createPermissionsObject(PermissionDescription)
