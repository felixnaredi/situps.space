/// <reference types="vite/client" />

interface ImportMetaEnv {
    readonly SITUPS_API_URL: string;
    readonly SITUPS_API_V2_URL: string;
    readonly SITUPS_WS_URL: string;
}

interface ImportMeta {
    readonly env: ImportMetaEnv;
}