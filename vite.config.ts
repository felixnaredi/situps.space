import { defineConfig, loadEnv } from "vite";
import vue from "@vitejs/plugin-vue";

// https://vitejs.dev/config/
export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, process.cwd(), "SITUP");
  console.log(env);

  return {
    plugins: [vue()],
    server: {
      proxy: {
        "/api": {
          target: env.SITUPS_API_URL,
          changeOrigin: true,
          secure: env.SITUPS_API_PROXY_SECURE === "true"
        },
      },
    },
    envPrefix: "SITUPS",
  };
});
