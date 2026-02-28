<script setup lang="ts">
import type { SessionRecord } from '../../types/session'
import { NImage, NImageGroup } from 'naive-ui'
import { computed, ref } from 'vue'
import PopupContent from '../popup/PopupContent.vue'

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
  session: SessionRecord | null
  appConfig: AppConfig
}

const props = defineProps<Props>()

// 将 SessionRecord 转换为 McpRequest 格式以复用 PopupContent
const mockRequest = computed(() => {
  if (!props.session)
    return null

  return {
    id: props.session.id,
    message: props.session.aiResponse,
    predefined_options: props.session.selectedOptions,
    is_markdown: true,
  }
})

// 格式化时间戳
function formatTimestamp(timestamp: string): string {
  const date = new Date(timestamp)
  const now = new Date()
  const diffMs = now.getTime() - date.getTime()
  const diffMins = Math.floor(diffMs / 60000)
  const diffHours = Math.floor(diffMs / 3600000)
  const diffDays = Math.floor(diffMs / 86400000)

  if (diffMins < 1)
    return '刚刚'
  if (diffMins < 60)
    return `${diffMins}分钟前`
  if (diffHours < 24)
    return `${diffHours}小时前`
  if (diffDays < 7)
    return `${diffDays}天前`

  return date.toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  })
}

// 获取来源类型的显示文本
function getSourceText(source: string): string {
  const sourceMap: Record<string, string> = {
    send: '发送',
    continue: '继续',
    enhance: '增强',
  }
  return sourceMap[source] || source
}

// 获取来源类型的图标
function getSourceIcon(source: string): string {
  const iconMap: Record<string, string> = {
    send: 'i-carbon-send-alt',
    continue: 'i-carbon-continue',
    enhance: 'i-carbon-magic-wand',
  }
  return iconMap[source] || 'i-carbon-document'
}

// 图片预览相关
const showImagePreview = ref(false)
const previewImageIndex = ref(0)

function openImagePreview(index: number) {
  previewImageIndex.value = index
  showImagePreview.value = true
}
</script>

<template>
  <div v-if="session" class="session-viewer h-full flex flex-col">
    <!-- 会话头部信息 -->
    <div class="session-header px-4 py-3 border-b border-gray-600/30">
      <div class="flex items-center justify-between mb-2">
        <div class="flex items-center gap-2">
          <div :class="getSourceIcon(session.source)" class="w-4 h-4 text-primary-500" />
          <span class="text-sm font-medium text-white">{{ getSourceText(session.source) }}</span>
        </div>
        <span class="text-xs text-white opacity-60">{{ formatTimestamp(session.timestamp) }}</span>
      </div>

      <!-- 用户输入 -->
      <div v-if="session.userInput" class="mt-3 p-3 bg-gray-800/50 rounded-lg">
        <div class="text-xs text-white opacity-60 mb-1 flex items-center gap-1">
          <div class="i-carbon-user w-3 h-3" />
          <span>用户输入</span>
        </div>
        <div class="text-sm text-white whitespace-pre-wrap">
          {{ session.userInput }}
        </div>
      </div>

      <!-- 选中的选项 -->
      <div v-if="session.selectedOptions.length > 0" class="mt-3">
        <div class="text-xs text-white opacity-60 mb-2 flex items-center gap-1">
          <div class="i-carbon-checkbox-checked w-3 h-3" />
          <span>选中的选项</span>
        </div>
        <div class="flex flex-wrap gap-2">
          <div
            v-for="(option, index) in session.selectedOptions"
            :key="`option-${index}`"
            class="px-2 py-1 text-xs bg-primary-500/20 text-primary-400 rounded border border-primary-500/50"
          >
            {{ option }}
          </div>
        </div>
      </div>

      <!-- 截图 -->
      <div v-if="session.images.length > 0" class="mt-3">
        <div class="text-xs text-white opacity-60 mb-2 flex items-center gap-1">
          <div class="i-carbon-image w-3 h-3" />
          <span>截图 ({{ session.images.length }})</span>
        </div>
        <NImageGroup>
          <div class="flex flex-wrap gap-2">
            <div
              v-for="(image, index) in session.images"
              :key="`image-${index}`"
              class="relative cursor-pointer"
            >
              <NImage
                :src="image.data"
                width="80"
                height="80"
                object-fit="cover"
                class="rounded border border-gray-600 hover:border-primary-500 transition-colors"
              />
              <div class="absolute bottom-1 left-1 w-4 h-4 bg-primary-500 text-white text-xs rounded-full flex items-center justify-center font-bold shadow-sm">
                {{ index + 1 }}
              </div>
            </div>
          </div>
        </NImageGroup>
      </div>
    </div>

    <!-- AI 回复内容 -->
    <div class="session-content flex-1 overflow-y-auto px-4 py-3">
      <div class="text-xs text-white opacity-60 mb-2 flex items-center gap-1">
        <div class="i-carbon-bot w-3 h-3" />
        <span>AI 回复</span>
      </div>
      <PopupContent
        :request="mockRequest"
        :loading="false"
        :current-theme="appConfig.theme"
      />
    </div>
  </div>

  <!-- 空状态 -->
  <div v-else class="h-full flex items-center justify-center">
    <div class="text-center text-white opacity-60">
      <div class="i-carbon-document-blank w-12 h-12 mx-auto mb-3 opacity-40" />
      <p class="text-sm">
        选择一个会话查看详情
      </p>
    </div>
  </div>
</template>

<style scoped>
.session-viewer {
  background-color: var(--color-surface-100, #1a1a1a);
}

.session-content {
  scrollbar-width: thin;
  scrollbar-color: rgba(255, 255, 255, 0.2) transparent;
}

.session-content::-webkit-scrollbar {
  width: 6px;
}

.session-content::-webkit-scrollbar-track {
  background: transparent;
}

.session-content::-webkit-scrollbar-thumb {
  background-color: rgba(255, 255, 255, 0.2);
  border-radius: 3px;
}

.session-content::-webkit-scrollbar-thumb:hover {
  background-color: rgba(255, 255, 255, 0.3);
}
</style>
