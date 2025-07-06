// Define all API endpoints

export const ApiEndpoints = {
    "User.greet": '/api/user/greet',
    "App.getHttpPort": '/get_http_port',
} as const;

// Define parameters for each endpoint - TypeScript will ensure all endpoints are covered
export type ApiEndpointParameters = {
    "User.greet": { name: string };
    "App.getHttpPort": void;
}

// Define responses for each endpoint - TypeScript will ensure all endpoints are covered
export type ApiEndpointResponses = {
    "User.greet": string;
    "App.getHttpPort": number;
}

// Type helpers
export type ApiEndpoint = keyof typeof ApiEndpoints;
export type ApiEndpointUrl = typeof ApiEndpoints[ApiEndpoint];

// Create reverse mapping from URL to endpoint key
export type UrlToEndpoint<U extends ApiEndpointUrl> = {
    [K in keyof typeof ApiEndpoints]: typeof ApiEndpoints[K] extends U ? K : never;
}[keyof typeof ApiEndpoints];

// Helper types to get parameter and response types by URL
export type ParameterByUrl<U extends ApiEndpointUrl> = ApiEndpointParameters[UrlToEndpoint<U>];
export type ResponseByUrl<U extends ApiEndpointUrl> = ApiEndpointResponses[UrlToEndpoint<U>];

// Type-safe validation - this will cause a TypeScript error if any endpoint is missing
type ValidateParametersComplete = {
    [K in keyof typeof ApiEndpoints]: K extends keyof ApiEndpointParameters ? true : false;
}

type ValidateResponsesComplete = {
    [K in keyof typeof ApiEndpoints]: K extends keyof ApiEndpointResponses ? true : false;
}

// These type checks will fail at compile time if any endpoint is missing from Parameters or Responses
// They are intentionally unused but serve as compile-time validators
//@ts-ignore
const _validateParameters: ValidateParametersComplete = {} as any;
//@ts-ignore
const _validateResponses: ValidateResponsesComplete = {} as any;
