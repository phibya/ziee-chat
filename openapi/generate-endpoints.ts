import { readFileSync, writeFileSync } from 'fs'
import { dirname, resolve } from 'path'
import { fileURLToPath } from 'url'

const scriptDir = dirname(fileURLToPath(import.meta.url))

const target = '../src/types/api-generated.ts'
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
}

function isSchemaReference(schema: any): schema is SchemaReference {
  return schema && typeof schema === 'object' && '$ref' in schema
}

function extractSchemaName(ref: string): string {
  return ref.replace('#/components/schemas/', '')
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
        responses[operationId] = generateResponseType(operation)
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

  // Add query parameters
  if (operation.parameters) {
    for (const param of operation.parameters) {
      if (param.in === 'query') {
        const isOptional = !param.required
        const paramType = getTypeFromSchema(param.schema)
        paramTypes.push(`${param.name}${isOptional ? '?' : ''}: ${paramType}`)
      }
    }
  }

  // Add request body type
  if (operation.requestBody) {
    const content = operation.requestBody.content['application/json']
    if (content && isSchemaReference(content.schema)) {
      return extractSchemaName(content.schema.$ref)
    } else if (content) {
      return 'any' // Generic fallback for complex inline schemas
    }
  }

  // Return parameter object type or void
  if (paramTypes.length === 0) {
    return 'void'
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

function generateResponseType(operation: PathOperation): string {
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
      return 'void'
    }
    return 'any'
  }

  // If there's no content, return void
  if (!successResponse.content) {
    return 'void'
  }

  // Look for application/json content
  const jsonContent = successResponse.content['application/json']
  if (!jsonContent || !jsonContent.schema) {
    return 'any'
  }

  // Extract schema reference or type
  if (isSchemaReference(jsonContent.schema)) {
    return extractSchemaName(jsonContent.schema.$ref)
  } else {
    return getTypeFromSchema(jsonContent.schema)
  }
}

function getTypeFromSchema(schema: any): string {
  // Handle boolean literal values (like profile: true in User schema)
  if (typeof schema === 'boolean') {
    return 'any' // or could be the literal boolean value
  }

  if (isSchemaReference(schema)) {
    return extractSchemaName(schema.$ref)
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
    // Handle union types like ["string", "null"]
    const types = schema.type.map((t: string) => {
      switch (t) {
        case 'string':
          return 'string'
        case 'integer':
        case 'number':
          return 'number'
        case 'boolean':
          return 'boolean'
        case 'null':
          return 'null'
        default:
          return 'any'
      }
    })
    return types.join(' | ')
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
      const isOptional = !schema.required?.includes(propName)
      const optionalMarker = isOptional ? '?' : ''
      const propType = getTypeFromSchema(propSchema)
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
