<script setup lang="ts">
import { useDebounceFn } from '@vueuse/core'
import { NInput } from 'naive-ui'
import { ref } from 'vue'

interface Emits {
  search: [query: string]
}

const emit = defineEmits<Emits>()

const searchQuery = ref('')

// 防抖搜索，300ms延迟
const debouncedSearch = useDebounceFn(() => {
  emit('search', searchQuery.value)
}, 300)

// 处理输入变化
function handleInput() {
  debouncedSearch()
}

// 清空搜索
function clearSearch() {
  searchQuery.value = ''
  emit('search', '')
}
</script>

<template>
  <div class="search-bar">
    <NInput
      v-model:value="searchQuery"
      placeholder="搜索会话..."
      clearable
      @input="handleInput"
      @clear="clearSearch"
    >
      <template #prefix>
        <div class="i-carbon-search w-4 h-4 text-on-surface-secondary" />
      </template>
    </NInput>
  </div>
</template>

<style scoped>
.search-bar {
  width: 100%;
}
</style>
