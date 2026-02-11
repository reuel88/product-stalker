// Placeholder for future environment variables.
// The desktop app currently uses SQLite-backed settings instead of env vars.
// This package exists as infrastructure for when env-based config is needed.
import { createEnv } from "@t3-oss/env-core";

export const env = createEnv({
	emptyStringAsUndefined: true,
});
