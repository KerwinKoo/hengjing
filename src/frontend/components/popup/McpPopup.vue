<script setup lang="ts">
import type { McpRequest } from '../../types/popup'
import type { SessionData, SessionUpdateData } from '../../types/session'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { useMessage } from 'naive-ui'
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { useSessionHistory } from '../../composables/useSessionHistory'

import PopupActions from './PopupActions.vue'
import PopupContent from './PopupContent.vue'
import PopupInput from './PopupInput.vue'
import { buildPopupSubmitResponse, canSendPopupResponse, hasPopupInputContent } from './popupSubmission'

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
  request: McpRequest | null
  appConfig: AppConfig
  mockMode?: boolean
  testMode?: boolean
}

interface Emits {
  response: [response: any]
  cancel: []
  themeChange: [theme: string]
  openMainLayout: []
  toggleAlwaysOnTop: []
  toggleAudioNotification: []
  updateAudioUrl: [url: string]
  testAudio: []
  stopAudio: []
  testAudioError: [error: any]
  updateWindowSize: [size: { width: number, height: number, fixed: boolean }]
}

const props = withDefaults(defineProps<Props>(), {
  mockMode: false,
  testMode: false,
})

const emit = defineEmits<Emits>()

// 使用消息提示
const message = useMessage()

// 使用会话历史管理
const sessionHistory = useSessionHistory()
const { saveSession, updateSession } = sessionHistory

// 响应式状态
const loading = ref(false)
const submitting = ref(false)
const selectedOptions = ref<string[]>([])
const userInput = ref('')
const draggedImages = ref<string[]>([])
const inputRef = ref()
const currentSessionId = ref<string | null>(null)

// 继续回复配置
const continueReplyEnabled = ref(true)
const continuePrompt = ref('请按照最佳实践继续')

// 计算属性
const isVisible = computed(() => !!props.request)
const hasOptions = computed(() => (props.request?.predefined_options?.length ?? 0) > 0)
const hasInputContent = computed(() => hasPopupInputContent({
  userInput: userInput.value,
  selectedOptions: selectedOptions.value,
  draggedImages: draggedImages.value,
}))
const canSubmit = computed(() => canSendPopupResponse(props.request))
const canEnhance = computed(() => hasInputContent.value)

// 获取输入组件的状态文本
const inputStatusText = computed(() => {
  if (!hasInputContent.value && canSubmit.value) {
    return hasOptions.value ? '可直接发送，或选择选项 / 输入补充说明' : '可直接发送，或补充说明'
  }
  return inputRef.value?.statusText || '等待输入...'
})

// 加载继续回复配置
async function loadReplyConfig() {
  try {
    const config = await invoke('get_reply_config')
    if (config) {
      const replyConfig = config as any
      continueReplyEnabled.value = replyConfig.enable_continue_reply ?? true
      continuePrompt.value = replyConfig.continue_prompt ?? '请按照最佳实践继续'
    }
  }
  catch (error) {
    console.log('加载继续回复配置失败，使用默认值:', error)
  }
}

// 监听配置变化（当从设置页面切换回来时）
watch(() => props.appConfig.reply, (newReplyConfig) => {
  if (newReplyConfig) {
    continueReplyEnabled.value = newReplyConfig.enabled
    continuePrompt.value = newReplyConfig.prompt
  }
}, { deep: true, immediate: true })

// Telegram事件监听器
let telegramUnlisten: (() => void) | null = null

// 监听请求变化
watch(() => props.request, async (newRequest) => {
  if (newRequest) {
    resetForm()
    loading.value = true
    // 每次显示弹窗时重新加载配置
    loadReplyConfig()

    // 创建会话记录（只保存 AI 消息，用户响应稍后更新）
    try {
      console.log('[McpPopup] 开始创建会话记录...')
      const sessionData: SessionData = {
        source: 'send',
        userInput: null,
        aiResponse: newRequest.message || '',
        selectedOptions: [],
        images: [],
      }
      const record = await saveSession(sessionData)
      currentSessionId.value = record.id
      console.log('[McpPopup] 会话已创建:', record.id)
    }
    catch (error) {
      console.error('[McpPopup] 创建会话失败:', error)
      currentSessionId.value = null
    }

    setTimeout(() => {
      loading.value = false
    }, 300)
  }
}, { immediate: true })

// 设置Telegram事件监听
async function setupTelegramListener() {
  try {
    telegramUnlisten = await listen('telegram-event', (event) => {
      console.log('🎯 [McpPopup] 收到Telegram事件:', event)
      console.log('🎯 [McpPopup] 事件payload:', event.payload)
      handleTelegramEvent(event.payload as any)
    })
    console.log('🎯 [McpPopup] Telegram事件监听器已设置')
  }
  catch (error) {
    console.error('🎯 [McpPopup] 设置Telegram事件监听器失败:', error)
  }
}

// 处理Telegram事件
function handleTelegramEvent(event: any) {
  console.log('🎯 [McpPopup] 开始处理事件:', event.type)

  switch (event.type) {
    case 'option_toggled':
      console.log('🎯 [McpPopup] 处理选项切换:', event.option)
      handleOptionToggle(event.option)
      break
    case 'text_updated':
      console.log('🎯 [McpPopup] 处理文本更新:', event.text)
      handleTextUpdate(event.text)
      break
    case 'continue_pressed':
      console.log('🎯 [McpPopup] 处理继续按钮')
      handleContinue()
      break
    case 'send_pressed':
      console.log('🎯 [McpPopup] 处理发送按钮')
      handleSubmit()
      break
    default:
      console.log('🎯 [McpPopup] 未知事件类型:', event.type)
  }
}

// 处理选项切换
function handleOptionToggle(option: string) {
  const index = selectedOptions.value.indexOf(option)
  if (index > -1) {
    // 取消选择
    selectedOptions.value.splice(index, 1)
  }
  else {
    // 添加选择
    selectedOptions.value.push(option)
  }

  // 同步到PopupInput组件
  if (inputRef.value) {
    inputRef.value.updateData({ selectedOptions: selectedOptions.value })
  }
}

// 处理文本更新
function handleTextUpdate(text: string) {
  userInput.value = text

  // 同步到PopupInput组件
  if (inputRef.value) {
    inputRef.value.updateData({ userInput: text })
  }
}

// 组件挂载时设置监听器和加载配置
onMounted(async () => {
  // 初始化会话历史
  await sessionHistory.initialize()
  loadReplyConfig()
  setupTelegramListener()
})

// 组件卸载时清理监听器
onUnmounted(() => {
  if (telegramUnlisten) {
    telegramUnlisten()
  }
})

// 更新会话记录（添加用户响应）
async function updateSessionRecord(updateData: SessionUpdateData) {
  if (!currentSessionId.value) {
    console.warn('[SessionHistory] 没有当前会话ID，跳过更新')
    return
  }

  try {
    console.log('[SessionHistory] 更新会话:', currentSessionId.value, updateData)
    await updateSession(currentSessionId.value, updateData)
    console.log('[SessionHistory] 会话更新成功')
  }
  catch (error) {
    // 错误隔离：历史记录错误不影响弹窗正常使用
    console.error('[SessionHistory] 更新会话失败（不影响弹窗功能）:', error)
  }
}

// 重置表单
function resetForm() {
  selectedOptions.value = []
  userInput.value = ''
  draggedImages.value = []
  submitting.value = false
  currentSessionId.value = null
}

// 处理提交
async function handleSubmit() {
  if (!props.request || submitting.value)
    return

  submitting.value = true

  try {
    // 从 inputRef 获取最新的输入值（避免防抖延迟）
    let finalUserInput = userInput.value
    if (inputRef.value) {
      const latestInput = inputRef.value.getCurrentInputValue?.() || userInput.value
      finalUserInput = latestInput
    }

    console.log('[DEBUG] handleSubmit 开始:', {
      userInput: finalUserInput,
      selectedOptions: selectedOptions.value,
      draggedImages: draggedImages.value.length,
    })

    const response = buildPopupSubmitResponse({
      userInput: finalUserInput,
      selectedOptions: selectedOptions.value,
      draggedImages: draggedImages.value,
    }, props.request.id)

    console.log('[DEBUG] handleSubmit 发送响应:', response)

    if (props.mockMode) {
      // 模拟模式下的延迟
      await new Promise(resolve => setTimeout(resolve, 1000))
      message.success('模拟响应发送成功')
    }

    // 更新会话记录（添加用户响应）
    await updateSessionRecord({
      source: 'send',
      userInput: response.user_input,
      selectedOptions: response.selected_options,
      images: response.images.map(img => ({
        data: img.data,
        mediaType: img.media_type,
        filename: img.filename,
      })),
    })

    // 通过事件触发统一的响应处理（send_mcp_response + exit_app）
    // 注意：不再在这里直接调用 invoke，避免重复发送
    emit('response', response)
  }
  catch (error) {
    console.error('提交响应失败:', error)
    message.error('提交失败，请重试')
  }
  finally {
    submitting.value = false
  }
}

// 处理输入更新
function handleInputUpdate(data: { userInput: string, selectedOptions: string[], draggedImages: string[] }) {
  console.log('[DEBUG] handleInputUpdate 收到:', data)
  // 更新 userInput，用于 canSubmit 判断
  userInput.value = data.userInput
  selectedOptions.value = data.selectedOptions
  draggedImages.value = data.draggedImages
}

// 处理图片添加 - 移除重复逻辑，避免双重添加
function handleImageAdd(_image: string) {
  // 这个函数现在只是为了保持接口兼容性，实际添加在PopupInput中完成
}

// 处理图片移除
function handleImageRemove(index: number) {
  draggedImages.value.splice(index, 1)
}

// 处理继续按钮点击
async function handleContinue() {
  if (submitting.value)
    return

  submitting.value = true

  try {
    // 使用新的结构化数据格式
    const response = {
      user_input: continuePrompt.value,
      selected_options: [],
      images: [],
      metadata: {
        timestamp: new Date().toISOString(),
        request_id: props.request?.id || null,
        source: 'popup_continue',
      },
    }

    if (props.mockMode) {
      // 模拟模式下的延迟
      await new Promise(resolve => setTimeout(resolve, 1000))
      message.success('继续请求发送成功')
    }

    // 更新会话记录（添加用户响应）
    await updateSessionRecord({
      source: 'continue',
      userInput: continuePrompt.value,
      selectedOptions: [],
      images: [],
    })

    // 通过事件触发统一的响应处理（send_mcp_response + exit_app）
    // 注意：不再在这里直接调用 invoke，避免重复发送
    emit('response', response)
  }
  catch (error) {
    console.error('发送继续请求失败:', error)
    message.error('继续请求失败，请重试')
  }
  finally {
    submitting.value = false
  }
}

// 处理引用消息
function handleQuoteMessage(messageContent: string) {
  if (inputRef.value) {
    inputRef.value.handleQuoteMessage(messageContent)
  }
}

// 处理增强按钮点击
async function handleEnhance() {
  if (submitting.value)
    return

  submitting.value = true

  try {
    // 构建增强prompt
    const enhancePrompt = `Use the following prompt to optimize and enhance the context of the content in 《》, and return the enhanced result by calling the tool '恒境' after completion.Here is an instruction that I'd like to give you, but it needs to be improved. Rewrite and enhance this instruction to make it clearer, more specific, less ambiguous, and correct any mistakes. Reply immediately with your answer, even if you're not sure. Consider the context of our conversation history when enhancing the prompt. Reply with the following format:

### BEGIN RESPONSE ###
Here is an enhanced version of the original instruction that is more specific and clear:
<augment-enhanced-prompt>enhanced prompt goes here</augment-enhanced-prompt>

### END RESPONSE ###

Here is my original instruction:

《${userInput.value.trim()}》`

    // 使用新的结构化数据格式
    const response = {
      user_input: enhancePrompt,
      selected_options: [],
      images: [],
      metadata: {
        timestamp: new Date().toISOString(),
        request_id: props.request?.id || null,
        source: 'popup_enhance',
      },
    }

    if (props.mockMode) {
      // 模拟模式下的延迟
      await new Promise(resolve => setTimeout(resolve, 1000))
      message.success('增强请求发送成功')
    }

    // 更新会话记录（添加用户响应）
    await updateSessionRecord({
      source: 'enhance',
      userInput: enhancePrompt,
      selectedOptions: [],
      images: [],
    })

    // 通过事件触发统一的响应处理（send_mcp_response + exit_app）
    // 注意：不再在这里直接调用 invoke，避免重复发送
    emit('response', response)
  }
  catch (error) {
    console.error('发送增强请求失败:', error)
    message.error('增强请求失败，请重试')
  }
  finally {
    submitting.value = false
  }
}
</script>

<template>
  <div v-if="isVisible" class="flex flex-col flex-1 min-h-0 overflow-hidden">
    <!-- 内容区域 - 可滚动 -->
    <div class="flex-1 overflow-y-auto scrollbar-thin min-h-0">
      <!-- 消息内容 - 允许选中 -->
      <div class="mx-2 mt-2 mb-1 px-4 py-3 bg-black-100 rounded-lg select-text" data-guide="popup-content">
        <PopupContent :request="request" :loading="loading" :current-theme="props.appConfig.theme" @quote-message="handleQuoteMessage" />
      </div>

      <!-- 输入和选项 - 允许选中 -->
      <div class="px-4 pb-3 bg-black select-text">
        <PopupInput
          ref="inputRef" :request="request" :loading="loading" :submitting="submitting"
          @update="handleInputUpdate" @image-add="handleImageAdd" @image-remove="handleImageRemove"
        />
      </div>
    </div>

    <!-- 底部操作栏 - 固定在底部 -->
    <div class="flex-shrink-0 bg-black-100 border-t-2 border-black-200" data-guide="popup-actions">
      <PopupActions
        :request="request" :loading="loading" :submitting="submitting" :can-submit="canSubmit"
        :can-enhance="canEnhance" :has-input-content="hasInputContent"
        :continue-reply-enabled="continueReplyEnabled" :input-status-text="inputStatusText"
        @submit="handleSubmit" @continue="handleContinue" @enhance="handleEnhance"
      />
    </div>
  </div>
</template>
