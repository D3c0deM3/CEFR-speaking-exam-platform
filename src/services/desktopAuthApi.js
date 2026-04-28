const configuredApiOrigin = (import.meta.env.VITE_DESKTOP_AUTH_API_BASE_URL || '').trim().replace(/\/+$/, '')
const defaultApiOrigin = 'https://guarded-fortress-75118-0236acf5cd86.herokuapp.com'

const apiOrigin = configuredApiOrigin || defaultApiOrigin
const API_BASE_URL = apiOrigin.endsWith('/api') ? apiOrigin : `${apiOrigin}/api`

async function request(path, options = {}) {
  const response = await fetch(`${API_BASE_URL}${path}`, {
    headers: {
      'Content-Type': 'application/json',
      ...(options.headers || {})
    },
    ...options
  })

  const data = await response.json().catch(() => ({}))

  if (!response.ok) {
    const error = new Error(data.error || 'Request failed')
    error.status = response.status
    error.data = data
    throw error
  }

  return data
}

export const desktopAuthApi = {
  login(credentials) {
    return request('/desktop-auth/login', {
      method: 'POST',
      body: JSON.stringify(credentials)
    })
  },

  register(credentials) {
    return request('/desktop-auth/register', {
      method: 'POST',
      body: JSON.stringify(credentials)
    })
  }
}
