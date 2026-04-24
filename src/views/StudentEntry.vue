<template>
  <div class="student-entry">
    <div class="entry-shell">
      <div class="entry-info">
        <span class="eyebrow">CEFR Speaking</span>
        <h1>CEFR Speaking Mock Test</h1>
        <p class="subtitle">Enter your full name, then start a random mock test or choose a saved full test.</p>
        <div class="instruction-list">
          <div class="instruction-item">
            <span class="bullet"></span>
            <span>Random mode keeps the current automatic test flow</span>
          </div>
          <div class="instruction-item">
            <span class="bullet"></span>
            <span>Choose Test mode starts one complete admin-created test</span>
          </div>
          <div class="instruction-item">
            <span class="bullet"></span>
            <span>Make sure your microphone is connected before starting</span>
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

        <div class="mode-toggle" role="tablist" aria-label="Exam start mode">
          <button
            type="button"
            class="mode-option"
            :class="{ active: examMode === 'random' }"
            @click="examMode = 'random'"
          >
            Random Test
          </button>
          <button
            type="button"
            class="mode-option"
            :class="{ active: examMode === 'selected' }"
            @click="examMode = 'selected'"
          >
            Choose Test
          </button>
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

        <div v-if="examMode === 'selected'" class="question-picker">
          <div class="picker-header">
            <div>
              <span class="picker-eyebrow">Saved Tests</span>
              <h3>Choose a full test</h3>
            </div>
            <span class="selection-count">{{ fullTests.length }} available</span>
          </div>

          <div v-if="isLoadingTests" class="picker-message">
            Loading tests...
          </div>
          <div v-else-if="testLoadError" class="picker-message error">
            {{ testLoadError }}
          </div>
          <div v-else-if="fullTests.length === 0" class="picker-message">
            No full tests found. Create a full test in the admin dashboard first.
          </div>
          <div v-else class="test-list">
            <label
              v-for="test in fullTests"
              :key="test.id"
              class="test-option"
              :class="{ selected: selectedFullTestId === test.id }"
            >
              <input
                type="radio"
                name="full-test"
                :value="test.id"
                v-model="selectedFullTestId"
              />
              <div class="test-copy">
                <strong>{{ test.name }}</strong>
                <span>{{ test.questions.length }} question(s)</span>
                <small>
                  <span
                    v-for="section in sectionOrder"
                    :key="section.key"
                  >
                    {{ section.label }}: {{ getTestSectionCount(test, section) }}
                  </span>
                </small>
              </div>
            </label>
          </div>
        </div>

        <button
          type="submit"
          :disabled="isStartDisabled"
          class="start-button"
        >
          {{ startButtonLabel }}
        </button>
      </form>
    </div>
  </div>
</template>

<script setup>
import { computed, onMounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import { useExamStore } from '../stores/exam'

const studentName = ref('')
const router = useRouter()
const examStore = useExamStore()
const isLoading = ref(false)
const examMode = ref('random')
const fullTests = ref([])
const selectedFullTestId = ref(null)
const isLoadingTests = ref(false)
const testLoadError = ref('')

const sectionOrder = [
  { key: '1-1', label: 'Part 1.1', part: 1, subPart: 1 },
  { key: '1-2', label: 'Part 1.2', part: 1, subPart: 2 },
  { key: '2-0', label: 'Part 2', part: 2, subPart: 0 },
  { key: '3-0', label: 'Part 3', part: 3, subPart: 0 }
]

const selectedFullTest = computed(() => {
  return fullTests.value.find(test => test.id === selectedFullTestId.value) || null
})

const isStartDisabled = computed(() => {
  if (!studentName.value.trim() || isLoading.value) return true
  if (examMode.value === 'selected') {
    return isLoadingTests.value || !selectedFullTest.value
  }
  return false
})

const startButtonLabel = computed(() => {
  if (isLoading.value) return 'Starting Exam...'
  if (examMode.value === 'selected') {
    return selectedFullTest.value
      ? `Start ${selectedFullTest.value.name}`
      : 'Start Selected Test'
  }
  return 'Start Random Test'
})

onMounted(() => {
  loadFullTests()
})

watch(examMode, (value) => {
  if (value === 'selected' && fullTests.value.length === 0 && !isLoadingTests.value) {
    loadFullTests()
  }
})

async function loadFullTests() {
  isLoadingTests.value = true
  testLoadError.value = ''

  try {
    const data = await invoke('get_full_tests')
    fullTests.value = (data || []).filter(isCompleteFullTest)
    if (!fullTests.value.some(test => test.id === selectedFullTestId.value)) {
      selectedFullTestId.value = fullTests.value[0]?.id || null
    }
  } catch (error) {
    console.error('Failed to load full tests:', error)
    testLoadError.value = 'Could not load tests. Please try again or use random mode.'
  } finally {
    isLoadingTests.value = false
  }
}

async function startExam() {
  if (!studentName.value.trim()) return
  if (examMode.value === 'selected' && !selectedFullTest.value) {
    alert('Choose a full test before starting selected mode.')
    return
  }

  isLoading.value = true
  try {
    if (examMode.value === 'selected') {
      await examStore.startSelectedExam(studentName.value.trim(), selectedFullTest.value.questions)
    } else {
      await examStore.startExam(studentName.value.trim())
    }
    router.push('/exam')
  } catch (error) {
    alert('Failed to start exam. Please try again: ' + error)
    console.error(error)
  } finally {
    isLoading.value = false
  }
}

function getTestSectionCount(test, section) {
  return (test.questions || []).filter((question) => (
    question.part === section.part && question.sub_part === section.subPart
  )).length
}

function isCompleteFullTest(test) {
  return sectionOrder.every(section => getTestSectionCount(test, section) > 0)
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

.mode-toggle {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 8px;
  padding: 6px;
  border-radius: 16px;
  background: #f1f5f9;
  border: 1px solid rgba(148, 163, 184, 0.28);
  margin-bottom: 24px;
}

.mode-option {
  border: none;
  border-radius: 12px;
  padding: 12px 10px;
  background: transparent;
  color: #64748b;
  font-family: 'Space Grotesk', sans-serif;
  font-size: 14px;
  font-weight: 700;
  cursor: pointer;
  transition: background 0.2s ease, color 0.2s ease, box-shadow 0.2s ease;
}

.mode-option.active {
  background: #ffffff;
  color: var(--accent-deep);
  box-shadow: 0 10px 28px rgba(15, 23, 42, 0.12);
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

.question-picker {
  margin: 0 0 24px;
  padding: 18px;
  border-radius: 20px;
  background:
    radial-gradient(circle at 10% 0%, rgba(29, 78, 216, 0.08), transparent 42%),
    #f8fafc;
  border: 1px solid rgba(148, 163, 184, 0.28);
}

.picker-header,
.group-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 14px;
}

.picker-header {
  margin-bottom: 16px;
}

.picker-eyebrow {
  display: block;
  margin-bottom: 4px;
  color: var(--accent);
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.18em;
  text-transform: uppercase;
}

.picker-header h3 {
  font-family: 'Fraunces', serif;
  font-size: 20px;
  color: var(--ink);
}

.selection-count {
  flex: 0 0 auto;
  padding: 7px 10px;
  border-radius: 999px;
  background: #dbeafe;
  color: #1d4ed8;
  font-size: 12px;
  font-weight: 700;
}

.picker-message {
  padding: 16px;
  border-radius: 14px;
  background: #ffffff;
  color: #64748b;
  font-size: 14px;
  font-weight: 600;
  border: 1px dashed rgba(100, 116, 139, 0.35);
}

.picker-message.error {
  background: #fef2f2;
  color: #b91c1c;
  border-color: rgba(248, 113, 113, 0.45);
}

.picker-groups {
  display: grid;
  gap: 14px;
  max-height: min(44vh, 420px);
  overflow-y: auto;
  padding-right: 4px;
}

.test-list {
  display: grid;
  gap: 12px;
  max-height: min(44vh, 420px);
  overflow-y: auto;
  padding-right: 4px;
}

.test-option {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 14px;
  border-radius: 14px;
  background: #ffffff;
  border: 1px solid rgba(148, 163, 184, 0.28);
  cursor: pointer;
  transition: border-color 0.2s ease, background 0.2s ease, transform 0.2s ease;
}

.test-option.selected {
  background: #eff6ff;
  border-color: rgba(29, 78, 216, 0.38);
}

.test-option:hover {
  transform: translateY(-1px);
  border-color: rgba(249, 115, 22, 0.45);
}

.test-option input {
  margin-top: 4px;
  accent-color: var(--accent-deep);
}

.test-copy {
  min-width: 0;
  display: grid;
  gap: 5px;
}

.test-copy strong {
  color: #0f172a;
  font-size: 15px;
}

.test-copy span {
  color: #334155;
  font-size: 13px;
  font-weight: 700;
}

.test-copy small {
  display: flex;
  flex-wrap: wrap;
  gap: 6px 10px;
  color: #64748b;
  font-size: 12px;
  font-weight: 600;
}

.picker-group {
  display: grid;
  gap: 10px;
  padding: 14px;
  border-radius: 16px;
  background: rgba(255, 255, 255, 0.86);
  border: 1px solid rgba(148, 163, 184, 0.24);
}

.group-header h4 {
  font-size: 14px;
  color: #0f172a;
}

.mini-action {
  border: none;
  border-radius: 999px;
  padding: 6px 10px;
  background: #ffedd5;
  color: #c2410c;
  font-family: 'Space Grotesk', sans-serif;
  font-size: 12px;
  font-weight: 700;
  cursor: pointer;
}

.question-option {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  padding: 12px;
  border-radius: 14px;
  background: #f8fafc;
  border: 1px solid rgba(148, 163, 184, 0.24);
  cursor: pointer;
  transition: border-color 0.2s ease, background 0.2s ease, transform 0.2s ease;
}

.question-option.selected {
  background: #eff6ff;
  border-color: rgba(29, 78, 216, 0.35);
}

.question-option:hover {
  transform: translateY(-1px);
  border-color: rgba(249, 115, 22, 0.45);
}

.question-option input {
  margin-top: 3px;
  accent-color: var(--accent-deep);
}

.question-copy {
  min-width: 0;
  display: grid;
  gap: 3px;
}

.question-copy strong {
  color: #0f172a;
  font-size: 13px;
}

.question-copy span {
  color: #334155;
  font-size: 13px;
  line-height: 1.4;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.question-copy small {
  color: #64748b;
  font-size: 12px;
  font-weight: 600;
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

  .entry-card {
    padding: 28px;
  }

  .picker-header,
  .group-header {
    align-items: flex-start;
    flex-direction: column;
  }
}
</style>
