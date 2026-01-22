import { defineConfig } from "astro/config";

export default defineConfig({
  outDir: "target",
  site: "https://wtrscape.com",
  compressHTML: true,
  build: {
    inlineStylesheets: "auto",
  },
});
