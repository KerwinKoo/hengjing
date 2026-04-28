import type { ImageAttachment, McpRequest, McpResponse } from '../../types/popup'

export const DEFAULT_EMPTY_SUBMIT_TEXT = '用户确认继续'

export interface PopupInputState {
  selectedOptions: string[]
  userInput: string
  draggedImages: string[]
}

export function hasPopupInputContent(state: PopupInputState): boolean {
  return state.selectedOptions.length > 0
    || state.userInput.trim().length > 0
    || state.draggedImages.length > 0
}

export function canSendPopupResponse(request: McpRequest | null): boolean {
  return request !== null
}

function mapDraggedImagesToAttachments(draggedImages: string[]): ImageAttachment[] {
  return draggedImages.map(imageData => ({
    data: imageData.split(',')[1] ?? imageData,
    media_type: 'image/png',
    filename: null,
  }))
}

export function buildPopupSubmitResponse(
  state: PopupInputState,
  requestId: string | null,
): McpResponse {
  const response: McpResponse = {
    user_input: state.userInput.trim() || null,
    selected_options: state.selectedOptions,
    images: mapDraggedImagesToAttachments(state.draggedImages),
    metadata: {
      timestamp: new Date().toISOString(),
      request_id: requestId,
      source: 'popup',
    },
  }

  if (!response.user_input && response.selected_options.length === 0 && response.images.length === 0) {
    response.user_input = DEFAULT_EMPTY_SUBMIT_TEXT
  }

  return response
}
