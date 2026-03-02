/**
 * 会话历史记录类型定义
 *
 * 这些类型定义与 Rust 后端的类型保持一致
 * 参考: src/rust/session_history/models.rs
 */

/**
 * 会话来源类型
 *
 * - send: 用户点击"发送"按钮触发的会话
 * - continue: 用户点击"继续"按钮触发的会话
 * - enhance: 用户点击"增强"按钮触发的会话
 */
export type SessionSource = 'send' | 'continue' | 'enhance'

/**
 * 图片附件
 *
 * 包含 Base64 编码的图片数据和元数据
 */
export interface ImageAttachment {
  /** Base64 编码的图片数据 */
  data: string
  /** 图片的 MIME 类型，如 "image/png", "image/jpeg" */
  mediaType: string
  /** 可选的文件名 */
  filename?: string | null
}

/**
 * 会话数据（用于保存）
 *
 * 用于从前端发送到后端保存会话时使用
 */
export interface SessionData {
  /** 会话来源类型 */
  source: SessionSource
  /** 用户输入的文本内容（可选） */
  userInput?: string | null
  /** AI 的回复内容（Markdown 格式） */
  aiResponse: string
  /** 用户选择的预定义选项 */
  selectedOptions: string[]
  /** 用户上传的截图列表 */
  images: ImageAttachment[]
}

/**
 * 会话记录
 *
 * 完整的会话记录，包含所有数据和元数据
 */
export interface SessionRecord {
  /** 唯一标识符（UUID） */
  id: string
  /** 会话创建时间戳（ISO 8601 格式） */
  timestamp: string
  /** 会话来源类型 */
  source: SessionSource
  /** 用户输入的文本内容（可选） */
  userInput?: string | null
  /** AI 的回复内容（Markdown 格式） */
  aiResponse: string
  /** 用户选择的预定义选项 */
  selectedOptions: string[]
  /** 用户上传的截图列表 */
  images: ImageAttachment[]
}

/**
 * 侧边栏状态
 *
 * 用于持久化侧边栏的展开/收起状态和宽度
 */
export interface SidebarState {
  /** 侧边栏是否展开 */
  isExpanded: boolean
  /** 侧边栏宽度（像素） */
  width: number
}

/**
 * 会话更新数据（用于更新已存在的会话）
 */
export interface SessionUpdateData {
  /** 会话来源类型 */
  source: SessionSource
  /** 用户输入的文本内容（可选） */
  userInput?: string | null
  /** 用户选择的预定义选项 */
  selectedOptions: string[]
  /** 用户上传的截图列表 */
  images: ImageAttachment[]
}

/**
 * 会话预览（用于列表显示）
 *
 * 用于在侧边栏列表中显示会话的简要信息
 */
export interface SessionPreview {
  /** 唯一标识符 */
  id: string
  /** 会话创建时间戳（ISO 8601 格式） */
  timestamp: string
  /** 内容摘要（前 50 个字符） */
  summary: string
  /** 是否包含截图 */
  hasImages: boolean
  /** 会话来源类型 */
  source: SessionSource
}
