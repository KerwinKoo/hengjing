import type { SessionData, SessionRecord } from '../types/session'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { ref } from 'vue'

// 单例实例
let sessionHistoryInstance: ReturnType<typeof createSessionHistory> | null = null

function createSessionHistory() {
  // 状态
  const sessions = ref<SessionRecord[]>([])
  const loading = ref(false)
  const error = ref<Error | null>(null)

  // 事件监听器清理函数
  let unlistenSessionCompleted: (() => void) | null = null

  /**
   * 处理会话完成事件
   */
  async function handleSessionCompleted(event: { payload: SessionData }) {
    try {
      await saveSession(event.payload)
    }
    catch (e) {
      console.error('处理会话完成事件失败:', e)
    }
  }

  /**
   * 保存会话记录
   *
   * @param session 会话数据
   * @throws 如果保存失败则抛出错误
   */
  async function saveSession(session: SessionData): Promise<void> {
    try {
      loading.value = true
      error.value = null

      const record = await invoke<SessionRecord>('save_session', { session })

      // 将新会话添加到列表开头（最新的在最前）
      sessions.value.unshift(record)
    }
    catch (e) {
      error.value = e as Error
      console.error('保存会话失败:', e)
      throw e
    }
    finally {
      loading.value = false
    }
  }

  /**
   * 加载所有会话记录（不包含截图数据）
   *
   * @throws 如果加载失败则抛出错误
   */
  async function loadSessions(): Promise<void> {
    try {
      loading.value = true
      error.value = null

      const loadedSessions = await invoke<SessionRecord[]>('load_sessions')
      sessions.value = loadedSessions
    }
    catch (e) {
      error.value = e as Error
      console.error('加载会话列表失败:', e)
      throw e
    }
    finally {
      loading.value = false
    }
  }

  /**
   * 获取单个会话（包含截图数据）
   *
   * @param id 会话ID
   * @returns 会话记录，如果不存在则返回 null
   * @throws 如果获取失败则抛出错误
   */
  async function getSession(id: string): Promise<SessionRecord | null> {
    try {
      loading.value = true
      error.value = null

      const session = await invoke<SessionRecord | null>('get_session', { id })
      return session
    }
    catch (e) {
      error.value = e as Error
      console.error('获取会话失败:', e)
      throw e
    }
    finally {
      loading.value = false
    }
  }

  /**
   * 删除单个会话
   *
   * @param id 会话ID
   * @throws 如果删除失败则抛出错误
   */
  async function deleteSession(id: string): Promise<void> {
    try {
      loading.value = true
      error.value = null

      await invoke('delete_session', { id })

      // 从本地状态中移除已删除的会话
      sessions.value = sessions.value.filter(s => s.id !== id)
    }
    catch (e) {
      error.value = e as Error
      console.error('删除会话失败:', e)
      throw e
    }
    finally {
      loading.value = false
    }
  }

  /**
   * 批量删除会话
   *
   * @param ids 会话ID列表
   * @throws 如果批量删除失败则抛出错误
   */
  async function batchDeleteSessions(ids: string[]): Promise<void> {
    try {
      loading.value = true
      error.value = null

      await invoke('batch_delete_sessions', { ids })

      // 从本地状态中移除已删除的会话
      sessions.value = sessions.value.filter(s => !ids.includes(s.id))
    }
    catch (e) {
      error.value = e as Error
      console.error('批量删除会话失败:', e)
      throw e
    }
    finally {
      loading.value = false
    }
  }

  /**
   * 清空所有会话
   *
   * @throws 如果清空失败则抛出错误
   */
  async function clearAllSessions(): Promise<void> {
    try {
      loading.value = true
      error.value = null

      await invoke('clear_all_sessions')

      // 清空本地状态
      sessions.value = []
    }
    catch (e) {
      error.value = e as Error
      console.error('清空所有会话失败:', e)
      throw e
    }
    finally {
      loading.value = false
    }
  }

  /**
   * 搜索会话（前端过滤）
   *
   * 在用户输入、AI回复和选项中搜索关键词
   *
   * @param query 搜索关键词
   * @returns 匹配的会话列表
   */
  function searchSessions(query: string): SessionRecord[] {
    if (!query.trim()) {
      return sessions.value
    }

    const lowerQuery = query.toLowerCase()

    return sessions.value.filter((session) => {
      // 在用户输入中搜索
      if (session.userInput && session.userInput.toLowerCase().includes(lowerQuery)) {
        return true
      }

      // 在AI回复中搜索
      if (session.aiResponse.toLowerCase().includes(lowerQuery)) {
        return true
      }

      // 在选项中搜索
      if (session.selectedOptions.some(option => option.toLowerCase().includes(lowerQuery))) {
        return true
      }

      return false
    })
  }

  /**
   * 设置会话完成事件监听器
   */
  async function setupEventListener() {
    try {
      // 监听会话完成事件
      unlistenSessionCompleted = await listen<SessionData>('sessionCompleted', handleSessionCompleted)
      console.log('会话完成事件监听器已设置')
    }
    catch (e) {
      console.error('设置会话完成事件监听器失败:', e)
    }
  }

  /**
   * 移除事件监听器
   */
  function removeEventListener() {
    if (unlistenSessionCompleted) {
      unlistenSessionCompleted()
      unlistenSessionCompleted = null
      console.log('会话完成事件监听器已移除')
    }
  }

  /**
   * 初始化会话历史
   *
   * 加载历史会话并设置事件监听器
   */
  async function initialize() {
    try {
      await loadSessions()
      await setupEventListener()
    }
    catch (e) {
      console.error('初始化会话历史失败:', e)
    }
  }

  /**
   * 清理资源
   */
  function cleanup() {
    removeEventListener()
  }

  return {
    // 状态
    sessions,
    loading,
    error,

    // 方法
    saveSession,
    loadSessions,
    getSession,
    deleteSession,
    batchDeleteSessions,
    clearAllSessions,
    searchSessions,
    initialize,
    cleanup,
  }
}

/**
 * 会话历史管理组合式函数
 *
 * 提供会话历史记录的管理功能，包括：
 * - 自动保存会话记录
 * - 加载和查询历史会话
 * - 删除和清空会话
 * - 搜索会话内容
 *
 * @example
 * ```typescript
 * const { sessions, loading, loadSessions, searchSessions } = useSessionHistory()
 *
 * // 在组件挂载时初始化
 * onMounted(async () => {
 *   await loadSessions()
 * })
 *
 * // 搜索会话
 * const results = searchSessions('关键词')
 * ```
 */
export function useSessionHistory() {
  if (!sessionHistoryInstance) {
    sessionHistoryInstance = createSessionHistory()
  }
  return sessionHistoryInstance
}
