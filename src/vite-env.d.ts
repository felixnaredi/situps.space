/// <reference types="vite/client" />

interface ImportMetaEnv {
    readonly SITUPS_V2_API_URL: string;
    readonly SITUPS_V2_WS_URL: string;
    readonly SITUPS_V1_API_URL: string;
    readonly SITUPS_V1_WS_URL: string;
}

interface ImportMeta {
    readonly env: ImportMetaEnv;
}