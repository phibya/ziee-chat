"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
var fs_1 = require("fs");
var path_1 = require("path");
var url_1 = require("url");
var scriptDir = (0, path_1.dirname)((0, url_1.fileURLToPath)(import.meta.url));
var target = '../src/types/api.ts';
var openapiJson = './openapi.json';
var targetPath = (0, path_1.resolve)(scriptDir, target);
var openapiJsonPath = (0, path_1.resolve)(scriptDir, openapiJson);
function isSchemaReference(schema) {
    return schema && typeof schema === 'object' && '$ref' in schema;
}
function extractSchemaName(ref) {
    var schemaName = ref.replace('#/components/schemas/', '');
    // Special cases: convert to primitive types
    if (schemaName === 'AnyType') {
        return 'any';
    }
    if (schemaName === 'BlobType') {
        return 'Blob';
    }
    return schemaName;
}
function generateEndpoints() {
    var _a;
    try {
        // Read and parse the OpenAPI JSON
        var openapiContent = (0, fs_1.readFileSync)(openapiJsonPath, 'utf8');
        var spec = JSON.parse(openapiContent);
        // Extract all endpoints
        var endpoints = {};
        var parameters = {};
        var responses = {};
        // Process each path
        for (var _i = 0, _b = Object.entries(spec.paths); _i < _b.length; _i++) {
            var _c = _b[_i], path = _c[0], methods = _c[1];
            for (var _d = 0, _e = Object.entries(methods); _d < _e.length; _d++) {
                var _f = _e[_d], method = _f[0], operation = _f[1];
                var operationId = operation.operationId;
                if (!operationId)
                    continue;
                // Generate endpoint mapping
                var httpMethod = method.toUpperCase();
                var apiPath = path.replace(/{([^}]+)}/g, '{$1}'); // Keep parameter format
                endpoints[operationId] = "".concat(httpMethod, " ").concat(apiPath);
                // Generate parameter types
                parameters[operationId] = generateParameterType(operation, path);
                // Generate response types
                responses[operationId] = generateResponseType(operation, httpMethod);
            }
        }
        // Generate the TypeScript file content
        var content = generateTypeScriptContent(endpoints, parameters, responses, ((_a = spec.components) === null || _a === void 0 ? void 0 : _a.schemas) || {});
        // Write the generated file
        (0, fs_1.writeFileSync)(targetPath, content, 'utf8');
        console.log("\u2705 Generated API endpoints to ".concat(targetPath));
    }
    catch (error) {
        console.error('❌ Error generating endpoints:', error);
        // eslint-disable-next-line no-process-exit
        process.exit(1);
    }
}
function detectQuerySchemaType(queryParams) {
    if (!queryParams.length)
        return null;
    var paramNames = queryParams.map(function (p) { return p.name; }).sort();
    // Detect standard pagination pattern
    if (paramNames.length === 2 &&
        paramNames.includes('page') &&
        paramNames.includes('per_page')) {
        return 'PaginationQuery';
    }
    // Detect conversation pagination pattern
    if (paramNames.length === 3 &&
        paramNames.includes('page') &&
        paramNames.includes('per_page') &&
        paramNames.includes('project_id')) {
        return 'ConversationPaginationQuery';
    }
    // Detect download pagination pattern
    if (paramNames.length === 3 &&
        paramNames.includes('page') &&
        paramNames.includes('per_page') &&
        paramNames.includes('status')) {
        return 'DownloadPaginationQuery';
    }
    // Detect project list query pattern
    if (paramNames.length === 3 &&
        paramNames.includes('page') &&
        paramNames.includes('per_page') &&
        paramNames.includes('search')) {
        return 'ProjectListQuery';
    }
    // Note: FileListParams pattern is same as ProjectListQuery,
    // will be handled by the previous case
    // Detect hub query params pattern
    if (paramNames.length === 1 && paramNames.includes('lang')) {
        return 'HubQueryParams';
    }
    // Detect search query pattern
    if (paramNames.length === 4 &&
        paramNames.includes('q') &&
        paramNames.includes('page') &&
        paramNames.includes('per_page') &&
        paramNames.includes('project_id')) {
        return 'SearchQuery';
    }
    return null;
}
function generateParameterType(operation, path) {
    var _a;
    var paramTypes = [];
    // Add path parameters
    var pathParams = path.match(/{([^}]+)}/g);
    if (pathParams) {
        for (var _i = 0, pathParams_1 = pathParams; _i < pathParams_1.length; _i++) {
            var param = pathParams_1[_i];
            var paramName = param.slice(1, -1); // Remove { }
            paramTypes.push("".concat(paramName, ": string"));
        }
    }
    // Collect query parameters and detect schema patterns
    var queryParams = [];
    var querySchemaType = null;
    if (operation.parameters) {
        for (var _b = 0, _c = operation.parameters; _b < _c.length; _b++) {
            var param = _c[_b];
            if (param.in === 'query') {
                var isOptional = !param.required;
                var paramType = getTypeFromSchema(param.schema, isOptional);
                queryParams.push("".concat(param.name).concat(isOptional ? '?' : '', ": ").concat(paramType));
            }
        }
        // Detect common query parameter patterns and map to schema types
        querySchemaType = detectQuerySchemaType(operation.parameters.filter(function (p) { return p.in === 'query'; }));
    }
    // Use schema type if detected, otherwise use individual parameters
    if (querySchemaType) {
        // Don't add individual query params, we'll use the schema type
    }
    else {
        paramTypes.push.apply(paramTypes, queryParams);
    }
    // Add request body type
    var requestBodyType = null;
    if (operation.requestBody) {
        // Try application/json first
        var content = operation.requestBody.content['application/json'];
        // If no application/json, try multipart/form-data
        if (!content) {
            content = operation.requestBody.content['multipart/form-data'];
        }
        if (content && isSchemaReference(content.schema)) {
            requestBodyType = extractSchemaName(content.schema.$ref);
        }
        else if (content) {
            // For multipart/form-data or complex inline schemas, use FormData or any
            if (operation.requestBody.content['multipart/form-data']) {
                requestBodyType = 'FormData';
            }
            else {
                requestBodyType = 'any'; // Generic fallback for complex inline schemas
            }
        }
    }
    // Return parameter object type or void
    if (paramTypes.length === 0 && !querySchemaType && !requestBodyType) {
        return 'void';
    }
    else if (paramTypes.length === 0 && !querySchemaType && requestBodyType) {
        // Only request body, no parameters
        return requestBodyType;
    }
    else if (querySchemaType && !requestBodyType && paramTypes.length === 0) {
        // Only query schema, no path params or request body
        return querySchemaType;
    }
    else if (querySchemaType && requestBodyType && paramTypes.length === 0) {
        // Query schema and request body, no path params
        return "".concat(querySchemaType, " & ").concat(requestBodyType);
    }
    else if (querySchemaType && paramTypes.length > 0 && !requestBodyType) {
        // Query schema and path params, no request body
        return "{ ".concat(paramTypes.join('; '), " } & ").concat(querySchemaType);
    }
    else if (querySchemaType && paramTypes.length > 0 && requestBodyType) {
        // Query schema, path params, and request body
        return "{ ".concat(paramTypes.join('; '), " } & ").concat(querySchemaType, " & ").concat(requestBodyType);
    }
    else if (paramTypes.length > 0 && requestBodyType) {
        // Both parameters and request body - combine them (fallback to old logic)
        return "{ ".concat(paramTypes.join('; '), " } & ").concat(requestBodyType);
    }
    else if (paramTypes.length === 1 &&
        !((_a = operation.parameters) === null || _a === void 0 ? void 0 : _a.some(function (p) { return p.in === 'query'; }))) {
        // Single path parameter, return as object
        return "{ ".concat(paramTypes[0], " }");
    }
    else {
        return "{ ".concat(paramTypes.join('; '), " }");
    }
}
function generateResponseType(operation, httpMethod) {
    if (!operation.responses || operation.responses['204']) {
        return 'void';
    }
    // Look for successful responses (200, 201, 204, etc.)
    var successResponse = operation.responses['200'] ||
        operation.responses['201'] ||
        operation.responses['202'];
    if (!successResponse) {
        return 'any';
    }
    // If there's no content, return void for non-POST or any for POST
    if (!successResponse.content) {
        return httpMethod === 'POST' ? 'any' : 'void';
    }
    // Look for application/json content
    var jsonContent = successResponse.content['application/json'];
    if (!jsonContent || !jsonContent.schema) {
        return httpMethod === 'POST' ? 'any' : 'any';
    }
    // Extract schema reference or type
    if (isSchemaReference(jsonContent.schema)) {
        return extractSchemaName(jsonContent.schema.$ref);
    }
    else {
        return getTypeFromSchema(jsonContent.schema);
    }
}
function getTypeFromSchema(schema, isOptionalParamOrNullable) {
    if (isOptionalParamOrNullable === void 0) { isOptionalParamOrNullable = false; }
    // Handle boolean literal values (like profile: true in User schema)
    if (typeof schema === 'boolean') {
        return 'any'; // or could be the literal boolean value
    }
    if (isSchemaReference(schema)) {
        return extractSchemaName(schema.$ref);
    }
    // Handle anyOf patterns (union types with schema references)
    if (schema.anyOf && Array.isArray(schema.anyOf)) {
        var types = schema.anyOf
            .map(function (subSchema) {
            if (isSchemaReference(subSchema)) {
                return extractSchemaName(subSchema.$ref);
            }
            else if (subSchema.type === 'null') {
                // If there's a null in anyOf and we're dealing with optional/nullable, skip it
                return isOptionalParamOrNullable ? null : 'null';
            }
            else {
                return getTypeFromSchema(subSchema, isOptionalParamOrNullable);
            }
        })
            .filter(function (type) { return type !== null; }); // Remove null entries when filtered out
        return types.length === 1 ? types[0] : types.join(' | ');
    }
    // Handle allOf patterns (intersection types with schema references)
    if (schema.allOf && Array.isArray(schema.allOf)) {
        var types = schema.allOf
            .map(function (subSchema) {
            if (isSchemaReference(subSchema)) {
                return extractSchemaName(subSchema.$ref);
            }
            else {
                return getTypeFromSchema(subSchema, isOptionalParamOrNullable);
            }
        })
            .filter(function (type) { return type !== null; });
        // For allOf with a single reference (common pattern for enums), return the single type
        if (types.length === 1) {
            return types[0];
        }
        // For multiple types, use intersection (though this is less common)
        return types.join(' & ');
    }
    // Handle enum types
    if (schema.enum && Array.isArray(schema.enum)) {
        // Convert enum values to union type with string literals
        return schema.enum.map(function (value) { return "'".concat(value, "'"); }).join(' | ');
    }
    if (typeof schema.type === 'string') {
        switch (schema.type) {
            case 'string':
                if (schema.format === 'date-time') {
                    return 'string'; // Could be Date if preferred
                }
                return 'string';
            case 'integer':
            case 'number':
                return 'number';
            case 'boolean':
                return 'boolean';
            case 'array':
                if (schema.items) {
                    var itemType = getTypeFromSchema(schema.items);
                    return "".concat(itemType, "[]");
                }
                return 'any[]';
            case 'object':
                if (schema.properties) {
                    // Generate inline object type
                    var props = [];
                    for (var _i = 0, _a = Object.entries(schema.properties); _i < _a.length; _i++) {
                        var _b = _a[_i], propName = _b[0], propSchema = _b[1];
                        var propType = getTypeFromSchema(propSchema);
                        props.push("".concat(propName, ": ").concat(propType));
                    }
                    return "{ ".concat(props.join('; '), " }");
                }
                return 'any';
            default:
                return 'any';
        }
    }
    else if (Array.isArray(schema.type)) {
        // Handle union types like ["string", "null"] or ["array", "null"]
        var types = schema.type
            .filter(function (t) {
            // If this is an optional parameter or nullable property, exclude null from the union
            // since optional parameters/properties don't need explicit null
            if (isOptionalParamOrNullable && t === 'null') {
                return false;
            }
            return true;
        })
            .map(function (t) {
            switch (t) {
                case 'string':
                    return 'string';
                case 'integer':
                case 'number':
                    return 'number';
                case 'boolean':
                    return 'boolean';
                case 'array':
                    // Handle array type in union - use the items schema
                    if (schema.items) {
                        var itemType = getTypeFromSchema(schema.items);
                        return "".concat(itemType, "[]");
                    }
                    return 'any[]';
                case 'null':
                    return 'null';
                default:
                    return 'any';
            }
        });
        return types.length === 1 ? types[0] : types.join(' | ');
    }
    return 'any';
}
function generateSchemaInterface(name, schema) {
    var _a;
    if (schema.$ref) {
        // This shouldn't happen for top-level schemas, but handle it just in case
        return "export type ".concat(name, " = ").concat(extractSchemaName(schema.$ref));
    }
    // Special handling for Permission type - convert to enum
    if (name === 'Permission' && schema.enum && Array.isArray(schema.enum)) {
        return generatePermissionEnum(schema.enum);
    }
    if (schema.type === 'object' && schema.properties) {
        var properties = [];
        for (var _i = 0, _b = Object.entries(schema.properties); _i < _b.length; _i++) {
            var _c = _b[_i], propName = _c[0], propSchema = _c[1];
            var isOptional = !((_a = schema.required) === null || _a === void 0 ? void 0 : _a.includes(propName));
            // Check if property is nullable (has null in union type or anyOf with null)
            var isNullableUnion = Array.isArray(propSchema.type) && propSchema.type.includes('null');
            var isNullableAnyOf = propSchema.anyOf &&
                Array.isArray(propSchema.anyOf) &&
                propSchema.anyOf.some(function (subSchema) { return subSchema.type === 'null'; });
            var isNullableAllOf = propSchema.allOf &&
                Array.isArray(propSchema.allOf) &&
                propSchema.allOf.some(function (subSchema) { return subSchema.type === 'null'; });
            var isNullable = isNullableUnion || isNullableAnyOf || isNullableAllOf;
            // If property is nullable, make it optional and exclude null from type
            if (isNullable) {
                isOptional = true;
            }
            var optionalMarker = isOptional ? '?' : '';
            var propType = getTypeFromSchema(propSchema, isNullable);
            properties.push("  ".concat(propName).concat(optionalMarker, ": ").concat(propType));
        }
        return "export interface ".concat(name, " {\n").concat(properties.join('\n'), "\n}");
    }
    else if (schema.type === 'array' && schema.items) {
        var itemType = getTypeFromSchema(schema.items);
        return "export type ".concat(name, " = ").concat(itemType, "[]");
    }
    else {
        // For primitive types or other cases
        var baseType = getTypeFromSchema(schema);
        return "export type ".concat(name, " = ").concat(baseType);
    }
}
function generatePermissionEnum(enumValues) {
    // Convert permission string values to PascalCase enum keys
    var enumEntries = [];
    for (var _i = 0, enumValues_1 = enumValues; _i < enumValues_1.length; _i++) {
        var value = enumValues_1[_i];
        if (typeof value === 'string') {
            var enumKey = convertPermissionToPascalCase(value);
            enumEntries.push("  ".concat(enumKey, " = '").concat(value, "'"));
        }
    }
    return "export enum Permission {\n".concat(enumEntries.join(',\n'), "\n}");
}
function convertPermissionToPascalCase(permission) {
    // Handle special case for wildcard
    if (permission === '*') {
        return 'All';
    }
    // Split by :: and - then convert to PascalCase
    return permission
        .split('::')
        .map(function (part) {
        return part
            .split('-')
            .map(function (word) { return word.charAt(0).toUpperCase() + word.slice(1); })
            .join('');
    })
        .join('');
}
function generateAllSchemas(schemas) {
    var interfaces = [];
    // Sort schema names for consistent output
    var sortedNames = Object.keys(schemas).sort();
    for (var _i = 0, sortedNames_1 = sortedNames; _i < sortedNames_1.length; _i++) {
        var schemaName = sortedNames_1[_i];
        // Skip primitive type schemas since they should be treated as built-in types
        if (schemaName === 'AnyType' || schemaName === 'BlobType') {
            continue;
        }
        var schema = schemas[schemaName];
        var interfaceDefinition = generateSchemaInterface(schemaName, schema);
        interfaces.push(interfaceDefinition);
    }
    return interfaces.join('\n\n');
}
function generateTypeScriptContent(endpoints, parameters, responses, schemas) {
    var sortedEndpoints = Object.keys(endpoints).sort();
    // Generate header and schema definitions
    var header = "/**\n * Generated API endpoint definitions\n * Auto-generated from OpenAPI specification\n * \n * \u26A0\uFE0F  DO NOT EDIT THIS FILE MANUALLY \u26A0\uFE0F\n * This file is automatically generated from the OpenAPI specification generated from the server code.\n */\n\n// =============================================================================\n// TYPE DEFINITIONS\n// =============================================================================\n\n";
    // Generate all schema interfaces
    var schemaDefinitions = generateAllSchemas(schemas) + '\n\n';
    // Generate endpoints object
    var endpointsSection = "// =============================================================================\n// API ENDPOINTS\n// =============================================================================\n\n// API endpoint definitions\nexport const ApiEndpoints = {\n".concat(sortedEndpoints.map(function (key) { return "  '".concat(key, "': '").concat(endpoints[key], "'"); }).join(',\n'), "\n} as const\n\n");
    // Generate parameter types
    var parametersSection = "// API endpoint parameters\nexport type ApiEndpointParameters = {\n".concat(sortedEndpoints.map(function (key) { return "  '".concat(key, "': ").concat(parameters[key]); }).join('\n'), "\n}\n\n");
    // Generate response types
    var responsesSection = "// API endpoint responses\nexport type ApiEndpointResponses = {\n".concat(sortedEndpoints.map(function (key) { return "  '".concat(key, "': ").concat(responses[key]); }).join('\n'), "\n}\n\n");
    // Generate helper types
    var helpersSection = "// Type helpers\nexport type ApiEndpoint = keyof typeof ApiEndpoints\nexport type ApiEndpointUrl = (typeof ApiEndpoints)[ApiEndpoint]\n\n// Extract endpoint key from URL pattern\nexport function getEndpointKey(url: string): ApiEndpoint | undefined {\n  const entries = Object.entries(ApiEndpoints) as [ApiEndpoint, string][]\n  const found = entries.find(([_key, value]) => value === url)\n  return found ? found[0] : undefined\n}\n\n// Get parameter type for endpoint\nexport type GetParameterType<K extends ApiEndpoint> = ApiEndpointParameters[K]\n\n// Get response type for endpoint  \nexport type GetResponseType<K extends ApiEndpoint> = ApiEndpointResponses[K]\n\n// Create reverse mapping from URL to endpoint key\nexport type UrlToEndpoint<U extends ApiEndpointUrl> = {\n  [K in keyof typeof ApiEndpoints]: (typeof ApiEndpoints)[K] extends U\n    ? K\n    : never\n}[keyof typeof ApiEndpoints]\n\n// Helper types to get parameter and response types by URL\nexport type ParameterByUrl<U extends ApiEndpointUrl> =\n  ApiEndpointParameters[UrlToEndpoint<U>]\nexport type ResponseByUrl<U extends ApiEndpointUrl> =\n  ApiEndpointResponses[UrlToEndpoint<U>]\n\n// Type-safe validation - this will cause a TypeScript error if any endpoint is missing\ntype ValidateParametersComplete = {\n  [K in keyof typeof ApiEndpoints]: K extends keyof ApiEndpointParameters\n    ? true\n    : false\n}\n\ntype ValidateResponsesComplete = {\n  [K in keyof typeof ApiEndpoints]: K extends keyof ApiEndpointResponses\n    ? true\n    : false\n}\n\n// Type-safe validation - these will cause a TypeScript error if any endpoint is missing\n// from Parameters or Responses. They are used for compile-time validation only.\nexport type { ValidateParametersComplete, ValidateResponsesComplete }\n";
    return (header +
        schemaDefinitions +
        endpointsSection +
        parametersSection +
        responsesSection +
        helpersSection);
}
// Run the generator
generateEndpoints();
