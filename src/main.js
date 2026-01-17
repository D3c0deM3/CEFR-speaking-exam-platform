import { createApp } from 'vue'
import { createPinia } from 'pinia'
import { createRouter, createMemoryHistory } from 'vue-router'
import App from './App.vue'
import { invoke } from '@tauri-apps/api/core'

// Make invoke available globally for debugging
window.invoke = invoke

// Import views
import StudentEntry from './views/StudentEntry.vue'
import Exam from './views/Exam.vue'
import ExamComplete from './views/ExamComplete.vue'
import AdminDashboard from './views/AdminDashboard.vue'

// Create Pinia store
const pinia = createPinia()

// Create router
const router = createRouter({
  history: createMemoryHistory(),
  routes: [
    { path: '/', component: StudentEntry },
    { path: '/exam', component: Exam },
    { path: '/complete', component: ExamComplete },
    { path: '/admin', component: AdminDashboard }
  ]
})

// Admin hotkey
let hotkeyListener = null

router.beforeEach((to, from, next) => {
  // Remove previous listener if exists
  if (hotkeyListener) {
    window.removeEventListener('keydown', hotkeyListener)
  }
  
  // Add new listener for admin access
  hotkeyListener = (e) => {
    if (e.ctrlKey && e.shiftKey && e.key === 'A') {
      e.preventDefault()
      const password = prompt('Enter admin password:')
      if (password === 'admin123') { // Change this in production!
        router.push('/admin')
      }
    }
  }
  
  window.addEventListener('keydown', hotkeyListener)
  next()
})

createApp(App)
  .use(pinia)
  .use(router)
  .mount('#app')