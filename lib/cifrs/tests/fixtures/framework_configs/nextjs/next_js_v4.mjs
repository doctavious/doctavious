// Since Next.js 12.1.0, you can use an async function:


module.exports = async (phase, { defaultConfig }) => {
  /**
   * @type {import('next').NextConfig}
   */
  const nextConfig = {
    /* config options here */
    distDir: 'build',
  }
  return nextConfig
}
