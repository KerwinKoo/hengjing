<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'
import { useMessage } from 'naive-ui'
import { onMounted, ref } from 'vue'

interface CliInstallStatus {
  installed: boolean
  hengjing_installed: boolean
  deng_installed: boolean
  install_dir: string
  app_macos_dir: string | null
  manual_commands: string | null
}

const message = useMessage()
const status = ref<CliInstallStatus | null>(null)
const isLoading = ref(false)
const isInstalling = ref(false)
const showManualCommands = ref(false)

async function loadStatus() {
  isLoading.value = true
  try {
    status.value = await invoke('get_cli_install_status')
  }
  catch (error) {
    console.error('获取 CLI 安装状态失败:', error)
  }
  finally {
    isLoading.value = false
  }
}

async function installCli() {
  isInstalling.value = true
  try {
    const result = await invoke('install_cli')
    message.success(result as string)
    await loadStatus()
  }
  catch (error) {
    const errorMsg = error as string
    if (errorMsg.includes('Permission denied') || errorMsg.includes('权限')) {
      message.warning('需要管理员权限，请使用下方的手动安装命令')
      showManualCommands.value = true
    }
    else {
      message.error(`安装失败: ${errorMsg}`)
    }
  }
  finally {
    isInstalling.value = false
  }
}

function copyCommands() {
  if (status.value?.manual_commands) {
    navigator.clipboard.writeText(status.value.manual_commands)
    message.success('命令已复制到剪贴板')
  }
}

onMounted(loadStatus)
</script>

<template>
  <n-space vertical size="large">
    <!-- 安装状态 -->
    <div class="flex items-center justify-between">
      <div class="flex items-center">
        <div class="w-1.5 h-1.5 rounded-full mr-3 flex-shrink-0" :class="status?.installed ? 'bg-success' : 'bg-warning'" />
        <div>
          <div class="text-sm font-medium leading-relaxed">
            CLI 工具状态
          </div>
          <div class="text-xs opacity-60">
            <template v-if="isLoading">
              检查中...
            </template>
            <template v-else-if="status?.installed">
              已安装到 {{ status.install_dir }}
            </template>
            <template v-else-if="status?.app_macos_dir">
              未安装 - 点击安装按钮部署到系统路径
            </template>
            <template v-else>
              请先从 .app 包启动应用
            </template>
          </div>
        </div>
      </div>
      <n-button
        v-if="status?.app_macos_dir && !status?.installed"
        size="small"
        type="primary"
        :loading="isInstalling"
        @click="installCli"
      >
        <template #icon>
          <div class="i-carbon-download w-4 h-4" />
        </template>
        一键安装
      </n-button>
      <n-tag v-else-if="status?.installed" type="success" size="small">
        已安装
      </n-tag>
    </div>

    <!-- 详细状态 -->
    <div v-if="status && !isLoading" class="flex items-start">
      <div class="w-1.5 h-1.5 bg-info rounded-full mr-3 flex-shrink-0 mt-2" />
      <div class="flex-1">
        <div class="text-sm font-medium leading-relaxed mb-2">
          工具详情
        </div>
        <div class="text-xs opacity-60 space-y-1">
          <div class="flex items-center gap-2">
            <span :class="status.hengjing_installed ? 'text-green-500' : 'text-orange-500'">
              {{ status.hengjing_installed ? '✓' : '✗' }}
            </span>
            <code class="bg-black/10 dark:bg-white/10 px-1 rounded">恒境</code>
            <span class="opacity-60">MCP 服务器</span>
          </div>
          <div class="flex items-center gap-2">
            <span :class="status.deng_installed ? 'text-green-500' : 'text-orange-500'">
              {{ status.deng_installed ? '✓' : '✗' }}
            </span>
            <code class="bg-black/10 dark:bg-white/10 px-1 rounded">等</code>
            <span class="opacity-60">设置界面</span>
          </div>
        </div>
      </div>
    </div>

    <!-- 手动安装命令 -->
    <div v-if="status?.manual_commands && (showManualCommands || !status?.installed)" class="flex items-start">
      <div class="w-1.5 h-1.5 bg-warning rounded-full mr-3 flex-shrink-0 mt-2" />
      <div class="flex-1">
        <div class="text-sm font-medium leading-relaxed mb-2">
          手动安装命令
        </div>
        <div class="text-xs opacity-60 mb-2">
          如果自动安装失败，请在终端中执行以下命令：
        </div>
        <pre class="command-block">{{ status.manual_commands }}</pre>
        <n-button
          size="small"
          type="primary"
          ghost
          class="mt-3"
          @click="copyCommands"
        >
          <template #icon>
            <div class="i-carbon-copy w-4 h-4" />
          </template>
          复制命令
        </n-button>
      </div>
    </div>
  </n-space>
</template>

<style scoped>
.command-block {
  background: #1a1a2e;
  color: #e0e0e0;
  padding: 12px 16px;
  border-radius: 8px;
  font-size: 12px;
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  overflow-x: auto;
  white-space: pre-wrap;
  word-break: break-all;
  border: 1px solid rgba(255, 255, 255, 0.1);
}
</style>
