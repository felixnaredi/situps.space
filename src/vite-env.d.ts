/// <reference types="vite/client" />

interface ImportMetaEnv {
    readonly SITUPS_API_URL: string;
}

interface ImportMeta {
    readonly env: ImportMetaEnv;
}