/// <reference types="vite/client" />

interface ImportMetaEnv {
    readonly SITUPS_API_URL: string;
    readonly SITUPS_WS_URL: string;
    readonly SITUPS_API_PROXY_SECURE: boolean;
}

interface ImportMeta {
    readonly env: ImportMetaEnv;
}