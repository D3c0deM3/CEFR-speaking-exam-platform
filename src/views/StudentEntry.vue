<template>
  <div class="student-entry">
    <div class="entry-shell">
      <div class="entry-info">
        <span class="eyebrow">CEFR Speaking</span>
        <h1>IELTS Speaking Mock Test</h1>
        <p class="subtitle">Enter your full name to begin. The test runs automatically once you start.</p>
        <div class="instruction-list">
          <div class="instruction-item">
            <span class="bullet"></span>
            <span>Exam will start immediately</span>
          </div>
          <div class="instruction-item">
            <span class="bullet"></span>
            <span>Make sure your microphone is connected</span>
          </div>
          <div class="instruction-item">
            <span class="bullet"></span>
            <span>Do not refresh or close the window</span>
          </div>
        </div>
        <div class="admin-note">
          <small>Press Ctrl+Shift+A for admin access</small>
        </div>
      </div>

      <form @submit.prevent="startExam" class="entry-card">
        <div class="card-header">
          <h2>Student Check-In</h2>
          <p>Use your official name for scoring.</p>
        </div>
        <div class="input-group">
          <label for="name">Full Name</label>
          <input
            id="name"
            type="text"
            required
            v-model="studentName"
            placeholder="John Smith"
            autocomplete="off"
          />
        </div>

        <button
          type="submit"
          :disabled="!studentName.trim() || isLoading"
          class="start-button"
        >
          {{ isLoading ? 'Starting Exam...' : 'Start Exam' }}
        </button>
      </form>
    </div>
  </div>
</template>

<script setup>
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useExamStore } from '../stores/exam'

const studentName = ref('')
const router = useRouter()
const examStore = useExamStore()
const isLoading = ref(false)

async function startExam() {
  if (!studentName.value.trim()) return

  isLoading.value = true
  try {
    await examStore.startExam(studentName.value.trim())
    router.push('/exam')
  } catch (error) {
    alert('Failed to start exam. Please try again: ' + error)
    console.error(error)
  } finally {
    isLoading.value = false
  }
}
</script>

<style scoped>
.student-entry {
  min-height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  background:
    radial-gradient(circle at 10% 20%, rgba(249, 115, 22, 0.25), transparent 45%),
    radial-gradient(circle at 90% 15%, rgba(29, 78, 216, 0.18), transparent 40%),
    linear-gradient(135deg, #fef3c7 0%, #f8fafc 45%, #f1f5f9 100%);
  padding: 32px;
}

.entry-shell {
  width: min(1100px, 100%);
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
  gap: 32px;
  align-items: center;
}

.entry-info {
  padding: 20px;
}

.eyebrow {
  font-size: 12px;
  letter-spacing: 0.3em;
  text-transform: uppercase;
  color: var(--accent-deep);
  font-weight: 600;
}

.entry-info h1 {
  font-family: 'Fraunces', serif;
  font-size: 42px;
  margin: 12px 0 16px;
  color: var(--ink);
}

.subtitle {
  font-size: 16px;
  color: var(--muted);
  max-width: 420px;
  line-height: 1.6;
  margin-bottom: 24px;
}

.instruction-list {
  display: grid;
  gap: 12px;
}

.instruction-item {
  display: flex;
  align-items: center;
  gap: 12px;
  font-size: 14px;
  color: #334155;
}

.bullet {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: var(--accent);
  box-shadow: 0 0 0 4px rgba(249, 115, 22, 0.2);
}

.admin-note {
  margin-top: 24px;
  padding-top: 16px;
  border-top: 1px dashed rgba(100, 116, 139, 0.4);
  color: #94a3b8;
}

.entry-card {
  background: var(--surface);
  border-radius: 24px;
  padding: 36px;
  box-shadow: 0 30px 90px rgba(15, 23, 42, 0.15);
  border: 1px solid rgba(148, 163, 184, 0.3);
  animation: floatIn 0.6s ease;
}

@keyframes floatIn {
  from { opacity: 0; transform: translateY(12px); }
  to { opacity: 1; transform: translateY(0); }
}

.card-header h2 {
  font-family: 'Fraunces', serif;
  font-size: 24px;
  margin-bottom: 6px;
  color: var(--ink);
}

.card-header p {
  font-size: 14px;
  color: var(--muted);
  margin-bottom: 24px;
}

.input-group {
  margin-bottom: 24px;
}

.input-group label {
  display: block;
  font-size: 13px;
  font-weight: 600;
  color: #475569;
  margin-bottom: 8px;
}

.input-group input {
  width: 100%;
  padding: 14px 16px;
  border: 2px solid rgba(148, 163, 184, 0.35);
  border-radius: 12px;
  font-size: 16px;
  font-family: 'Space Grotesk', sans-serif;
  transition: border-color 0.2s, box-shadow 0.2s;
}

.input-group input:focus {
  border-color: var(--accent);
  box-shadow: 0 0 0 3px rgba(249, 115, 22, 0.2);
}

.start-button {
  width: 100%;
  padding: 16px;
  background: var(--accent-deep);
  color: white;
  border: none;
  border-radius: 14px;
  font-size: 16px;
  font-weight: 600;
  cursor: pointer;
  transition: transform 0.2s ease, background 0.2s ease;
}

.start-button:hover:not(:disabled) {
  background: #1e40af;
  transform: translateY(-2px);
}

.start-button:disabled {
  background: #94a3b8;
  cursor: not-allowed;
}

@media (max-width: 720px) {
  .student-entry {
    padding: 20px;
  }

  .entry-info h1 {
    font-size: 32px;
  }
}
</style>
