declare const __APP_VERSION__: string
declare const __BUILD_TIME__: string

export function useBuildInfo() {
  const version = __APP_VERSION__ || '0.0.0'
  const buildTime = __BUILD_TIME__ || 'unknown'

  const buildInfo = `v${version} (${buildTime})`

  return {
    version,
    buildTime,
    buildInfo,
  }
}
