import type { SidebarState } from '../types/session'
import { invoke } from '@tauri-apps/api/core'
import { ref } from 'vue'

// 单例实例
let sidebarStateInstance: ReturnType<typeof createSidebarState> | null = null

function createSidebarState() {
  // 状态
  const isExpanded = ref(false)
  const width = ref(300) // 默认宽度 300px

  // 常量
  const MIN_WIDTH = 200
  const MAX_WIDTH_RATIO = 0.5

  /**
   * 切换侧边栏展开/收起状态
   */
  async function toggleSidebar() {
    console.log('[useSidebarState] 切换侧边栏，当前状态:', isExpanded.value)
    isExpanded.value = !isExpanded.value
    console.log('[useSidebarState] 新状态:', isExpanded.value)
    await saveSidebarState()
  }

  /**
   * 设置侧边栏宽度（带约束）
   *
   * 宽度将被限制在 200px 到窗口宽度的 50% 之间
   *
   * @param newWidth 新的宽度值（像素）
   */
  function setWidth(newWidth: number) {
    const maxWidth = window.innerWidth * MAX_WIDTH_RATIO
    width.value = Math.max(MIN_WIDTH, Math.min(newWidth, maxWidth))
  }

  /**
   * 保存侧边栏状态到后端
   *
   * @throws 如果保存失败则抛出错误
   */
  async function saveSidebarState(): Promise<void> {
    try {
      const state: SidebarState = {
        isExpanded: isExpanded.value,
        width: width.value,
      }

      await invoke('save_sidebar_state', { stateData: state })
    }
    catch (e) {
      console.error('保存侧边栏状态失败:', e)
      // 不抛出错误，避免影响用户体验
    }
  }

  /**
   * 从后端加载侧边栏状态
   *
   * @throws 如果加载失败则抛出错误
   */
  async function loadSidebarState(): Promise<void> {
    try {
      const state = await invoke<SidebarState | null>('load_sidebar_state')

      if (state) {
        isExpanded.value = state.isExpanded
        width.value = state.width
      }
    }
    catch (e) {
      console.error('加载侧边栏状态失败:', e)
      // 使用默认值，不抛出错误
    }
  }

  return {
    // 状态
    isExpanded,
    width,

    // 常量
    MIN_WIDTH,
    MAX_WIDTH_RATIO,

    // 方法
    toggleSidebar,
    setWidth,
    saveSidebarState,
    loadSidebarState,
  }
}

/**
 * 侧边栏状态管理组合式函数
 *
 * 提供侧边栏的展开/收起状态和宽度管理功能，包括：
 * - 切换侧边栏展开/收起状态
 * - 设置侧边栏宽度（带约束）
 * - 持久化状态到本地存储
 * - 从本地存储加载状态
 *
 * 状态会自动持久化，应用重启后会恢复上次的状态。
 *
 * @example
 * ```typescript
 * const { isExpanded, width, toggleSidebar, setWidth } = useSidebarState()
 *
 * // 在组件挂载时加载状态
 * onMounted(async () => {
 *   await loadSidebarState()
 * })
 *
 * // 切换侧边栏
 * await toggleSidebar()
 *
 * // 设置宽度（会自动应用约束）
 * setWidth(400)
 * ```
 */
export function useSidebarState() {
  if (!sidebarStateInstance) {
    sidebarStateInstance = createSidebarState()
  }
  return sidebarStateInstance
}
