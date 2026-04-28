import { describe, expect, it } from 'vitest'
import {
  buildPopupSubmitResponse,
  canSendPopupResponse,
  DEFAULT_EMPTY_SUBMIT_TEXT,
  hasPopupInputContent,
} from './popupSubmission'

describe('popupSubmission', () => {
  it('允许在弹窗为空时直接发送', () => {
    expect(canSendPopupResponse(null)).toBe(false)
    expect(canSendPopupResponse({
      id: 'request-1',
      message: '继续处理',
      is_markdown: true,
    })).toBe(true)

    expect(hasPopupInputContent({
      userInput: '',
      selectedOptions: [],
      draggedImages: [],
    })).toBe(false)
  })

  it('为空提交补默认确认文本', () => {
    const response = buildPopupSubmitResponse({
      userInput: '   ',
      selectedOptions: [],
      draggedImages: [],
    }, 'request-1')

    expect(response.user_input).toBe(DEFAULT_EMPTY_SUBMIT_TEXT)
    expect(response.selected_options).toEqual([])
    expect(response.images).toEqual([])
    expect(response.metadata.request_id).toBe('request-1')
    expect(response.metadata.source).toBe('popup')
  })

  it('保留用户输入的文本和图片', () => {
    const response = buildPopupSubmitResponse({
      userInput: '  补充说明  ',
      selectedOptions: ['查看文件内容'],
      draggedImages: ['data:image/png;base64,Zm9vYmFy'],
    }, 'request-2')

    expect(hasPopupInputContent({
      userInput: '  补充说明  ',
      selectedOptions: ['查看文件内容'],
      draggedImages: ['data:image/png;base64,Zm9vYmFy'],
    })).toBe(true)

    expect(response.user_input).toBe('补充说明')
    expect(response.selected_options).toEqual(['查看文件内容'])
    expect(response.images).toEqual([{
      data: 'Zm9vYmFy',
      media_type: 'image/png',
      filename: null,
    }])
  })
})
