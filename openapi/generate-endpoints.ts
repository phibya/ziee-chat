import { readFileSync, writeFileSync } from 'fs'
import { dirname, resolve } from 'path'
import { fileURLToPath } from 'url'

const scriptDir = dirname(fileURLToPath(import.meta.url))

const target = '../src/types/api.ts'
const openapiJson = './openapi.json'

const targetPath = resolve(scriptDir, target)
const openapiJsonPath = resolve(scriptDir, openapiJson)

interface OpenApiSpec {
  openapi: string
  info: {
    title: string
    version: string
  }
  paths: Record<string, Record<string, PathOperation>>
  components: {
    schemas: Record<string, SchemaDefinition>
  }
}

interface PathOperation {
  tags?: string[]
  description?: string
  operationId: string
  parameters?: Parameter[]
  requestBody?: RequestBody
  responses?: Record<string, ResponseDefinition>
}

interface Parameter {
  in: 'query' | 'path' | 'header'
  name: string
  schema: SchemaReference | SchemaType
  required?: boolean
  style?: string
}

interface RequestBody {
  content: Record<string, { schema: SchemaReference }>
  required?: boolean
}

interface ResponseDefinition {
  description?: string
  content?: Record<string, { schema: SchemaReference | SchemaType }>
}

interface SchemaReference {
  $ref: string
}

interface SchemaType {
  type: string | string[]
  format?: string
  properties?: Record<string, SchemaDefinition>
  items?: SchemaDefinition
  minimum?: number
}

interface SchemaDefinition extends SchemaType {
  $ref?: string
  required?: string[]
  anyOf?: (SchemaReference | SchemaType)[]
}

function isSchemaReference(schema: any): schema is SchemaReference {
  return schema && typeof schema === 'object' && '$ref' in schema
}

function extractSchemaName(ref: string): string {
  const schemaName = ref.replace('#/components/schemas/', '')
  
  // Special cases: convert to primitive types
  if (schemaName === 'AnyType') {
    return 'any'
  }
  if (schemaName === 'BlobType') {
    return 'Blob'
  }
  
  return schemaName
}

function generateEndpoints(): void {
  try {
    // Read and parse the OpenAPI JSON
    const openapiContent = readFileSync(openapiJsonPath, 'utf8')
    const spec: OpenApiSpec = JSON.parse(openapiContent)

    // Extract all endpoints
    const endpoints: Record<string, string> = {}
    const parameters: Record<string, string> = {}
    const responses: Record<string, string> = {}

    // Process each path
    for (const [path, methods] of Object.entries(spec.paths)) {
      for (const [method, operation] of Object.entries(methods)) {
        const operationId = operation.operationId
        if (!operationId) continue

        // Generate endpoint mapping
        const httpMethod = method.toUpperCase()
        const apiPath = path.replace(/{([^}]+)}/g, '{$1}') // Keep parameter format
        endpoints[operationId] = `${httpMethod} ${apiPath}`

        // Generate parameter types
        parameters[operationId] = generateParameterType(operation, path)

        // Generate response types
        responses[operationId] = generateResponseType(operation, httpMethod)
      }
    }

    // Generate the TypeScript file content
    const content = generateTypeScriptContent(
      endpoints,
      parameters,
      responses,
      spec.components?.schemas || {},
    )

    // Write the generated file
    writeFileSync(targetPath, content, 'utf8')
    console.log(`✅ Generated API endpoints to ${targetPath}`)
  } catch (error) {
    console.error('❌ Error generating endpoints:', error)
    // eslint-disable-next-line no-process-exit
    process.exit(1)
  }
}

function detectQuerySchemaType(queryParams: Parameter[]): string | null {
  if (!queryParams.length) return null
  
  const paramNames = queryParams.map(p => p.name).sort()
  
  // Detect standard pagination pattern
  if (paramNames.length === 2 && 
      paramNames.includes('page') && 
      paramNames.includes('per_page')) {
    return 'PaginationQuery'
  }
  
  // Detect conversation pagination pattern  
  if (paramNames.length === 3 && 
      paramNames.includes('page') && 
      paramNames.includes('per_page') && 
      paramNames.includes('project_id')) {
    return 'ConversationPaginationQuery'
  }
  
  // Detect download pagination pattern
  if (paramNames.length === 3 && 
      paramNames.includes('page') && 
      paramNames.includes('per_page') && 
      paramNames.includes('status')) {
    return 'DownloadPaginationQuery'
  }
  
  // Detect project list query pattern
  if (paramNames.length === 3 && 
      paramNames.includes('page') && 
      paramNames.includes('per_page') && 
      paramNames.includes('search')) {
    return 'ProjectListQuery'
  }
  
  // Note: FileListParams pattern is same as ProjectListQuery, 
  // will be handled by the previous case
  
  // Detect hub query params pattern
  if (paramNames.length === 1 && 
      paramNames.includes('lang')) {
    return 'HubQueryParams'
  }
  
  // Detect search query pattern
  if (paramNames.length === 4 && 
      paramNames.includes('q') &&
      paramNames.includes('page') && 
      paramNames.includes('per_page') && 
      paramNames.includes('project_id')) {
    return 'SearchQuery'
  }
  
  return null
}

function generateParameterType(operation: PathOperation, path: string): string {
  const paramTypes: string[] = []

  // Add path parameters
  const pathParams = path.match(/{([^}]+)}/g)
  if (pathParams) {
    for (const param of pathParams) {
      const paramName = param.slice(1, -1) // Remove { }
      paramTypes.push(`${paramName}: string`)
    }
  }

  // Collect query parameters and detect schema patterns
  const queryParams: string[] = []
  let querySchemaType: string | null = null
  
  if (operation.parameters) {
    for (const param of operation.parameters) {
      if (param.in === 'query') {
        const isOptional = !param.required
        const paramType = getTypeFromSchema(param.schema, isOptional)
        queryParams.push(`${param.name}${isOptional ? '?' : ''}: ${paramType}`)
      }
    }
    
    // Detect common query parameter patterns and map to schema types
    querySchemaType = detectQuerySchemaType(operation.parameters.filter(p => p.in === 'query'))
  }
  
  // Use schema type if detected, otherwise use individual parameters
  if (querySchemaType) {
    // Don't add individual query params, we'll use the schema type
  } else {
    paramTypes.push(...queryParams)
  }

  // Add request body type
  let requestBodyType: string | null = null
  if (operation.requestBody) {
    // Try application/json first
    let content = operation.requestBody.content['application/json']
    
    // If no application/json, try multipart/form-data
    if (!content) {
      content = operation.requestBody.content['multipart/form-data']
    }
    
    if (content && isSchemaReference(content.schema)) {
      requestBodyType = extractSchemaName(content.schema.$ref)
    } else if (content) {
      // For multipart/form-data or complex inline schemas, use FormData or any
      if (operation.requestBody.content['multipart/form-data']) {
        requestBodyType = 'FormData'
      } else {
        requestBodyType = 'any' // Generic fallback for complex inline schemas
      }
    }
  }

  // Return parameter object type or void
  if (paramTypes.length === 0 && !querySchemaType && !requestBodyType) {
    return 'void'
  } else if (paramTypes.length === 0 && !querySchemaType && requestBodyType) {
    // Only request body, no parameters
    return requestBodyType
  } else if (querySchemaType && !requestBodyType && paramTypes.length === 0) {
    // Only query schema, no path params or request body
    return querySchemaType
  } else if (querySchemaType && requestBodyType && paramTypes.length === 0) {
    // Query schema and request body, no path params
    return `${querySchemaType} & ${requestBodyType}`
  } else if (querySchemaType && paramTypes.length > 0 && !requestBodyType) {
    // Query schema and path params, no request body
    return `{ ${paramTypes.join('; ')} } & ${querySchemaType}`
  } else if (querySchemaType && paramTypes.length > 0 && requestBodyType) {
    // Query schema, path params, and request body
    return `{ ${paramTypes.join('; ')} } & ${querySchemaType} & ${requestBodyType}`
  } else if (paramTypes.length > 0 && requestBodyType) {
    // Both parameters and request body - combine them (fallback to old logic)
    return `{ ${paramTypes.join('; ')} } & ${requestBodyType}`
  } else if (
    paramTypes.length === 1 &&
    !operation.parameters?.some(p => p.in === 'query')
  ) {
    // Single path parameter, return as object
    return `{ ${paramTypes[0]} }`
  } else {
    return `{ ${paramTypes.join('; ')} }`
  }
}

function generateResponseType(operation: PathOperation, httpMethod?: string): string {
  if (!operation.responses) {
    return 'void'
  }

  // Look for successful responses (200, 201, 204, etc.)
  const successResponse =
    operation.responses['200'] ||
    operation.responses['201'] ||
    operation.responses['202']

  if (!successResponse) {
    // Check for 204 No Content
    if (operation.responses['204']) {
      // For POST requests, even 204 should return 'any' instead of 'void'
      return httpMethod === 'POST' ? 'any' : 'void'
    }
    return 'any'
  }

  // If there's no content, return void for non-POST or any for POST
  if (!successResponse.content) {
    return httpMethod === 'POST' ? 'any' : 'void'
  }

  // Look for application/json content
  const jsonContent = successResponse.content['application/json']
  if (!jsonContent || !jsonContent.schema) {
    return httpMethod === 'POST' ? 'any' : 'any'
  }

  // Extract schema reference or type
  if (isSchemaReference(jsonContent.schema)) {
    return extractSchemaName(jsonContent.schema.$ref)
  } else {
    return getTypeFromSchema(jsonContent.schema)
  }
}

function getTypeFromSchema(schema: any, isOptionalParamOrNullable = false): string {
  // Handle boolean literal values (like profile: true in User schema)
  if (typeof schema === 'boolean') {
    return 'any' // or could be the literal boolean value
  }

  if (isSchemaReference(schema)) {
    return extractSchemaName(schema.$ref)
  }

  // Handle anyOf patterns (union types with schema references)
  if (schema.anyOf && Array.isArray(schema.anyOf)) {
    const types = schema.anyOf
      .map((subSchema: any) => {
        if (isSchemaReference(subSchema)) {
          return extractSchemaName(subSchema.$ref)
        } else if (subSchema.type === 'null') {
          // If there's a null in anyOf and we're dealing with optional/nullable, skip it
          return isOptionalParamOrNullable ? null : 'null'
        } else {
          return getTypeFromSchema(subSchema, isOptionalParamOrNullable)
        }
      })
      .filter((type: string | null) => type !== null) // Remove null entries when filtered out
    
    return types.length === 1 ? types[0] : types.join(' | ')
  }

  // Handle enum types
  if (schema.enum && Array.isArray(schema.enum)) {
    // Convert enum values to union type with string literals
    return schema.enum.map((value: any) => `'${value}'`).join(' | ')
  }

  if (typeof schema.type === 'string') {
    switch (schema.type) {
      case 'string':
        if (schema.format === 'date-time') {
          return 'string' // Could be Date if preferred
        }
        return 'string'
      case 'integer':
      case 'number':
        return 'number'
      case 'boolean':
        return 'boolean'
      case 'array':
        if (schema.items) {
          const itemType = getTypeFromSchema(schema.items)
          return `${itemType}[]`
        }
        return 'any[]'
      case 'object':
        if (schema.properties) {
          // Generate inline object type
          const props: string[] = []
          for (const [propName, propSchema] of Object.entries(
            schema.properties,
          )) {
            const propType = getTypeFromSchema(propSchema)
            props.push(`${propName}: ${propType}`)
          }
          return `{ ${props.join('; ')} }`
        }
        return 'any'
      default:
        return 'any'
    }
  } else if (Array.isArray(schema.type)) {
    // Handle union types like ["string", "null"] or ["array", "null"]
    const types = schema.type
      .filter((t: string) => {
        // If this is an optional parameter or nullable property, exclude null from the union
        // since optional parameters/properties don't need explicit null
        if (isOptionalParamOrNullable && t === 'null') {
          return false
        }
        return true
      })
      .map((t: string) => {
        switch (t) {
          case 'string':
            return 'string'
          case 'integer':
          case 'number':
            return 'number'
          case 'boolean':
            return 'boolean'
          case 'array':
            // Handle array type in union - use the items schema
            if (schema.items) {
              const itemType = getTypeFromSchema(schema.items)
              return `${itemType}[]`
            }
            return 'any[]'
          case 'null':
            return 'null'
          default:
            return 'any'
        }
      })
    return types.length === 1 ? types[0] : types.join(' | ')
  }

  return 'any'
}

function generateSchemaInterface(
  name: string,
  schema: SchemaDefinition,
): string {
  if (schema.$ref) {
    // This shouldn't happen for top-level schemas, but handle it just in case
    return `export type ${name} = ${extractSchemaName(schema.$ref)}`
  }

  if (schema.type === 'object' && schema.properties) {
    const properties: string[] = []

    for (const [propName, propSchema] of Object.entries(schema.properties)) {
      let isOptional = !schema.required?.includes(propName)
      
      // Check if property is nullable (has null in union type or anyOf with null)
      const isNullableUnion = Array.isArray(propSchema.type) && propSchema.type.includes('null')
      const isNullableAnyOf = propSchema.anyOf && 
        Array.isArray(propSchema.anyOf) && 
        propSchema.anyOf.some((subSchema: any) => subSchema.type === 'null')
      const isNullable = isNullableUnion || isNullableAnyOf
      
      // If property is nullable, make it optional and exclude null from type
      if (isNullable) {
        isOptional = true
      }
      
      const optionalMarker = isOptional ? '?' : ''
      const propType = getTypeFromSchema(propSchema, isNullable)
      properties.push(`  ${propName}${optionalMarker}: ${propType}`)
    }

    return `export interface ${name} {
${properties.join('\n')}
}`
  } else if (schema.type === 'array' && schema.items) {
    const itemType = getTypeFromSchema(schema.items)
    return `export type ${name} = ${itemType}[]`
  } else {
    // For primitive types or other cases
    const baseType = getTypeFromSchema(schema)
    return `export type ${name} = ${baseType}`
  }
}

function generateAllSchemas(schemas: Record<string, SchemaDefinition>): string {
  const interfaces: string[] = []

  // Sort schema names for consistent output
  const sortedNames = Object.keys(schemas).sort()

  for (const schemaName of sortedNames) {
    // Skip primitive type schemas since they should be treated as built-in types
    if (schemaName === 'AnyType' || schemaName === 'BlobType') {
      continue
    }
    
    const schema = schemas[schemaName]
    const interfaceDefinition = generateSchemaInterface(schemaName, schema)
    interfaces.push(interfaceDefinition)
  }

  return interfaces.join('\n\n')
}

function generateTypeScriptContent(
  endpoints: Record<string, string>,
  parameters: Record<string, string>,
  responses: Record<string, string>,
  schemas: Record<string, SchemaDefinition>,
): string {
  const sortedEndpoints = Object.keys(endpoints).sort()

  // Generate header and schema definitions
  const header = `/**
 * Generated API endpoint definitions
 * Auto-generated from OpenAPI specification
 * 
 * ⚠️  DO NOT EDIT THIS FILE MANUALLY ⚠️
 * This file is automatically generated from the OpenAPI specification generated from the server code.
 */

// =============================================================================
// TYPE DEFINITIONS
// =============================================================================

`

  // Generate all schema interfaces
  const schemaDefinitions = generateAllSchemas(schemas) + '\n\n'

  // Generate endpoints object
  const endpointsSection = `// =============================================================================
// API ENDPOINTS
// =============================================================================

// API endpoint definitions
export const ApiEndpoints = {
${sortedEndpoints.map(key => `  '${key}': '${endpoints[key]}'`).join(',\n')}
} as const

`

  // Generate parameter types
  const parametersSection = `// API endpoint parameters
export type ApiEndpointParameters = {
${sortedEndpoints.map(key => `  '${key}': ${parameters[key]}`).join('\n')}
}

`

  // Generate response types
  const responsesSection = `// API endpoint responses
export type ApiEndpointResponses = {
${sortedEndpoints.map(key => `  '${key}': ${responses[key]}`).join('\n')}
}

`

  // Generate helper types
  const helpersSection = `// Type helpers
export type ApiEndpoint = keyof typeof ApiEndpoints
export type ApiEndpointUrl = (typeof ApiEndpoints)[ApiEndpoint]

// Extract endpoint key from URL pattern
export function getEndpointKey(url: string): ApiEndpoint | undefined {
  const entries = Object.entries(ApiEndpoints) as [ApiEndpoint, string][]
  const found = entries.find(([_key, value]) => value === url)
  return found ? found[0] : undefined
}

// Get parameter type for endpoint
export type GetParameterType<K extends ApiEndpoint> = ApiEndpointParameters[K]

// Get response type for endpoint  
export type GetResponseType<K extends ApiEndpoint> = ApiEndpointResponses[K]

// Create reverse mapping from URL to endpoint key
export type UrlToEndpoint<U extends ApiEndpointUrl> = {
  [K in keyof typeof ApiEndpoints]: (typeof ApiEndpoints)[K] extends U
    ? K
    : never
}[keyof typeof ApiEndpoints]

// Helper types to get parameter and response types by URL
export type ParameterByUrl<U extends ApiEndpointUrl> =
  ApiEndpointParameters[UrlToEndpoint<U>]
export type ResponseByUrl<U extends ApiEndpointUrl> =
  ApiEndpointResponses[UrlToEndpoint<U>]

// Type-safe validation - this will cause a TypeScript error if any endpoint is missing
type ValidateParametersComplete = {
  [K in keyof typeof ApiEndpoints]: K extends keyof ApiEndpointParameters
    ? true
    : false
}

type ValidateResponsesComplete = {
  [K in keyof typeof ApiEndpoints]: K extends keyof ApiEndpointResponses
    ? true
    : false
}

// Type-safe validation - these will cause a TypeScript error if any endpoint is missing
// from Parameters or Responses. They are used for compile-time validation only.
export type { ValidateParametersComplete, ValidateResponsesComplete }
`

  return (
    header +
    schemaDefinitions +
    endpointsSection +
    parametersSection +
    responsesSection +
    helpersSection
  )
}

// Run the generator
generateEndpoints()
