/**
 * Preservation Property Tests for Popup Session History Bug Fix
 * 
 * **Validates: Requirements 3.1, 3.2, 3.3, 3.4, 3.5**
 * 
 * These tests verify that non-popup scenarios continue to work correctly
 * after the bug fix is implemented. They establish the baseline behavior
 * that must be preserved.
 * 
 * CRITICAL: These tests should PASS on UNFIXED code.
 * They capture the current correct behavior of non-popup scenarios.
 * After the fix, these tests should STILL PASS (no regression).
 * 
 * The preservation property: All session operations that don't involve
 * popup session creation should behave identically before and after the fix.
 */

import { beforeEach, describe, expect, vi } from 'vitest'
import { test as fcTest } from '@fast-check/vitest'
import * as fc from 'fast-check'
import { invoke } from '@tauri-apps/api/core'
import type { SessionRecord } from '../types/session'

// Mock Tauri invoke
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

describe('Preservation Property: Non-Popup Scenarios Behavior', () => {
  beforeEach(() => {
    // Reset all mocks before each test
    vi.clearAllMocks()
    
    // Reset the singleton instance by clearing the module cache
    vi.resetModules()
  })

  fcTest.prop([
    fc.record({
      userInput: fc.string({ minLength: 1, maxLength: 100 }),
      aiResponse: fc.string({ minLength: 1, maxLength: 200 }),
      selectedOptions: fc.array(fc.string(), { maxLength: 3 }),
    }),
  ], { timeout: 10000 })('Property 2.1: Main interface session creation works correctly', async (testCase) => {
    /**
     * **Validates: Requirement 3.1**
     * 
     * This test verifies that session creation in non-popup scenarios
     * (e.g., main interface) works correctly on unfixed code.
     * 
     * Expected behavior (should PASS on unfixed code):
     * - saveSession() successfully creates a session
     * - The session is added to sessions.value
     * - A valid session ID is returned
     * - The session data matches what was provided
     */
    
    const mockInvoke = vi.mocked(invoke)
    
    // Mock successful session creation (this is the baseline behavior)
    const mockSessionId = `session-${Date.now()}`
    const mockSessionRecord: SessionRecord = {
      id: mockSessionId,
      timestamp: new Date().toISOString(),
      source: 'send',
      userInput: testCase.userInput,
      aiResponse: testCase.aiResponse,
      selectedOptions: testCase.selectedOptions,
      images: [],
    }
    
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'save_session') {
        return Promise.resolve(mockSessionRecord)
      }
      if (cmd === 'load_sessions') {
        return Promise.resolve([])
      }
      return Promise.reject(new Error(`Unknown command: ${cmd}`))
    })

    const { useSessionHistory } = await import('./useSessionHistory')
    const sessionHistory = useSessionHistory()
    
    await sessionHistory.initialize()
    
    // Save a session (non-popup scenario)
    const sessionData = {
      source: 'send',
      userInput: testCase.userInput,
      aiResponse: testCase.aiResponse,
      selectedOptions: testCase.selectedOptions,
      images: [],
    }
    
    const result = await sessionHistory.saveSession(sessionData)
    
    // Verify session was created successfully
    expect(result).not.toBeNull()
    expect(result.id).toBe(mockSessionId)
    expect(sessionHistory.sessions.value).toHaveLength(1)
    expect(sessionHistory.sessions.value[0].id).toBe(mockSessionId)
    expect(sessionHistory.sessions.value[0].userInput).toBe(testCase.userInput)
    expect(sessionHistory.sessions.value[0].aiResponse).toBe(testCase.aiResponse)
  })

  fcTest.prop([
    fc.array(
      fc.record({
        id: fc.string({ minLength: 5, maxLength: 20 }),
        userInput: fc.string({ minLength: 1, maxLength: 100 }),
        aiResponse: fc.string({ minLength: 1, maxLength: 200 }),
      }),
      { minLength: 1, maxLength: 5 }
    ),
    fc.integer({ min: 0, max: 4 }),
  ], { timeout: 10000 })('Property 2.2: Session deletion works correctly', async (sessions, deleteIndex) => {
    /**
     * **Validates: Requirement 3.2**
     * 
     * This test verifies that session deletion works correctly on unfixed code.
     * 
     * Expected behavior (should PASS on unfixed code):
     * - deleteSession() successfully removes a session
     * - The session is removed from sessions.value
     * - Other sessions remain unchanged
     */
    
    const mockInvoke = vi.mocked(invoke)
    
    const mockSessions: SessionRecord[] = sessions.map(s => ({
      id: s.id,
      timestamp: new Date().toISOString(),
      source: 'send',
      userInput: s.userInput,
      aiResponse: s.aiResponse,
      selectedOptions: [],
      images: [],
    }))
    
    mockInvoke.mockImplementation((cmd: string, _args?: any) => {
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
    expect(initialLength).toBe(sessions.length)
    
    // Delete a session
    const sessionToDelete = sessionHistory.sessions.value[deleteIndex % initialLength]
    await sessionHistory.deleteSession(sessionToDelete.id)
    
    // Verify session was deleted
    expect(sessionHistory.sessions.value).toHaveLength(initialLength - 1)
    expect(sessionHistory.sessions.value.find(s => s.id === sessionToDelete.id)).toBeUndefined()
  })

  fcTest.prop([
    fc.array(
      fc.record({
        id: fc.string({ minLength: 5, maxLength: 20 }),
        userInput: fc.string({ minLength: 1, maxLength: 100 }),
        aiResponse: fc.string({ minLength: 1, maxLength: 200 }),
      }),
      { minLength: 3, maxLength: 10 }
    ),
    fc.string({ minLength: 1, maxLength: 20 }),
  ], { timeout: 10000 })('Property 2.3: Session search works correctly', async (sessions, searchQuery) => {
    /**
     * **Validates: Requirement 3.3**
     * 
     * This test verifies that session search works correctly on unfixed code.
     * 
     * Expected behavior (should PASS on unfixed code):
     * - searchSessions() returns sessions matching the query
     * - Search works on userInput, aiResponse, and selectedOptions
     * - Empty query returns all sessions
     */
    
    const mockInvoke = vi.mocked(invoke)
    
    // Create sessions with predictable content for search testing
    const mockSessions: SessionRecord[] = sessions.map((s, index) => ({
      id: s.id,
      timestamp: new Date().toISOString(),
      source: 'send',
      userInput: index === 0 ? searchQuery : s.userInput,
      aiResponse: index === 1 ? searchQuery : s.aiResponse,
      selectedOptions: index === 2 ? [searchQuery] : [],
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
    
    // Search for sessions
    const results = sessionHistory.searchSessions(searchQuery)
    
    // Verify search results
    expect(results.length).toBeGreaterThan(0)
    expect(results.length).toBeLessThanOrEqual(3) // At most 3 matches (userInput, aiResponse, selectedOptions)
    
    // Verify each result contains the search query
    results.forEach(result => {
      const matchesUserInput = result.userInput?.toLowerCase().includes(searchQuery.toLowerCase())
      const matchesAiResponse = result.aiResponse.toLowerCase().includes(searchQuery.toLowerCase())
      const matchesOptions = result.selectedOptions.some(opt => opt.toLowerCase().includes(searchQuery.toLowerCase()))
      
      expect(matchesUserInput || matchesAiResponse || matchesOptions).toBe(true)
    })
  })

  fcTest.prop([
    fc.record({
      id: fc.string({ minLength: 5, maxLength: 20 }),
      userInput: fc.string({ minLength: 1, maxLength: 100 }),
      aiResponse: fc.string({ minLength: 1, maxLength: 200 }),
      images: fc.array(
        fc.record({
          data: fc.string({ minLength: 10, maxLength: 50 }),
          mediaType: fc.constantFrom('image/png', 'image/jpeg'),
          filename: fc.option(fc.string({ minLength: 5, maxLength: 20 }), { nil: null }),
        }),
        { maxLength: 2 }
      ),
    }),
  ], { timeout: 10000 })('Property 2.4: Session detail loading works correctly', async (sessionData) => {
    /**
     * **Validates: Requirement 3.4**
     * 
     * This test verifies that loading session details (including screenshots)
     * works correctly on unfixed code.
     * 
     * Expected behavior (should PASS on unfixed code):
     * - getSession() returns the complete session data
     * - Screenshots are included in the response
     * - Session data matches what was stored
     */
    
    const mockInvoke = vi.mocked(invoke)
    
    const mockSession: SessionRecord = {
      id: sessionData.id,
      timestamp: new Date().toISOString(),
      source: 'send',
      userInput: sessionData.userInput,
      aiResponse: sessionData.aiResponse,
      selectedOptions: [],
      images: sessionData.images,
    }
    
    mockInvoke.mockImplementation((cmd: string, _args?: any) => {
      if (cmd === 'get_session') {
        return Promise.resolve(mockSession)
      }
      if (cmd === 'load_sessions') {
        return Promise.resolve([])
      }
      return Promise.reject(new Error(`Unknown command: ${cmd}`))
    })

    const { useSessionHistory } = await import('./useSessionHistory')
    const sessionHistory = useSessionHistory()
    
    await sessionHistory.initialize()
    
    // Get session details
    const result = await sessionHistory.getSession(sessionData.id)
    
    // Verify session details are complete
    expect(result).not.toBeNull()
    expect(result?.id).toBe(sessionData.id)
    expect(result?.userInput).toBe(sessionData.userInput)
    expect(result?.aiResponse).toBe(sessionData.aiResponse)
    expect(result?.images).toEqual(sessionData.images)
  })

  fcTest.prop([
    fc.array(
      fc.record({
        id: fc.string({ minLength: 5, maxLength: 20 }),
        userInput: fc.string({ minLength: 1, maxLength: 100 }),
        aiResponse: fc.string({ minLength: 1, maxLength: 200 }),
      }),
      { minLength: 0, maxLength: 10 }
    ),
  ], { timeout: 10000 })('Property 2.5: App initialization loads sessions correctly', async (sessions) => {
    /**
     * **Validates: Requirement 3.5**
     * 
     * This test verifies that app initialization loads session history
     * correctly on unfixed code.
     * 
     * Expected behavior (should PASS on unfixed code):
     * - initialize() loads all historical sessions
     * - sessions.value contains all loaded sessions
     * - Sessions are ordered correctly (newest first)
     */
    
    const mockInvoke = vi.mocked(invoke)
    
    const mockSessions: SessionRecord[] = sessions.map(s => ({
      id: s.id,
      timestamp: new Date().toISOString(),
      source: 'send',
      userInput: s.userInput,
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
    expect(sessionHistory.sessions.value).toHaveLength(sessions.length)
    
    // Verify session data matches
    sessionHistory.sessions.value.forEach((session, index) => {
      expect(session.id).toBe(mockSessions[index].id)
      expect(session.userInput).toBe(mockSessions[index].userInput)
      expect(session.aiResponse).toBe(mockSessions[index].aiResponse)
    })
  })
})
