import { invoke } from '@tauri-apps/api/core'

export interface AIRole {
  name: string
  description: string
  system_prompt: string
  temperature: number
  max_tokens: number
  tools_enabled: boolean
  context_role_template: string
  context_task_template: string
  context_instructions_template: string
}

export interface AIRoleEntry {
  id: string
  role: AIRole
}

export interface AIConfig {
  default_role: string
  roles: Record<string, AIRole>
}

export class AIConfigService {
  static async getConfig(): Promise<AIConfig> {
    return await invoke<AIConfig>('get_ai_config')
  }

  static async getRole(roleId: string): Promise<AIRole | null> {
    return await invoke<AIRole | null>('get_ai_role', { roleId })
  }

  static async updateRole(roleId: string, role: AIRole): Promise<void> {
    await invoke<void>('update_ai_role', { roleId, role })
  }

  static async createRole(role: AIRole): Promise<string> {
    return await invoke<string>('add_ai_role', { role })
  }

  static async deleteRole(roleId: string): Promise<void> {
    await invoke<void>('delete_ai_role', { roleId })
  }

  static async setDefaultRole(roleId: string): Promise<void> {
    await invoke<void>('set_default_ai_role', { roleId })
  }

  static async getAllRoles(): Promise<AIRoleEntry[]> {
    return await invoke<AIRoleEntry[]>('get_all_ai_roles')
  }
}
