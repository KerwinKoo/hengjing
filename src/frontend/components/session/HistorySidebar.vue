<script setup lang="ts">
import { NButton, NPopconfirm, useDialog } from 'naive-ui'
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { useSessionHistory } from '../../composables/useSessionHistory'
import { useSidebarState } from '../../composables/useSidebarState'
import SearchBar from './SearchBar.vue'
import SessionList from './SessionList.vue'
import SessionViewer from './SessionViewer.vue'

interface AppConfig {
  theme: string
  window: {
    alwaysOnTop: boolean
    width: number
    height: number
    fixed: boolean
  }
  audio: {
    enabled: boolean
    url: string
  }
  reply: {
    enabled: boolean
    prompt: string
  }
}

interface Props {
  appConfig: AppConfig
}

const props = defineProps<Props>()

const dialog = useDialog()

// 使用组合式函数
const {
  sessions,
  loading,
  getSession,
  deleteSession,
  batchDeleteSessions,
  clearAllSessions,
  searchSessions,
  initialize,
  cleanup,
} = useSessionHistory()

const {
  isExpanded,
  width,
  MIN_WIDTH,
  toggleSidebar,
  setWidth,
  loadSidebarState,
} = useSidebarState()

// 本地状态
const searchQuery = ref('')
const selectedSessionId = ref<string | null>(null)
const selectedSession = ref<any>(null)
const selectionMode = ref(false)
const isResizing = ref(false)

// 窗口尺寸管理
const originalWindowWidth = ref(0)
const appWindow = getCurrentWindow()

// 监听侧边栏展开状态，调整窗口大小
watch(isExpanded, async (expanded) => {
  try {
    const currentSize = await appWindow.innerSize()
    
    if (expanded) {
      // 保存原始窗口宽度
      if (originalWindowWidth.value === 0) {
        originalWindowWidth.value = currentSize.width
      }
      // 扩展窗口宽度
      await appWindow.setSize({
        width: originalWindowWidth.value + width.value,
        height: currentSize.height,
      })
    }
    else {
      // 恢复原始窗口宽度
      if (originalWindowWidth.value > 0) {
        await appWindow.setSize({
          width: originalWindowWidth.value,
          height: currentSize.height,
        })
      }
    }
  }
  catch (e) {
    console.error('调整窗口大小失败:', e)
  }
})

// 监听侧边栏宽度变化，调整窗口大小
watch(width, async (newWidth, oldWidth) => {
  if (isExpanded.value && originalWindowWidth.value > 0) {
    try {
      const currentSize = await appWindow.innerSize()
      const widthDiff = newWidth - oldWidth
      await appWindow.setSize({
        width: currentSize.width + widthDiff,
        height: currentSize.height,
      })
    }
    catch (e) {
      console.error('调整窗口大小失败:', e)
    }
  }
})

// 过滤后的会话列表
const filteredSessions = computed(() => {
  return searchSessions(searchQuery.value)
})

// 处理搜索
function handleSearch(query: string) {
  searchQuery.value = query
  // 如果搜索后没有结果，清空选中的会话
  if (filteredSessions.value.length === 0) {
    selectedSessionId.value = null
    selectedSession.value = null
  }
}

// 处理会话选择
async function handleSelectSession(sessionId: string) {
  selectedSessionId.value = sessionId
  try {
    selectedSession.value = await getSession(sessionId)
  }
  catch (e) {
    console.error('加载会话详情失败:', e)
  }
}

// 处理会话删除
async function handleDeleteSession(sessionId: string) {
  try {
    await deleteSession(sessionId)

    // 如果删除的是当前选中的会话，清空选中状态
    if (selectedSessionId.value === sessionId) {
      selectedSessionId.value = null
      selectedSession.value = null
    }
  }
  catch (e) {
    console.error('删除会话失败:', e)
  }
}

// 处理批量删除
async function handleBatchDelete(sessionIds: string[]) {
  try {
    await batchDeleteSessions(sessionIds)

    // 如果删除的包含当前选中的会话，清空选中状态
    if (selectedSessionId.value && sessionIds.includes(selectedSessionId.value)) {
      selectedSessionId.value = null
      selectedSession.value = null
    }
  }
  catch (e) {
    console.error('批量删除会话失败:', e)
  }
}

// 处理清空所有历史
function handleClearAll() {
  dialog.warning({
    title: '确认清空所有历史',
    content: '确定要清空所有会话记录吗？此操作无法撤销。',
    positiveText: '清空',
    negativeText: '取消',
    onPositiveClick: async () => {
      try {
        await clearAllSessions()
        selectedSessionId.value = null
        selectedSession.value = null
      }
      catch (e) {
        console.error('清空所有会话失败:', e)
      }
    },
  })
}

// 切换多选模式
function toggleSelectionMode() {
  selectionMode.value = !selectionMode.value

  // 退出多选模式时清空选中状态
  if (!selectionMode.value) {
    selectedSessionId.value = null
    selectedSession.value = null
  }
}

// 拖拽调整宽度
const resizeStartX = ref(0)
const resizeStartWidth = ref(0)

function startResize(event: MouseEvent) {
  isResizing.value = true
  resizeStartX.value = event.clientX
  resizeStartWidth.value = width.value

  document.addEventListener('mousemove', handleResize)
  document.addEventListener('mouseup', stopResize)

  // 防止文本选择
  event.preventDefault()
}

function handleResize(event: MouseEvent) {
  if (!isResizing.value)
    return

  const deltaX = resizeStartX.value - event.clientX
  const newWidth = resizeStartWidth.value + deltaX

  setWidth(newWidth)
}

function stopResize() {
  if (isResizing.value) {
    isResizing.value = false
    document.removeEventListener('mousemove', handleResize)
    document.removeEventListener('mouseup', stopResize)
    
    // 保存新的宽度状态
    saveSidebarState()
  }
}

// 生命周期
onMounted(async () => {
  console.log('[HistorySidebar] 组件已挂载')
  
  // 获取初始窗口宽度
  try {
    const currentSize = await appWindow.innerSize()
    originalWindowWidth.value = currentSize.width
    console.log('[HistorySidebar] 初始窗口宽度:', originalWindowWidth.value)
  }
  catch (e) {
    console.error('获取窗口大小失败:', e)
  }
  
  await loadSidebarState()
  console.log('[HistorySidebar] 侧边栏状态已加载:', { isExpanded: isExpanded.value, width: width.value })
  await initialize()
  console.log('[HistorySidebar] 会话历史已初始化，会话数量:', sessions.value.length)
})

onUnmounted(() => {
  console.log('[HistorySidebar] 组件即将卸载')
  cleanup()
  stopResize()
})
</script>

<template>
  <div 
    class="history-sidebar" 
    :class="{ expanded: isExpanded }"
    :style="{ '--sidebar-width': `${width}px` }"
  >
    <!-- 展开/收起按钮 -->
    <div class="toggle-button" @click="toggleSidebar">
      <div
        :class="isExpanded ? 'i-carbon-chevron-right' : 'i-carbon-chevron-left'"
        class="w-5 h-5"
      />
    </div>

    <!-- 侧边栏内容 -->
    <transition name="sidebar-slide">
      <div
        v-if="isExpanded"
        class="sidebar-content"
        :style="{ width: `${width}px` }"
      >
        <!-- 拖拽调整宽度的手柄 -->
        <div
          class="resize-handle"
          @mousedown="startResize"
        />

        <!-- 侧边栏头部 -->
        <div class="sidebar-header">
          <div class="header-title">
            <div class="i-carbon-recently-viewed w-5 h-5" />
            <span>会话历史</span>
          </div>

          <div class="header-actions">
            <!-- 多选模式切换按钮 -->
            <NButton
              text
              size="small"
              :type="selectionMode ? 'primary' : 'default'"
              @click="toggleSelectionMode"
            >
              <template #icon>
                <div class="i-carbon-checkbox-checked w-4 h-4" />
              </template>
            </NButton>

            <!-- 清空所有历史按钮 -->
            <NPopconfirm @positive-click="handleClearAll">
              <template #trigger>
                <NButton
                  text
                  size="small"
                  :disabled="sessions.length === 0"
                >
                  <template #icon>
                    <div class="i-carbon-trash-can w-4 h-4" />
                  </template>
                </NButton>
              </template>
              确定要清空所有历史记录吗？
            </NPopconfirm>
          </div>
        </div>

        <!-- 搜索栏 -->
        <div class="sidebar-search">
          <SearchBar @search="handleSearch" />
        </div>

        <!-- 会话列表和查看器 -->
        <div class="sidebar-body">
          <!-- 会话列表 -->
          <div class="session-list-container" :class="{ 'with-viewer': selectedSession }">
            <SessionList
              :sessions="filteredSessions"
              :search-query="searchQuery"
              :selected-session-id="selectedSessionId"
              :selection-mode="selectionMode"
              @select-session="handleSelectSession"
              @delete-session="handleDeleteSession"
              @batch-delete="handleBatchDelete"
            />
          </div>

          <!-- 会话查看器 -->
          <transition name="viewer-slide">
            <div v-if="selectedSession && !selectionMode" class="session-viewer-container">
              <div class="viewer-header">
                <span class="viewer-title">会话详情</span>
                <NButton
                  text
                  size="small"
                  @click="() => { selectedSessionId = null; selectedSession = null }"
                >
                  <template #icon>
                    <div class="i-carbon-close w-4 h-4" />
                  </template>
                </NButton>
              </div>

              <SessionViewer
                :session="selectedSession"
                :app-config="appConfig"
              />
            </div>
          </transition>
        </div>
      </div>
    </transition>
  </div>
</template>

<style scoped>
.history-sidebar {
  position: fixed;
  top: 0;
  right: 0;
  height: 100vh;
  z-index: 1000;
}

.toggle-button {
  position: absolute;
  top: 50%;
  right: 0;
  transform: translateY(-50%);
  width: 32px;
  height: 64px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgb(var(--surface-container));
  border: 1px solid rgb(var(--surface-border));
  border-right: none;
  border-radius: 8px 0 0 8px;
  cursor: pointer;
  transition: all 0.2s;
  color: rgb(var(--on-surface));
  box-shadow: -2px 0 4px rgba(0, 0, 0, 0.1);
}

.toggle-button:hover {
  background: rgb(var(--surface-container-hover));
  width: 36px;
  box-shadow: -2px 0 8px rgba(0, 0, 0, 0.2);
}

.history-sidebar.expanded .toggle-button {
  right: var(--sidebar-width);
}

.sidebar-content {
  position: absolute;
  right: 0;
  top: 0;
  height: 100vh;
  background: rgb(var(--surface));
  border-left: 1px solid rgb(var(--surface-border));
  display: flex;
  flex-direction: column;
}

.resize-handle {
  position: absolute;
  left: 0;
  top: 0;
  bottom: 0;
  width: 4px;
  cursor: ew-resize;
  z-index: 10;
}

.resize-handle:hover {
  background: rgb(var(--primary));
}

.resize-handle:active {
  background: rgb(var(--primary));
}

.sidebar-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px;
  border-bottom: 1px solid rgb(var(--surface-border));
  background: rgb(var(--surface-container));
}

.header-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 16px;
  font-weight: 600;
  color: rgb(var(--on-surface));
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 4px;
}

.sidebar-search {
  padding: 12px 16px;
  border-bottom: 1px solid rgb(var(--surface-border));
}

.sidebar-body {
  flex: 1;
  display: flex;
  overflow: hidden;
}

.session-list-container {
  flex: 1;
  overflow: hidden;
  transition: flex 0.3s;
}

.session-list-container.with-viewer {
  flex: 0 0 40%;
  border-right: 1px solid rgb(var(--surface-border));
}

.session-viewer-container {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.viewer-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border-bottom: 1px solid rgb(var(--surface-border));
  background: rgb(var(--surface-container));
}

.viewer-title {
  font-size: 14px;
  font-weight: 600;
  color: rgb(var(--on-surface));
}

/* 动画 */
.sidebar-slide-enter-active,
.sidebar-slide-leave-active {
  transition: all 0.3s ease;
}

.sidebar-slide-enter-from {
  transform: translateX(100%);
  opacity: 0;
}

.sidebar-slide-leave-to {
  transform: translateX(100%);
  opacity: 0;
}

.viewer-slide-enter-active,
.viewer-slide-leave-active {
  transition: all 0.3s ease;
}

.viewer-slide-enter-from {
  transform: translateX(100%);
  opacity: 0;
}

.viewer-slide-leave-to {
  transform: translateX(100%);
  opacity: 0;
}
</style>
