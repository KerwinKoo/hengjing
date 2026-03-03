/**
 * Bug Condition Exploration Test for Popup Session History Empty Bug
 * 
 * **Validates: Requirements 2.1, 2.2, 2.5**
 * 
 * This test explores the bug condition where popup session creation fails.
 * 
 * CRITICAL: This test asserts the EXPECTED BEHAVIOR.
 * It FAILS on unfixed code (proving the bug exists).
 * After the fix is implemented, this test will PASS (proving the fix works).
 * 
 * The bug: When popup opens in test/mock mode, saveSession() calls real Tauri invoke
 * which fails because the backend is not available, resulting in empty session history.
 */

import { beforeEach, describe, expect, it, vi } from 'vitest'
import { test as fcTest } from '@fast-check/vitest'
import * as fc from 'fast-check'
import { invoke } from '@tauri-apps/api/core'
import type { SessionRecord } from '../types/session'

// Mock Tauri invoke
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

describe('Bug Condition Exploration: Popup Session Creation Failure', () => {
  beforeEach(() => {
    // Reset all mocks before each test
    vi.clearAllMocks()
    
    // Reset the singleton instance by clearing the module cache
    vi.resetModules()
  })

  fcTest.prop([
    fc.record({
      aiResponse: fc.string({ minLength: 1, maxLength: 200 }),
    }),
  ], { timeout: 10000 })('Property 1: Fault Condition - Popup session creation should succeed (FAILS on unfixed code)', async (testCase) => {
    /**
     * **Validates: Requirements 2.1, 2.2, 2.5**
     * 
     * This test asserts the EXPECTED behavior: when popup opens and saveSession() is called,
     * the session SHOULD be created and appear in the history list.
     * 
     * On UNFIXED code, this test FAILS because:
     * - invoke('save_session') times out or fails
     * - sessions.value remains empty
     * - No session ID is returned
     * 
     * After the FIX, this test PASSES because:
     * - Mock mode support is added
     * - Sessions are created without calling real Tauri backend
     * - sessions.value contains the new session
     * 
     * COUNTEREXAMPLES (found on unfixed code):
     * - saveSession() is called but sessions.value.length = 0
     * - Console shows "invoke 调用超时" or "Backend not available"
     * - currentSessionId.value remains null
     * - updateSession() cannot work because there's no session ID
     */
    
    // Simulate the actual bug: invoke fails because backend is unavailable in test mode
    const mockInvoke = vi.mocked(invoke)
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'save_session') {
        // Simulate backend not available (this is what happens in unfixed code)
        return Promise.reject(new Error('Backend not available'))
      }
      if (cmd === 'load_sessions') {
        // Return empty array - no existing sessions
        return Promise.resolve([])
      }
      return Promise.reject(new Error(`Unknown command: ${cmd}`))
    })

    // Get session history instance
    const { useSessionHistory } = await import('./useSessionHistory')
    const sessionHistory = useSessionHistory()
    
    // Enable mock mode (this is the FIX)
    sessionHistory.setMockMode(true)
    
    // Initialize (should load empty sessions)
    await sessionHistory.initialize()
    
    // Verify initial state: sessions list is empty
    expect(sessionHistory.sessions.value).toHaveLength(0)
    
    // Simulate popup opening and calling saveSession()
    const sessionData = {
      source: 'send',
      userInput: null,
      aiResponse: testCase.aiResponse,
      selectedOptions: [],
      images: [],
    }
    
    // Attempt to save session
    // On UNFIXED code: this throws an error
    // On FIXED code: this succeeds with mock mode support
    let result: { id: string } | null = null
    let saveError: Error | null = null
    
    try {
      result = await sessionHistory.saveSession(sessionData)
    }
    catch (error) {
      saveError = error as Error
    }
    
    // ASSERT EXPECTED BEHAVIOR (these assertions FAIL on unfixed code):
    
    // 1. saveSession() should NOT throw an error
    expect(saveError).toBeNull()
    
    // 2. saveSession() should return a valid session ID
    expect(result).not.toBeNull()
    expect(result?.id).toBeTruthy()
    
    // 3. sessions.value should contain the new session
    expect(sessionHistory.sessions.value.length).toBeGreaterThan(0)
    
    // 4. The new session should be at the beginning of the list (newest first)
    expect(sessionHistory.sessions.value[0].aiResponse).toBe(testCase.aiResponse)
    
    // 5. The session ID should match the returned ID
    if (result) {
      expect(sessionHistory.sessions.value[0].id).toBe(result.id)
    }
    
    // This test FAILS on unfixed code, proving the bug exists.
    // After the fix (adding mock mode support), this test will PASS.
  })
})

/**
 * Preservation Property Tests
 * 
 * **Validates: Requirements 3.1, 3.2, 3.3, 3.4, 3.5**
 * 
 * These tests verify that non-popup scenarios continue to work correctly.
 * They should PASS on unfixed code (establishing baseline behavior).
 * After the fix, they should still PASS (confirming no regression).
 */

describe('Preservation: Non-Popup Scenarios', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    vi.resetModules()
  })

  fcTest.prop([
    fc.record({
      userInput: fc.string({ minLength: 1, maxLength: 100 }),
      aiResponse: fc.string({ minLength: 1, maxLength: 200 }),
    }),
  ])('Property 2.1: Main interface session creation works correctly', async (testCase) => {
    /**
     * **Validates: Requirement 3.1**
     * 
     * Non-popup session creation (e.g., from main interface) should work correctly.
     * This test establishes baseline behavior that must be preserved after the fix.
     */
    
    const mockInvoke = vi.mocked(invoke)
    const mockSessionId = `session-${Date.now()}`
    
    mockInvoke.mockImplementation((cmd: string, args?: any) => {
      if (cmd === 'save_session') {
        // Simulate successful backend save - return full SessionRecord
        const mockRecord: SessionRecord = {
          id: mockSessionId,
          timestamp: Date.now(),
          source: args.session.source,
          userInput: args.session.userInput,
          aiResponse: args.session.aiResponse,
          selectedOptions: args.session.selectedOptions,
          images: args.session.images,
        }
        return Promise.resolve(mockRecord)
      }
      if (cmd === 'load_sessions') {
        return Promise.resolve([])
      }
      return Promise.reject(new Error(`Unknown command: ${cmd}`))
    })

    const { useSessionHistory } = await import('./useSessionHistory')
    const sessionHistory = useSessionHistory()
    
    await sessionHistory.initialize()
    
    const sessionData = {
      source: 'send',
      userInput: testCase.userInput,
      aiResponse: testCase.aiResponse,
      selectedOptions: [],
      images: [],
    }
    
    const result = await sessionHistory.saveSession(sessionData)
    
    // Verify session was created successfully
    expect(result).not.toBeNull()
    expect(result?.id).toBe(mockSessionId)
    expect(sessionHistory.sessions.value.length).toBeGreaterThan(0)
    expect(sessionHistory.sessions.value[0].userInput).toBe(testCase.userInput)
    expect(sessionHistory.sessions.value[0].aiResponse).toBe(testCase.aiResponse)
  })

  fcTest.prop([
    fc.array(fc.string({ minLength: 10, maxLength: 20 }), { minLength: 1, maxLength: 5 }),
  ])('Property 2.2: Session deletion works correctly', async (sessionIds) => {
    /**
     * **Validates: Requirement 3.2**
     * 
     * Session deletion should continue to work correctly after the fix.
     */
    
    const mockInvoke = vi.mocked(invoke)
    
    // Create mock sessions
    const mockSessions: SessionRecord[] = sessionIds.map((id, index) => ({
      id,
      timestamp: Date.now() - index * 1000,
      source: 'send',
      userInput: `User input ${index}`,
      aiResponse: `AI response ${index}`,
      selectedOptions: [],
      images: [],
    }))
    
    mockInvoke.mockImplementation((cmd: string, args?: any) => {
      if (cmd === 'load_sessions') {
        return Promise.resolve(mockSessions)
      }
      if (cmd === 'delete_session') {
        return Promise.resolve()
      }
      return Promise.reject(new Error(`Unknown command: ${cmd}`))
    })

    const { useSessionHistory } = await import('./useSessionHistory')
    const sessionHistory = useSessionHistory()
    
    await sessionHistory.initialize()
    
    const initialLength = sessionHistory.sessions.value.length
    expect(initialLength).toBe(sessionIds.length)
    
    // Delete first session
    const sessionToDelete = sessionHistory.sessions.value[0]
    await sessionHistory.deleteSession(sessionToDelete.id)
    
    // Verify session was removed
    expect(sessionHistory.sessions.value.length).toBe(initialLength - 1)
    expect(sessionHistory.sessions.value.find(s => s.id === sessionToDelete.id)).toBeUndefined()
  })

  fcTest.prop([
    fc.array(
      fc.record({
        id: fc.string({ minLength: 10, maxLength: 20 }),
        content: fc.string({ minLength: 5, maxLength: 50 }),
      }),
      { minLength: 3, maxLength: 10 }
    ),
    fc.string({ minLength: 1, maxLength: 10 }),
  ])('Property 2.3: Session search works correctly', async (sessions, searchTerm) => {
    /**
     * **Validates: Requirement 3.3**
     * 
     * Session search functionality should continue to work correctly after the fix.
     */
    
    const mockInvoke = vi.mocked(invoke)
    
    // Create mock sessions with searchable content
    const mockSessions: SessionRecord[] = sessions.map((s, index) => ({
      id: s.id,
      timestamp: Date.now() - index * 1000,
      source: 'send',
      userInput: `User ${s.content}`,
      aiResponse: `AI ${s.content}`,
      selectedOptions: [],
      images: [],
    }))
    
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'load_sessions') {
        return Promise.resolve(mockSessions)
      }
      return Promise.reject(new Error(`Unknown command: ${cmd}`))
    })

    const { useSessionHistory } = await import('./useSessionHistory')
    const sessionHistory = useSessionHistory()
    
    await sessionHistory.initialize()
    
    // Perform search using searchSessions method
    const results = sessionHistory.searchSessions(searchTerm)
    
    // Verify search results contain only matching sessions
    results.forEach(session => {
      const matchesUserInput = session.userInput?.toLowerCase().includes(searchTerm.toLowerCase())
      const matchesAiResponse = session.aiResponse?.toLowerCase().includes(searchTerm.toLowerCase())
      const matchesOptions = session.selectedOptions.some(opt => opt.toLowerCase().includes(searchTerm.toLowerCase()))
      expect(matchesUserInput || matchesAiResponse || matchesOptions).toBe(true)
    })
  })

  fcTest.prop([
    fc.string({ minLength: 10, maxLength: 20 }),
  ])('Property 2.4: Session detail loading works correctly', async (sessionId) => {
    /**
     * **Validates: Requirement 3.4**
     * 
     * Loading session details (including screenshots) should continue to work correctly.
     */
    
    const mockInvoke = vi.mocked(invoke)
    
    const mockSession: SessionRecord = {
      id: sessionId,
      timestamp: Date.now(),
      source: 'send',
      userInput: 'Test user input',
      aiResponse: 'Test AI response',
      selectedOptions: ['option1'],
      images: ['image1.png', 'image2.png'],
    }
    
    mockInvoke.mockImplementation((cmd: string, args?: any) => {
      if (cmd === 'get_session') {
        return Promise.resolve(mockSession)
      }
      return Promise.reject(new Error(`Unknown command: ${cmd}`))
    })

    const { useSessionHistory } = await import('./useSessionHistory')
    const sessionHistory = useSessionHistory()
    
    const detail = await sessionHistory.getSession(sessionId)
    
    // Verify all session data is loaded correctly
    expect(detail).not.toBeNull()
    expect(detail?.id).toBe(sessionId)
    expect(detail?.userInput).toBe(mockSession.userInput)
    expect(detail?.aiResponse).toBe(mockSession.aiResponse)
    expect(detail?.selectedOptions).toEqual(mockSession.selectedOptions)
    expect(detail?.images).toEqual(mockSession.images)
  })

  fcTest.prop([
    fc.array(
      fc.record({
        id: fc.string({ minLength: 10, maxLength: 20 }),
        aiResponse: fc.string({ minLength: 5, maxLength: 50 }),
      }),
      { minLength: 0, maxLength: 10 }
    ),
  ])('Property 2.5: App initialization loads sessions correctly', async (sessions) => {
    /**
     * **Validates: Requirement 3.5**
     * 
     * Application initialization should continue to load session history correctly.
     */
    
    const mockInvoke = vi.mocked(invoke)
    
    const mockSessions: SessionRecord[] = sessions.map((s, index) => ({
      id: s.id,
      timestamp: Date.now() - index * 1000,
      source: 'send',
      userInput: null,
      aiResponse: s.aiResponse,
      selectedOptions: [],
      images: [],
    }))
    
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'load_sessions') {
        return Promise.resolve(mockSessions)
      }
      return Promise.reject(new Error(`Unknown command: ${cmd}`))
    })

    const { useSessionHistory } = await import('./useSessionHistory')
    const sessionHistory = useSessionHistory()
    
    // Initialize should load all sessions
    await sessionHistory.initialize()
    
    // Verify all sessions were loaded
    expect(sessionHistory.sessions.value.length).toBe(sessions.length)
    sessions.forEach((s, index) => {
      expect(sessionHistory.sessions.value[index].id).toBe(s.id)
      expect(sessionHistory.sessions.value[index].aiResponse).toBe(s.aiResponse)
    })
  })
})
