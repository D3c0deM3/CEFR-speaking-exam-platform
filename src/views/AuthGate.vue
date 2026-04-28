<template>
  <main class="auth-page">
    <section class="auth-hero">
      <span class="eyebrow">CEFR Speaking</span>
      <h1>Sign in to start the mock test app</h1>
      <p>
        Accounts are activated by the desktop app admin. Register first, then sign in after your subscription is active.
      </p>
      <div class="status-strip">
        <span>30-day access</span>
        <span>Admin activated</span>
        <span>Secure login</span>
      </div>
    </section>

    <section class="auth-panel">
      <div class="mode-toggle" role="tablist" aria-label="Authentication mode">
        <button
          type="button"
          :class="{ active: mode === 'login' }"
          @click="switchMode('login')"
        >
          Login
        </button>
        <button
          type="button"
          :class="{ active: mode === 'register' }"
          @click="switchMode('register')"
        >
          Register
        </button>
      </div>

      <form class="auth-form" @submit.prevent="submit">
        <div class="form-copy">
          <h2>{{ mode === 'login' ? 'Welcome back' : 'Create account' }}</h2>
          <p>{{ formSubtitle }}</p>
        </div>

        <div v-if="message" class="notice" :class="messageType">
          {{ message }}
        </div>

        <label class="field">
          <span>Username</span>
          <input
            v-model.trim="form.username"
            type="text"
            autocomplete="username"
            required
            minlength="3"
            placeholder="desktop_user"
          />
        </label>

        <label v-if="mode === 'register'" class="field">
          <span>Email</span>
          <input
            v-model.trim="form.email"
            type="email"
            autocomplete="email"
            placeholder="name@example.com"
          />
        </label>

        <label class="field">
          <span>Password</span>
          <input
            v-model="form.password"
            type="password"
            autocomplete="current-password"
            required
            :minlength="mode === 'register' ? 6 : 1"
            placeholder="Enter password"
          />
        </label>

        <button class="submit-button" type="submit" :disabled="authStore.isLoading">
          {{ authStore.isLoading ? 'Please wait...' : submitLabel }}
        </button>
      </form>
    </section>
  </main>
</template>

<script setup>
import { computed, reactive, ref } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '../stores/auth'

const router = useRouter()
const authStore = useAuthStore()
const mode = ref('login')
const message = ref('')
const messageType = ref('info')

const form = reactive({
  username: '',
  email: '',
  password: ''
})

const formSubtitle = computed(() => {
  if (mode.value === 'login') {
    return 'Use your active subscription account to open the app.'
  }

  return 'New accounts stay inactive until the admin enables the subscription.'
})

const submitLabel = computed(() => mode.value === 'login' ? 'Login' : 'Register')

function switchMode(nextMode) {
  mode.value = nextMode
  message.value = ''
  messageType.value = 'info'
}

async function submit() {
  message.value = ''
  messageType.value = 'info'

  try {
    if (mode.value === 'login') {
      await authStore.login({
        username: form.username,
        password: form.password
      })
      router.replace('/')
      return
    }

    await authStore.register({
      username: form.username,
      email: form.email || undefined,
      password: form.password
    })

    form.password = ''
    mode.value = 'login'
    messageType.value = 'success'
    message.value = 'Registration saved. Ask the admin to activate your subscription, then log in.'
  } catch (error) {
    messageType.value = 'error'
    if (error.status === 403) {
      message.value = 'Your subscription is inactive or expired. Ask the admin to activate it.'
    } else {
      message.value = error.message || 'Authentication failed. Please try again.'
    }
  }
}
</script>

<style scoped>
.auth-page {
  min-height: 100vh;
  display: grid;
  grid-template-columns: minmax(0, 1.1fr) minmax(360px, 0.9fr);
  background:
    radial-gradient(circle at 12% 18%, rgba(249, 115, 22, 0.22), transparent 38%),
    radial-gradient(circle at 88% 12%, rgba(29, 78, 216, 0.16), transparent 36%),
    linear-gradient(135deg, #fff7ed 0%, #f8fafc 52%, #eef2ff 100%);
  color: #0f172a;
}

.auth-hero {
  display: flex;
  flex-direction: column;
  justify-content: center;
  padding: clamp(32px, 7vw, 96px);
}

.eyebrow {
  font-size: 12px;
  letter-spacing: 0.28em;
  text-transform: uppercase;
  color: #1d4ed8;
  font-weight: 700;
  margin-bottom: 18px;
}

.auth-hero h1 {
  font-family: 'Fraunces', serif;
  font-size: clamp(42px, 7vw, 76px);
  line-height: 0.96;
  max-width: 720px;
  margin-bottom: 24px;
}

.auth-hero p {
  max-width: 560px;
  color: #475569;
  font-size: 18px;
  line-height: 1.7;
}

.status-strip {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
  margin-top: 34px;
}

.status-strip span {
  border: 1px solid rgba(15, 23, 42, 0.12);
  background: rgba(255, 255, 255, 0.68);
  border-radius: 999px;
  padding: 10px 14px;
  font-size: 13px;
  font-weight: 700;
}

.auth-panel {
  display: flex;
  flex-direction: column;
  justify-content: center;
  padding: 32px;
  background: rgba(255, 255, 255, 0.72);
  border-left: 1px solid rgba(15, 23, 42, 0.08);
  backdrop-filter: blur(16px);
}

.mode-toggle {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  background: #e2e8f0;
  padding: 4px;
  border-radius: 14px;
  margin-bottom: 18px;
}

.mode-toggle button {
  border: 0;
  background: transparent;
  border-radius: 11px;
  padding: 12px;
  color: #475569;
  font-weight: 800;
  cursor: pointer;
}

.mode-toggle button.active {
  background: #ffffff;
  color: #0f172a;
  box-shadow: 0 12px 30px rgba(15, 23, 42, 0.08);
}

.auth-form {
  border: 1px solid rgba(15, 23, 42, 0.1);
  background: #ffffff;
  border-radius: 20px;
  padding: 28px;
  box-shadow: 0 30px 80px rgba(15, 23, 42, 0.12);
}

.form-copy {
  margin-bottom: 22px;
}

.form-copy h2 {
  font-family: 'Fraunces', serif;
  font-size: 30px;
  margin-bottom: 6px;
}

.form-copy p {
  color: #64748b;
  line-height: 1.5;
}

.notice {
  border-radius: 12px;
  padding: 12px 14px;
  margin-bottom: 16px;
  font-size: 14px;
  font-weight: 700;
}

.notice.success {
  background: #dcfce7;
  color: #166534;
}

.notice.error {
  background: #fee2e2;
  color: #991b1b;
}

.field {
  display: block;
  margin-bottom: 16px;
}

.field span {
  display: block;
  margin-bottom: 8px;
  font-size: 13px;
  color: #334155;
  font-weight: 800;
}

.field input {
  width: 100%;
  height: 48px;
  border: 1px solid #cbd5e1;
  border-radius: 12px;
  padding: 0 14px;
  font: inherit;
  color: #0f172a;
  background: #f8fafc;
}

.field input:focus {
  border-color: #f97316;
  box-shadow: 0 0 0 4px rgba(249, 115, 22, 0.15);
}

.submit-button {
  width: 100%;
  min-height: 50px;
  border: 0;
  border-radius: 999px;
  background: #0f172a;
  color: #ffffff;
  font-weight: 900;
  font-size: 15px;
  cursor: pointer;
  box-shadow: 0 16px 34px rgba(15, 23, 42, 0.2);
}

.submit-button:disabled {
  opacity: 0.65;
  cursor: not-allowed;
}

@media (max-width: 820px) {
  .auth-page {
    grid-template-columns: 1fr;
  }

  .auth-hero {
    padding-bottom: 8px;
  }

  .auth-panel {
    border-left: 0;
    padding-top: 12px;
  }
}
</style>
