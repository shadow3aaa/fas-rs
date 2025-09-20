import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  output: "export",
  distDir: "webroot",
  trailingSlash: true,
};

export default nextConfig;
