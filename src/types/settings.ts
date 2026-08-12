/* ============================================================================
    Settings Types - ttsbard-echo
    ============================================================================ */

import { InjectionKey, Ref } from 'vue'

/* ==========================================================================
    Theme
    ========================================================================== */
export type Theme = 'dark' | 'light'

/* ==========================================================================
    Connections
    ========================================================================== */
export interface ConnectionConfig {
  id: string
  name: string
  url: string
  enabled: boolean
  access_token?: string
}

export interface ConnectionRuntimeSnapshot {
  id: string
  status: import('./types').ConnectionStatus
  lastMessage?: string
  errorMessage?: string
  isTyping: boolean
  previewText?: string
}

/* ==========================================================================
    Logging
    ========================================================================== */
export interface LoggingSettingsDto {
  enabled: boolean
  level: string
  module_levels: Record<string, string>
}

/* ==========================================================================
    Hotkeys
    ========================================================================== */
export interface HotkeySettingsDto {
  enabled: boolean
  toggle_window?: string
}

/* ==========================================================================
    General
    ========================================================================== */
export interface GeneralSettingsDto {
  exclude_from_capture: boolean
  theme?: Theme
  message_clear_interval_seconds: number
}

/* ==========================================================================
    Windows
    ========================================================================== */
export interface WindowPositionDto {
  x?: number
  y?: number
}

export interface FloatingWindowSettingsDto {
  x?: number
  y?: number
  opacity: number
  bg_color: string
  clickthrough: boolean
  use_custom_color: boolean
  visible: boolean
}

export interface FloatingAppearanceDto {
  opacity: number
  bg_color: string
  use_custom_color: boolean
  clickthrough: boolean
}

export function opacityToTransparency(opacity: number): number {
  return 100 - Math.min(100, Math.max(10, opacity))
}

export function transparencyToOpacity(transparency: number): number {
  return 100 - Math.min(90, Math.max(0, transparency))
}

export interface WindowsSettingsDto {
  main: WindowPositionDto
  floating: FloatingWindowSettingsDto
}

/* ==========================================================================
    Main Settings DTO
    ========================================================================== */
export interface AppSettingsDto {
  connections: ConnectionConfig[]
  logging: LoggingSettingsDto
  hotkeys: HotkeySettingsDto
  general: GeneralSettingsDto
  windows: WindowsSettingsDto
}

/* ==========================================================================
    Injection Key
    ========================================================================== */
export interface AppSettingsContext {
  settings: Ref<AppSettingsDto | null>
  isLoading: Ref<boolean>
  error: Ref<string | null>
  reload: () => Promise<void>
  cleanup?: () => void
}

export const APP_SETTINGS_KEY: InjectionKey<AppSettingsContext> =
  Symbol('app-settings')
