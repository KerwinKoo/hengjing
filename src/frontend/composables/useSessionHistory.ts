import type { SessionRecord } from '../types/session'
import { invoke } from '@tauri-apps/api/core'
import { ref } from 'vue'

// 单例实例
let sessionHistoryInstance: ReturnType<typeof createSessionHistory> | null = null

function createSessionHistory() {
  // 状态
  const sessions = ref<SessionRecord[]>([])
  const loading = ref(false)

  /**
   * 加载所有会话记录（不包含截图数据）
   */
  async function loadSessions(): Promise<void> {
    try {
      loading.value = true
      console.log('[SessionHistory] 开始加载会话列表...')
      const loadedSessions = await invoke<SessionRecord[]>('load_sessions')
      sessions.value = loadedSessions
      console.log('[SessionHistory] 会话列表加载完成，共', loadedSessions.length, '条记录')
      if (loadedSessions.length > 0) {
        console.log('[SessionHistory] 最新会话:', loadedSessions[0])
      }
    }
    catch (e) {
      console.error('加载会话列表失败:', e)
      throw e
    }
    finally {
      loading.value = false
    }
  }

  /**
   * 获取单个会话（包含截图数据）
   */
  async function getSession(id: string): Promise<SessionRecord | null> {
    try {
      loading.value = true
      const session = await invoke<SessionRecord | null>('get_session', { id })
      return session
    }
    catch (e) {
      console.error('获取会话失败:', e)
      throw e
    }
    finally {
      loading.value = false
    }
  }

  /**
   * 删除单个会话
   */
  async function deleteSession(id: string): Promise<void> {
    try {
      loading.value = true
      await invoke('delete_session', { id })
      sessions.value = sessions.value.filter(s => s.id !== id)
    }
    catch (e) {
      console.error('删除会话失败:', e)
      throw e
    }
    finally {
      loading.value = false
    }
  }

  /**
   * 批量删除会话
   */
  async function batchDeleteSessions(ids: string[]): Promise<void> {
    try {
      loading.value = true
      await invoke('batch_delete_sessions', { ids })
      sessions.value = sessions.value.filter(s => !ids.includes(s.id))
    }
    catch (e) {
      console.error('批量删除会话失败:', e)
      throw e
    }
    finally {
      loading.value = false
    }
  }

  /**
   * 清空所有会话
   */
  async function clearAllSessions(): Promise<void> {
    try {
      loading.value = true
      await invoke('clear_all_sessions')
      sessions.value = []
    }
    catch (e) {
      console.error('清空所有会话失败:', e)
      throw e
    }
    finally {
      loading.value = false
    }
  }

  /**
   * 搜索会话（前端过滤）
   */
  function searchSessions(query: string): SessionRecord[] {
    if (!query.trim()) {
      return sessions.value
    }

    const lowerQuery = query.toLowerCase()

    return sessions.value.filter((session) => {
      if (session.userInput && session.userInput.toLowerCase().includes(lowerQuery)) {
        return true
      }
      if (session.aiResponse.toLowerCase().includes(lowerQuery)) {
        return true
      }
      if (session.selectedOptions.some(option => option.toLowerCase().includes(lowerQuery))) {
        return true
      }
      return false
    })
  }

  /**
   * 保存新会话（创建会话记录）
   */
  async function saveSession(sessionData: any): Promise<{ id: string }> {
    try {
      loading.value = true
      console.log('[SessionHistory] 开始保存会话...', sessionData)
      console.log('[SessionHistory] 准备调用 invoke("save_session")...')

      // 添加超时检测
      const timeoutPromise = new Promise((_, reject) => {
        setTimeout(() => reject(new Error('invoke 调用超时 (5秒)')), 5000)
      })

      const invokePromise = invoke<SessionRecord>('save_session', { session: sessionData })

      const record = await Promise.race([invokePromise, timeoutPromise]) as SessionRecord
      console.log('[SessionHistory] invoke 调用成功，返回记录:', record)
      // 立即添加到前端列表（插入到开头，最新的在前面）
      sessions.value.unshift(record)
      console.log('[SessionHistory] 会话已保存并添加到列表:', record.id, '当前列表长度:', sessions.value.length)
      return { id: record.id }
    }
    catch (e) {
      console.error('[SessionHistory] 保存会话失败 - 错误类型:', typeof e)
      console.error('[SessionHistory] 保存会话失败 - 错误详情:', e)
      console.error('[SessionHistory] 保存会话失败 - 错误堆栈:', (e as Error).stack)
      throw e
    }
    finally {
      loading.value = false
    }
  }

  /**
   * 更新会话（添加用户响应）
   */
  async function updateSession(id: string, updateData: any): Promise<void> {
    try {
      loading.value = true
      const updatedRecord = await invoke<SessionRecord>('update_session', { id, update: updateData })
      // 更新前端列表中的记录
      const index = sessions.value.findIndex(s => s.id === id)
      if (index !== -1) {
        sessions.value[index] = updatedRecord
        console.log('[SessionHistory] 会话已更新:', id)
      }
    }
    catch (e) {
      console.error('更新会话失败:', e)
      throw e
    }
    finally {
      loading.value = false
    }
  }

  /**
   * 初始化会话历史（加载历史会话）
   */
  async function initialize() {
    try {
      console.log('[SessionHistory] 初始化会话历史...')
      await loadSessions()
      console.log('[SessionHistory] 初始化完成，当前列表长度:', sessions.value.length)
    }
    catch (e) {
      console.error('初始化会话历史失败:', e)
    }
  }

  return {
    sessions,
    loading,
    saveSession,
    updateSession,
    getSession,
    deleteSession,
    batchDeleteSessions,
    clearAllSessions,
    searchSessions,
    initialize,
  }
}

/**
 * 会话历史管理组合式函数
 */
export function useSessionHistory() {
  if (!sessionHistoryInstance) {
    sessionHistoryInstance = createSessionHistory()
  }
  return sessionHistoryInstance
}
