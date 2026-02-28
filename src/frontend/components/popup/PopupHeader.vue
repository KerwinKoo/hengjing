<script setup lang="ts">
import ThemeIcon from '../common/ThemeIcon.vue'

interface Props {
  currentTheme?: string
  loading?: boolean
  showMainLayout?: boolean
  alwaysOnTop?: boolean
}

interface Emits {
  themeChange: [theme: string]
  openMainLayout: []
  toggleAlwaysOnTop: []
  openHistory: []
}

const props = withDefaults(defineProps<Props>(), {
  currentTheme: 'dark',
  loading: false,
  showMainLayout: false,
  alwaysOnTop: false,
})

const emit = defineEmits<Emits>()

function handleThemeChange() {
  // 切换到下一个主题
  const nextTheme = props.currentTheme === 'light' ? 'dark' : 'light'
  emit('themeChange', nextTheme)
}

function handleOpenMainLayout() {
  emit('openMainLayout')
}

function handleToggleAlwaysOnTop() {
  emit('toggleAlwaysOnTop')
}

function handleOpenHistory() {
  emit('openHistory')
}
</script>

<template>
  <div class="px-4 py-3 select-none">
    <div class="flex items-center justify-between">
      <!-- 左侧：标题 -->
      <div class="flex items-center gap-3">
        <div class="w-3 h-3 rounded-full bg-primary-500" />
        <h1 class="text-base font-medium text-white">
          恒境 - AI 交互确认助手，助力AI持续交互
        </h1>
      </div>

      <!-- 右侧：操作按钮 -->
      <n-space size="small">
        <!-- 置顶按钮 -->
        <n-button
          size="small"
          quaternary
          circle
          :title="props.alwaysOnTop ? '取消置顶' : '窗口置顶'"
          @click="handleToggleAlwaysOnTop"
        >
          <template #icon>
            <div
              :class="props.alwaysOnTop ? 'i-carbon-pin-filled' : 'i-carbon-pin'"
              class="w-4 h-4 text-white"
            />
          </template>
        </n-button>
        <!-- 历史记录按钮 -->
        <n-button
          size="small"
          quaternary
          circle
          title="会话历史"
          @click="handleOpenHistory"
        >
          <template #icon>
            <div class="i-carbon-recently-viewed w-4 h-4 text-white" />
          </template>
        </n-button>
        <n-button
          size="small"
          quaternary
          circle
          :title="props.showMainLayout ? '返回聊天' : '打开设置'"
          @click="handleOpenMainLayout"
        >
          <template #icon>
            <div
              :class="props.showMainLayout ? 'i-carbon-chat' : 'i-carbon-settings'"
              class="w-4 h-4 text-white"
            />
          </template>
        </n-button>
        <n-button
          size="small"
          quaternary
          circle
          :title="`切换到${props.currentTheme === 'light' ? '深色' : '浅色'}主题`"
          @click="handleThemeChange"
        >
          <template #icon>
            <ThemeIcon :theme="props.currentTheme" class="w-4 h-4 text-white" />
          </template>
        </n-button>
      </n-space>
    </div>
  </div>
</template>
