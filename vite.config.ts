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
          rewrite: (path) => path.replace("/api", ""),
        },
        "/socket.io": {
          target: env.SITUPS_WS_URL,
          changeOrigin: true,
          ws: true,
          rewrite: (path) => path.replace("/socket.io", ""),
        },
      },
    },
    envPrefix: "SITUPS"
  };
});
