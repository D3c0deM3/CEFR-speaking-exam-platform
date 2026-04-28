import { computed, ref } from 'vue'
import { defineStore } from 'pinia'
import { desktopAuthApi } from '../services/desktopAuthApi'

const TOKEN_KEY = 'desktopAppAuthToken'
const USER_KEY = 'desktopAppAuthUser'

export const useAuthStore = defineStore('auth', () => {
  const token = ref(sessionStorage.getItem(TOKEN_KEY) || '')
  const user = ref(JSON.parse(sessionStorage.getItem(USER_KEY) || 'null'))
  const isLoading = ref(false)

  const isAuthenticated = computed(() => Boolean(token.value && user.value))

  function persistSession(nextToken, nextUser) {
    token.value = nextToken
    user.value = nextUser
    sessionStorage.setItem(TOKEN_KEY, nextToken)
    sessionStorage.setItem(USER_KEY, JSON.stringify(nextUser))
  }

  function restoreSession() {
    token.value = sessionStorage.getItem(TOKEN_KEY) || ''
    user.value = JSON.parse(sessionStorage.getItem(USER_KEY) || 'null')
  }

  async function login(credentials) {
    isLoading.value = true
    try {
      const data = await desktopAuthApi.login(credentials)
      persistSession(data.token, data.user)
      return data
    } finally {
      isLoading.value = false
    }
  }

  async function register(credentials) {
    isLoading.value = true
    try {
      return await desktopAuthApi.register(credentials)
    } finally {
      isLoading.value = false
    }
  }

  function logout() {
    token.value = ''
    user.value = null
    sessionStorage.removeItem(TOKEN_KEY)
    sessionStorage.removeItem(USER_KEY)
  }

  return {
    token,
    user,
    isLoading,
    isAuthenticated,
    restoreSession,
    login,
    register,
    logout
  }
})
