/* ============================================================================
   General Types - ttsbard-echo
   ============================================================================ */

import type { Component } from 'vue'

/* ==========================================================================
   Panel Types
   ========================================================================== */
export type Panel = 'connections' | 'settings'

export interface FloatingVisibilityPayload {
  visible: boolean
}

/* ==========================================================================
   Connection Status
   ========================================================================== */
export type ConnectionStatus = 
  | 'Disconnected'
  | 'Connecting'
  | 'Connected'
  | `Error: ${string}`

/* ==========================================================================
   Message Types
   ========================================================================== */
export type MessageType = 'info' | 'success' | 'warning' | 'error'

/* ==========================================================================
   Provider Types
   ========================================================================== */
export type ProviderType = 'openai' | 'elevenlabs' | 'azure' | 'google' | 'custom'

/* ==========================================================================
   Toast/Notification Types
   ========================================================================== */
export interface ToastMessage {
  id: string
  type: MessageType
  message: string
  duration?: number
}

/* ==========================================================================
   Component Props Types
   ========================================================================== */
export interface StatusMessageProps {
  type: MessageType
  message: string
  timeout?: number
}

export interface InputWithToggleProps {
  modelValue: string
  enabled: boolean
  label: string
  placeholder?: string
}

/* ==========================================================================
   Icon Type
   ========================================================================== */
export type IconComponent = Component
