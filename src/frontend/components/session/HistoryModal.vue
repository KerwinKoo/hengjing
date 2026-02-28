<script setup lang="ts">
import { NButton, NModal, NPopconfirm, useDialog } from 'naive-ui'
import { computed, ref, watch } from 'vue'
import { useSessionHistory } from '../../composables/useSessionHistory'
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

const show = defineModel<boolean>('show', { default: false })

const dialog = useDialog()

const {
  sessions,
  getSession,
  deleteSession,
  batchDeleteSessions,
  clearAllSessions,
  searchSessions,
  initialize,
} = useSessionHistory()

const searchQuery = ref('')
const selectedSessionId = ref<string | null>(null)
const selectedSession = ref<any>(null)
const selectionMode = ref(false)

const filteredSessions = computed(() => searchSessions(searchQuery.value))

// 打开时初始化数据
watch(show, async (val) => {
  if (val) {
    await initialize()
    selectedSessionId.value = null
    selectedSession.value = null
    selectionMode.value = false
    searchQuery.value = ''
  }
})

function handleSearch(query: string) {
  searchQuery.value = query
  if (filteredSessions.value.length === 0) {
    selectedSessionId.value = null
    selectedSession.value = null
  }
}

async function handleSelectSession(sessionId: string) {
  selectedSessionId.value = sessionId
  try {
    selectedSession.value = await getSession(sessionId)
  }
  catch (e) {
    console.error('加载会话详情失败:', e)
  }
}

async function handleDeleteSession(sessionId: string) {
  await deleteSession(sessionId)
  if (selectedSessionId.value === sessionId) {
    selectedSessionId.value = null
    selectedSession.value = null
  }
}

async function handleBatchDelete(sessionIds: string[]) {
  await batchDeleteSessions(sessionIds)
  if (selectedSessionId.value && sessionIds.includes(selectedSessionId.value)) {
    selectedSessionId.value = null
    selectedSession.value = null
  }
}

function handleClearAll() {
  dialog.warning({
    title: '确认清空所有历史',
    content: '确定要清空所有会话记录吗？此操作无法撤销。',
    positiveText: '清空',
    negativeText: '取消',
    onPositiveClick: async () => {
      await clearAllSessions()
      selectedSessionId.value = null
      selectedSession.value = null
    },
  })
}

function toggleSelectionMode() {
  selectionMode.value = !selectionMode.value
  if (!selectionMode.value) {
    selectedSessionId.value = null
    selectedSession.value = null
  }
}
</script>

<template>
  <NModal
    v-model:show="show"
    preset="card"
    title="会话历史"
    style="width: 800px; max-width: 90vw; height: 600px; max-height: 85vh;"
    :bordered="false"
    size="small"
    :segmented="{ content: true }"
  >
    <template #header-extra>
      <div class="flex items-center gap-1">
        <NButton
          text
          size="small"
          :type="selectionMode ? 'primary' : 'default'"
          title="多选模式"
          @click="toggleSelectionMode"
        >
          <template #icon>
            <div class="i-carbon-checkbox-checked w-4 h-4" />
          </template>
        </NButton>
        <NPopconfirm @positive-click="handleClearAll">
          <template #trigger>
            <NButton
              text
              size="small"
              :disabled="sessions.length === 0"
              title="清空所有历史"
            >
              <template #icon>
                <div class="i-carbon-trash-can w-4 h-4" />
              </template>
            </NButton>
          </template>
          确定要清空所有历史记录吗？
        </NPopconfirm>
      </div>
    </template>

    <div class="flex flex-col h-full" style="height: 500px;">
      <!-- 搜索栏 -->
      <div class="pb-3 border-b border-surface-border mb-3">
        <SearchBar @search="handleSearch" />
      </div>

      <!-- 内容区 -->
      <div class="flex flex-1 overflow-hidden gap-3">
        <!-- 会话列表 -->
        <div
          class="overflow-hidden flex-shrink-0 transition-all"
          :style="{ width: selectedSession && !selectionMode ? '40%' : '100%' }"
        >
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

        <!-- 会话详情 -->
        <div
          v-if="selectedSession && !selectionMode"
          class="flex-1 flex flex-col overflow-hidden border-l border-surface-border pl-3"
        >
          <div class="flex items-center justify-between mb-2">
            <span class="text-sm font-semibold opacity-70">会话详情</span>
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
            :app-config="props.appConfig"
          />
        </div>
      </div>
    </div>
  </NModal>
</template>
