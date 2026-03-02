// Vitest setup file
import { beforeAll, vi } from 'vitest'

// Mock Tauri API
beforeAll(() => {
  // Mock invoke function
  vi.mock('@tauri-apps/api/core', () => ({
    invoke: vi.fn(),
  }))

  // Mock listen function
  vi.mock('@tauri-apps/api/event', () => ({
    listen: vi.fn(),
  }))
})
