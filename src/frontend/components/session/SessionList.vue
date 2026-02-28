<script setup lang="ts">
import type { SessionRecord } from '../../types/session'
import { NButton, NCheckbox, NEmpty, NPopconfirm, NVirtualList, useDialog } from 'naive-ui'
import { computed, ref } from 'vue'

interface Props {
  sessions: SessionRecord[]
  searchQuery: string
  selectedSessionId: string | null
  selectionMode: boolean
}

interface Emits {
  selectSession: [sessionId: string]
  deleteSession: [sessionId: string]
  toggleSelection: [sessionId: string]
  batchDelete: [sessionIds: string[]]
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

const dialog = useDialog()

// 选中的会话ID集合
const selectedSessionIds = ref<Set<string>>(new Set())

// 虚拟滚动配置
const ITEM_HEIGHT = 80

// 格式化时间戳
function formatTimestamp(timestamp: string): string {
  const date = new Date(timestamp)
  const now = new Date()
  const diff = now.getTime() - date.getTime()

  // 小于1分钟
  if (diff < 60 * 1000) {
    return '刚刚'
  }

  // 小于1小时
  if (diff < 60 * 60 * 1000) {
    const minutes = Math.floor(diff / (60 * 1000))
    return `${minutes}分钟前`
  }

  // 小于1天
  if (diff < 24 * 60 * 60 * 1000) {
    const hours = Math.floor(diff / (60 * 60 * 1000))
    return `${hours}小时前`
  }

  // 小于7天
  if (diff < 7 * 24 * 60 * 60 * 1000) {
    const days = Math.floor(diff / (24 * 60 * 60 * 1000))
    return `${days}天前`
  }

  // 显示完整日期
  return date.toLocaleDateString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  })
}

// 生成会话摘要（前50个字符）
function generateSummary(session: SessionRecord): string {
  // 优先使用用户输入
  const content = session.userInput || session.aiResponse

  // 移除Markdown格式和多余空白
  const plainText = content
    .replace(/[#*`_~[\]()]/g, '')
    .replace(/\s+/g, ' ')
    .trim()

  // 截取前50个字符
  if (plainText.length <= 50) {
    return plainText
  }

  return `${plainText.substring(0, 50)}...`
}

// 处理会话点击
function handleSessionClick(session: SessionRecord) {
  if (props.selectionMode) {
    // 多选模式：切换选中状态
    toggleSessionSelection(session.id)
  }
  else {
    // 普通模式：选择会话
    emit('selectSession', session.id)
  }
}

// 切换会话选中状态
function toggleSessionSelection(sessionId: string) {
  if (selectedSessionIds.value.has(sessionId)) {
    selectedSessionIds.value.delete(sessionId)
  }
  else {
    selectedSessionIds.value.add(sessionId)
  }
  emit('toggleSelection', sessionId)
}

// 全选/取消全选
function toggleSelectAll() {
  if (isAllSelected.value) {
    // 取消全选
    selectedSessionIds.value.clear()
  }
  else {
    // 全选
    props.sessions.forEach((session) => {
      selectedSessionIds.value.add(session.id)
    })
  }
}

// 是否全选
const isAllSelected = computed(() => {
  return props.sessions.length > 0
    && selectedSessionIds.value.size === props.sessions.length
})

// 是否有选中项
const hasSelection = computed(() => {
  return selectedSessionIds.value.size > 0
})

// 批量删除
function handleBatchDelete() {
  const count = selectedSessionIds.value.size

  dialog.warning({
    title: '确认批量删除',
    content: `确定要删除选中的 ${count} 个会话吗？此操作无法撤销。`,
    positiveText: '删除',
    negativeText: '取消',
    onPositiveClick: () => {
      const ids = Array.from(selectedSessionIds.value)
      emit('batchDelete', ids)
      selectedSessionIds.value.clear()
    },
  })
}

// 处理单个删除
function handleDeleteSession(sessionId: string, event: Event) {
  event.stopPropagation()
  emit('deleteSession', sessionId)
}

// 获取来源类型的显示文本
function getSourceLabel(source: string): string {
  const labels: Record<string, string> = {
    send: '发送',
    continue: '继续',
    enhance: '增强',
  }
  return labels[source] || source
}
</script>

<template>
  <div class="session-list">
    <!-- 多选模式工具栏 -->
    <div v-if="selectionMode" class="selection-toolbar">
      <div class="selection-info">
        <NCheckbox
          :checked="isAllSelected"
          :indeterminate="hasSelection && !isAllSelected"
          @update:checked="toggleSelectAll"
        >
          全选
        </NCheckbox>
        <span v-if="hasSelection" class="selection-count">
          已选中 {{ selectedSessionIds.size }} 项
        </span>
      </div>

      <NButton
        v-if="hasSelection"
        type="error"
        size="small"
        @click="handleBatchDelete"
      >
        <template #icon>
          <div class="i-carbon-trash-can w-4 h-4" />
        </template>
        批量删除
      </NButton>
    </div>

    <!-- 会话列表 -->
    <div v-if="sessions.length > 0" class="list-container">
      <NVirtualList
        :items="sessions"
        :item-size="ITEM_HEIGHT"
        class="virtual-list"
      >
        <template #default="{ item: session }">
          <div
            class="session-item"
            :class="{
              'selected': selectedSessionId === session.id,
              'selection-mode': selectionMode,
            }"
            @click="handleSessionClick(session)"
          >
            <!-- 多选复选框 -->
            <div v-if="selectionMode" class="checkbox-wrapper" @click.stop>
              <NCheckbox
                :checked="selectedSessionIds.has(session.id)"
                @update:checked="() => toggleSessionSelection(session.id)"
              />
            </div>

            <!-- 会话内容 -->
            <div class="session-content">
              <div class="session-header">
                <span class="session-time">{{ formatTimestamp(session.timestamp) }}</span>
                <span class="session-source">{{ getSourceLabel(session.source) }}</span>
              </div>

              <div class="session-summary">
                {{ generateSummary(session) }}
              </div>

              <div v-if="session.images.length > 0" class="session-meta">
                <div class="i-carbon-image w-4 h-4" />
                <span class="image-count">{{ session.images.length }} 张图片</span>
              </div>
            </div>

            <!-- 删除按钮（非多选模式） -->
            <div v-if="!selectionMode" class="session-actions">
              <NPopconfirm
                @positive-click="(e) => handleDeleteSession(session.id, e)"
              >
                <template #trigger>
                  <NButton
                    text
                    size="small"
                    class="delete-btn"
                    @click.stop
                  >
                    <template #icon>
                      <div class="i-carbon-trash-can w-4 h-4" />
                    </template>
                  </NButton>
                </template>
                确定要删除这个会话吗？
              </NPopconfirm>
            </div>
          </div>
        </template>
      </NVirtualList>
    </div>

    <!-- 空状态 -->
    <div v-else class="empty-state">
      <NEmpty
        :description="searchQuery ? '未找到匹配的会话' : '暂无会话记录'"
        size="large"
      >
        <template #icon>
          <div
            :class="searchQuery ? 'i-carbon-search-locate' : 'i-carbon-chat'"
            class="w-16 h-16 text-on-surface-tertiary"
          />
        </template>
        <template v-if="!searchQuery" #extra>
          <div class="empty-hint">
            完成会话后，历史记录将自动保存在这里
          </div>
        </template>
      </NEmpty>
    </div>
  </div>
</template>

<style scoped>
.session-list {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

.selection-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border-bottom: 1px solid rgb(var(--surface-border));
  background: rgb(var(--surface-container));
}

.selection-info {
  display: flex;
  align-items: center;
  gap: 12px;
}

.selection-count {
  font-size: 14px;
  color: rgb(var(--on-surface-secondary));
}

.list-container {
  flex: 1;
  overflow: hidden;
}

.virtual-list {
  height: 100%;
}

.session-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  min-height: 80px;
  border-bottom: 1px solid rgb(var(--surface-border));
  cursor: pointer;
  transition: background-color 0.2s;
}

.session-item:hover {
  background: rgb(var(--surface-container-hover));
}

.session-item.selected {
  background: rgb(var(--primary-container));
}

.session-item.selection-mode {
  padding-left: 8px;
}

.checkbox-wrapper {
  flex-shrink: 0;
}

.session-content {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.session-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.session-time {
  font-size: 12px;
  color: rgb(var(--on-surface-secondary));
}

.session-source {
  font-size: 11px;
  padding: 2px 6px;
  border-radius: 4px;
  background: rgb(var(--surface-container));
  color: rgb(var(--on-surface-secondary));
}

.session-summary {
  font-size: 14px;
  color: rgb(var(--on-surface));
  line-height: 1.4;
  overflow: hidden;
  text-overflow: ellipsis;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
}

.session-meta {
  display: flex;
  align-items: center;
  gap: 4px;
  color: rgb(var(--on-surface-tertiary));
  font-size: 12px;
}

.image-count {
  font-size: 12px;
}

.session-actions {
  flex-shrink: 0;
  opacity: 0;
  transition: opacity 0.2s;
}

.session-item:hover .session-actions {
  opacity: 1;
}

.delete-btn {
  color: rgb(var(--error));
}

.delete-btn:hover {
  color: rgb(var(--error-hover));
}

.empty-state {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 32px 16px;
}

.empty-hint {
  margin-top: 8px;
  font-size: 13px;
  color: rgb(var(--on-surface-tertiary));
  text-align: center;
}
</style>
