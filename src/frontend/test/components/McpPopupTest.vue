<script setup lang="ts">
import { onMounted, ref } from 'vue'
import McpPopup from '../../components/popup/McpPopup.vue'
import { useSessionHistory } from '../../composables/useSessionHistory'

// 初始化会话历史管理器
const sessionHistory = useSessionHistory()

// 组件挂载时初始化测试环境
onMounted(async () => {
  // 设置为测试模式
  sessionHistory.setMockMode(true)
  // 初始化会话历史
  await sessionHistory.initialize()
  console.log('[McpPopupTest] 测试环境已初始化，mockMode已启用')
})

// Props
const props = defineProps<{
  showControls?: boolean
}>()

// 默认显示控制面板
const showControls = ref(props.showControls !== false)

const currentTheme = ref('dark')
const showPopup = ref(true)

// 模拟不同类型的 MCP 请求
const requestTemplates = [
  {
    name: '基础文本请求',
    request: {
      id: 'test-basic',
      message: '这是一个基础的模拟请求，用于测试弹窗功能。请确认是否继续执行操作。',
      is_markdown: false,
    },
  },
  {
    name: '预定义选项请求',
    request: {
      id: 'test-options',
      message: '请选择您需要的操作类型：',
      predefined_options: ['创建新文件', '修改现有文件', '删除文件', '查看文件内容'],
      is_markdown: false,
    },
  },
  {
    name: 'Markdown + 代码块',
    request: {
      id: 'test-markdown-code',
      message: `# 代码审查请求

我需要对以下代码进行审查和优化：

## 当前代码

\`\`\`typescript
interface User {
  id: string
  name: string
  email: string
}

function createUser(data: Partial<User>): User {
  return {
    id: Math.random().toString(36),
    name: data.name || 'Unknown',
    email: data.email || 'unknown@example.com'
  }
}
\`\`\`

## 发现的问题

1. **ID生成不安全** - 使用 \`Math.random()\` 可能产生重复ID
2. **类型安全性** - 缺少必要的验证
3. **错误处理** - 没有处理无效输入

## 建议的改进

\`\`\`typescript

interface User {
  id: string
  name: string
  email: string
}

interface CreateUserData {
  name: string
  email: string
}

function createUser(data: CreateUserData): User {
  if (!data.name || !data.email) {
    throw new Error('Name and email are required')
  }

  if (!isValidEmail(data.email)) {
    throw new Error('Invalid email format')
  }

  return {
    id: uuidv4(),
    name: data.name.trim(),
    email: data.email.toLowerCase().trim()
  }
}

function isValidEmail(email: string): boolean {
  const emailRegex = /^[^\\s@]+@[^\\s@]+\\.[^\\s@]+$/
  return emailRegex.test(email)
}
\`\`\`

请选择您希望的操作：`,
      predefined_options: ['应用建议的改进', '需要进一步讨论', '查看更多示例', '拒绝修改'],
      is_markdown: true,
    },
  },
  {
    name: '自定义请求',
    request: {
      id: 'test-custom',
      message: `# 🎨 新弹窗系统测试

欢迎使用重构后的弹窗系统！

## ✨ 新特性
- 🧩 **模块化组件**：头部、内容、输入、操作栏独立组件
- 🎭 **过渡动画**：流畅的切换效果和骨架屏
- 🏠 **主界面切换**：点击头部按钮可切换到主界面
- 🎯 **状态管理**：完整的应用状态管理系统
- 🧪 **模拟数据**：支持完全脱离MCP服务运行

## 🔧 测试功能
请尝试以下操作：
1. 切换主题
2. 选择预定义选项
3. 输入文本内容
4. 拖拽或粘贴图片
5. 点击主界面按钮

\`\`\`typescript
// 新的弹窗系统架构
interface PopupSystem {
  manager: PopupManager
  components: ModularComponents
  transitions: SmoothAnimations
  state: ReactiveState
}
\`\`\`

请选择您要测试的功能：`,
      predefined_options: [
        '🎨 测试主题切换',
        '🏠 切换到主界面',
        '📝 测试文本输入',
        '🖼️ 测试图片上传',
        '⚡ 测试快捷键',
        '🔄 测试状态管理',
      ],
      is_markdown: true,
    },
  },
]

const currentTemplate = ref(2) // 默认显示markdown模板
const currentRequest = ref(requestTemplates[2].request)

function switchTemplate(index: number) {
  currentTemplate.value = index
  currentRequest.value = requestTemplates[index].request
}

function handleResponse(response: any) {
  console.log('MCP 响应:', response)
}

function handleCancel() {
  console.log('MCP 取消')
}

function handleThemeChange(theme: string) {
  currentTheme.value = theme
  console.log('主题切换:', theme)
}

function handleOpenMainLayout() {
  console.log('打开主界面')
}

function togglePopup() {
  showPopup.value = !showPopup.value
}
</script>

<template>
  <div class="mcp-popup-test">
    <!-- 控制面板模式 -->
    <div v-if="showControls">
      <n-card title="MCP 弹窗测试 - 新弹窗系统">
        <template #header-extra>
          <n-space>
            <n-tag size="small" type="info">
              测试模式
            </n-tag>
            <n-button size="small" @click="togglePopup">
              {{ showPopup ? '隐藏弹窗' : '显示弹窗' }}
            </n-button>
          </n-space>
        </template>

        <!-- 控制面板 -->
        <div class="control-panel">
          <n-card title="测试控制" size="small">
            <n-space vertical>
              <div class="control-section">
                <h4>请求模板:</h4>
                <n-space>
                  <n-button
                    v-for="(template, index) in requestTemplates" :key="index"
                    :type="currentTemplate === index ? 'primary' : 'default'" size="small"
                    @click="switchTemplate(index)"
                  >
                    {{ template.name }}
                  </n-button>
                </n-space>
              </div>

              <div class="control-section">
                <h4>当前状态:</h4>
                <n-space vertical size="small">
                  <n-space align="center" justify="space-between">
                    <span>主题:</span>
                    <n-tag size="small" :type="currentTheme === 'dark' ? 'warning' : 'info'">
                      {{ currentTheme }}
                    </n-tag>
                  </n-space>

                  <n-space align="center" justify="space-between">
                    <span>弹窗:</span>
                    <n-tag size="small" :type="showPopup ? 'success' : 'default'">
                      {{ showPopup ? '显示' : '隐藏' }}
                    </n-tag>
                  </n-space>

                  <n-space align="center" justify="space-between">
                    <span>选项数量:</span>
                    <n-tag size="small" type="info">
                      {{ currentRequest.predefined_options?.length || 0 }}
                    </n-tag>
                  </n-space>
                </n-space>
              </div>
            </n-space>
          </n-card>
        </div>

        <!-- 弹窗组件显示区域 -->
        <div class="popup-container">
          <!-- 弹窗组件 -->
          <div v-if="showPopup" class="popup-mode">
            <div class="popup-overlay">
              <McpPopup
                :request="currentRequest" :current-theme="currentTheme" :mock-mode="true"
                @response="handleResponse" @cancel="handleCancel" @theme-change="handleThemeChange"
                @open-main-layout="handleOpenMainLayout"
              />
            </div>
          </div>

          <!-- 隐藏状态提示 -->
          <div v-else class="hidden-state">
            <div class="hidden-message">
              <h3>弹窗已隐藏</h3>
              <p>点击"显示弹窗"按钮来查看弹窗组件</p>
            </div>
          </div>
        </div>

        <!-- 说明信息 -->
        <div class="info-panel">
          <n-card title="测试说明" size="small">
            <n-space vertical size="small">
              <div class="flex items-center text-sm">
                <div class="w-1.5 h-1.5 bg-green-500 rounded-full mr-3 flex-shrink-0" />
                全新的模块化弹窗系统，支持完整的状态管理和过渡动画
              </div>
              <div class="flex items-center text-sm">
                <div class="w-1.5 h-1.5 bg-green-500 rounded-full mr-3 flex-shrink-0" />
                模块化组件：头部、内容、输入、操作栏独立组件
              </div>
              <div class="flex items-center text-sm">
                <div class="w-1.5 h-1.5 bg-green-500 rounded-full mr-3 flex-shrink-0" />
                支持模拟数据，无需依赖MCP服务
              </div>
              <div class="flex items-center text-sm">
                <div class="w-1.5 h-1.5 bg-green-500 rounded-full mr-3 flex-shrink-0" />
                符合代码规范，使用UnoCSS和Naive UI组件
              </div>
              <div class="flex items-center text-sm">
                <div class="w-1.5 h-1.5 bg-blue-500 rounded-full mr-3 flex-shrink-0" />
                <span class="opacity-70">src/frontend/components/popup/</span>
              </div>
            </n-space>
          </n-card>
        </div>
      </n-card>
    </div>

    <!-- 纯净模式 - 只显示弹窗 -->
    <div v-else class="pure-mode">
      <McpPopup
        :request="currentRequest" :current-theme="currentTheme" :mock-mode="true" @response="handleResponse"
        @cancel="handleCancel" @theme-change="handleThemeChange" @open-main-layout="handleOpenMainLayout"
      />
    </div>
  </div>
</template>

<style scoped>
.mcp-popup-test {
  max-width: 1200px;
  margin: 0 auto;
}

.control-panel {
  margin-bottom: 20px;
}

.control-section {
  margin-bottom: 15px;
}

.control-section h4 {
  margin: 0 0 8px 0;
  color: var(--text-color);
  font-size: 0.9rem;
  font-weight: 500;
}

.popup-container {
  margin: 20px 0;
  border: 2px dashed var(--border-color);
  border-radius: 8px;
  padding: 0;
  background: var(--card-color);
  position: relative;
  min-height: 400px;
  overflow: hidden;
}

.popup-container::before {
  content: '新弹窗系统预览 - 支持模块化组件和状态管理';
  position: absolute;
  top: -10px;
  left: 20px;
  background: var(--card-color);
  padding: 0 10px;
  font-size: 0.8rem;
  color: var(--text-color);
  opacity: 0.6;
  z-index: 10;
}

.popup-overlay {
  position: relative;
  width: 100%;
  height: 100%;
  min-height: 400px;
  background: rgba(0, 0, 0, 0.1);
  display: flex;
  align-items: center;
  justify-content: center;
}

.popup-overlay :deep(.popup-container) {
  position: relative !important;
  width: 100% !important;
  height: 100% !important;
  max-width: none !important;
  max-height: none !important;
  border-radius: 0 !important;
}

.info-panel {
  margin-top: 20px;
}

/* 纯净模式 */
.pure-mode {
  width: 100%;
  height: 100%;
}

.pure-mode :deep(.popup-container) {
  position: relative !important;
  inset: 0 !important;
  width: 100% !important;
  height: 100% !important;
}

/* 增强模式样式 */
.enhanced-mode {
  @apply w-full h-full min-h-[500px];
}

/* 基础模式样式 */
.basic-mode {
  @apply w-full h-full min-h-[500px];
}

/* 隐藏状态样式 */
.hidden-state {
  @apply flex items-center justify-center w-full h-full min-h-[300px];
  @apply bg-gray-50 dark:bg-gray-800 rounded-lg;
}

.hidden-message {
  @apply text-center space-y-2;
}

.hidden-message h3 {
  @apply text-lg font-medium text-gray-700 dark:text-gray-300;
}

.hidden-message p {
  @apply text-sm text-gray-500 dark:text-gray-400;
}
</style>
