/** @type {import('next').NextConfig} */
const nextConfig = {
  typescript: {
    ignoreBuildErrors: true,
  },
  images: {
    unoptimized: true,
    remotePatterns: [
      { hostname: "image.tmdb.org" },
      { hostname: "cdn.myanimelist.net" },
    ],
  },
  // Static export for embedding into the Rust binary
  ...(process.env.NODE_ENV === "production" ? { output: "export" } : {}),
  // Dev proxy: forward /api/* to the Rust backend
  async rewrites() {
    if (process.env.NODE_ENV === "production") return []
    return [
      {
        source: "/api/:path*",
        destination: `http://localhost:${process.env.KARS_API_PORT || 3001}/api/:path*`,
      },
    ]
  },
}

export default nextConfig
