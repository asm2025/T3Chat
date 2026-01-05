import type { EndpointOption } from "@/types/librechat";

/**
 * Returns a fresh copy of the base endpoint options so callers can mutate safely.
 */
export const createDefaultEndpointOptions = (): EndpointOption => ({
    endpoint: "openai",
    model: "gpt-4-turbo",
    parameters: {
        temperature: 0.7,
        max_tokens: 2048,
    },
    featureFlags: {},
});

