<template>
  <div class="admin-dashboard">
    <div class="header">
      <div>
        <p class="eyebrow">Admin Control</p>
        <h1>Dashboard</h1>
      </div>
      <router-link to="/" class="logout-btn"><- Return Home</router-link>
    </div>

    <div class="tabs">
      <button
        v-for="tab in tabs"
        :key="tab"
        :class="['tab-btn', { active: activeTab === tab }]"
        @click="activeTab = tab"
      >
        {{ tab }}
      </button>
    </div>

    <!-- Attempts Tab -->
    <div v-if="activeTab === 'Attempts'" class="tab-content">
      <div class="section-header">
        <h2>Student Attempts</h2>
        <div class="search-bar">
          <input
            v-model="searchQuery"
            type="text"
            placeholder="Search student name..."
            class="search-input"
          />
        </div>
      </div>
      <div v-if="filteredAttempts.length === 0" class="empty-state">
        <p>No attempts found</p>
      </div>
      <div v-else class="table-wrapper">
        <table class="attempts-table">
          <thead>
            <tr>
              <th>Student Name</th>
              <th>Started At</th>
              <th>Finished At</th>
              <th>Status</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="attempt in filteredAttempts" :key="attempt.id">
              <td>{{ attempt.student_name }}</td>
              <td>{{ formatDate(attempt.started_at) }}</td>
              <td>{{ attempt.finished_at ? formatDate(attempt.finished_at) : 'In Progress' }}</td>
              <td>
                <span :class="['status-badge', attempt.finished_at ? 'completed' : 'pending']">
                  {{ attempt.finished_at ? 'Completed' : 'Pending' }}
                </span>
              </td>
              <td>
                <button class="action-btn" @click="viewAttemptDetails(attempt)">View Details</button>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>

    <!-- Questions Tab -->
    <div v-if="activeTab === 'Questions'" class="tab-content">
      <div class="section">
        <h2>Manage Questions</h2>
        <div class="form-grid">
          <div class="form-group">
            <label>Part</label>
            <select v-model="newQuestion.part" class="form-input">
              <option value="">Select Part</option>
              <option value="1.1">Part 1.1</option>
              <option value="1.2">Part 1.2</option>
              <option value="2">Part 2</option>
              <option value="3">Part 3</option>
            </select>
          </div>
          <div class="form-group">
            <label>Response Time (seconds)</label>
            <input v-model.number="newQuestion.response_time" type="number" class="form-input" />
          </div>
          <div class="form-group form-span">
            <label>Question Text</label>
            <textarea
              v-model="newQuestion.text"
              class="form-input"
              rows="3"
              placeholder="Enter the question text shown to students"
            ></textarea>
          </div>
          <div class="form-group">
            <label>Audio File (Optional)</label>
            <div class="file-input-wrapper">
              <input
                ref="audioInput"
                type="file"
                accept="audio/*"
                @change="onAudioSelected"
                class="file-input"
              />
              <label class="file-label" @click="$refs.audioInput?.click()">
                {{ newQuestion.audioFileName ? `Selected: ${newQuestion.audioFileName}` : 'Choose Audio File' }}
              </label>
            </div>
          </div>
          <div class="form-group" v-if="newQuestion.part === '1.2'">
            <label>Image File (Required for Part 1.2)</label>
            <div class="file-input-wrapper">
              <input
                ref="imageInput"
                type="file"
                accept="image/*"
                @change="onImageSelected"
                class="file-input"
              />
              <label class="file-label" @click="$refs.imageInput?.click()">
                {{ newQuestion.imageFileName ? `Selected: ${newQuestion.imageFileName}` : 'Choose Image File' }}
              </label>
            </div>
          </div>
        </div>
        <button @click="addQuestion" class="add-btn" :disabled="addingQuestion">
          {{ addingQuestion ? 'Adding...' : 'Add Question' }}
        </button>
      </div>

      <div class="section">
        <h3>Existing Questions</h3>
        <div v-if="questions.length === 0" class="empty-state">
          <p>No questions added yet</p>
        </div>
        <div v-else class="questions-list">
          <div v-for="question in questions" :key="question.id" class="question-card">
            <div class="question-info">
              <strong>{{ formatPart(question.part, question.sub_part) }}</strong>
              <span v-if="question.text" class="question-text">{{ question.text }}</span>
              <span>{{ question.response_time }}s</span>
              <span v-if="question.audio_path" class="tag">Audio</span>
              <span v-if="question.image_path" class="tag">Image</span>
            </div>
            <button
              @click="deleteQuestion(question.id)"
              class="delete-btn"
              title="Delete this question"
            >
              Delete
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Recordings Tab -->
    <div v-if="activeTab === 'Recordings'" class="tab-content">
      <h2>Student Recordings</h2>
      <div v-if="recordingsByAttempt.length === 0" class="empty-state">
        <p>No recordings available yet</p>
      </div>
      <div v-else class="recordings-list">
        <div v-for="attempt in recordingsByAttempt" :key="attempt.attemptId" class="recording-group">
          <div class="group-header">
            <div>
              <h3>{{ attempt.studentName }}</h3>
              <p>{{ formatDate(attempt.attemptStartedAt) }}</p>
            </div>
          </div>
          <div class="recording-items">
            <div v-for="recording in attempt.items" :key="recording.id" class="recording-item">
              <div class="recording-info">
                <strong>{{ formatPart(recording.part, recording.sub_part) }}</strong>
                <span>Question {{ recording.question_id }}</span>
                <span v-if="recording.question_text" class="recording-question">
                  {{ recording.question_text }}
                </span>
                <span>{{ recording.duration }}s</span>
                <span>{{ formatDate(recording.recorded_at) }}</span>
              </div>
              <div class="recording-player">
                <button
                  v-if="!recording.audioUrl"
                  class="load-btn"
                  :disabled="recording.isLoading"
                  @click="loadRecordingAudio(recording)"
                >
                  {{ recording.isLoading ? 'Loading...' : 'Load Audio' }}
                </button>
                <audio v-else controls :src="recording.audioUrl"></audio>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Statistics Tab -->
    <div v-if="activeTab === 'Statistics'" class="tab-content">
      <h2>Test Statistics</h2>
      <div class="stats-grid">
        <div class="stat-card">
          <h3>Total Attempts</h3>
          <p class="stat-value">{{ totalAttempts }}</p>
        </div>
        <div class="stat-card">
          <h3>Completed</h3>
          <p class="stat-value">{{ completedAttempts }}</p>
        </div>
        <div class="stat-card">
          <h3>In Progress</h3>
          <p class="stat-value">{{ inProgressAttempts }}</p>
        </div>
        <div class="stat-card">
          <h3>Total Questions</h3>
          <p class="stat-value">{{ questions.length }}</p>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const activeTab = ref('Attempts')
const tabs = ['Attempts', 'Questions', 'Recordings', 'Statistics']
const searchQuery = ref('')
const attempts = ref([])
const questions = ref([])
const recordings = ref([])
const addingQuestion = ref(false)
const audioInput = ref(null)
const imageInput = ref(null)

const newQuestion = ref({
  part: '',
  response_time: 30,
  text: '',
  audioFileName: '',
  audioData: null,
  imageFileName: '',
  imageData: null
})

const filteredAttempts = computed(() => {
  return attempts.value.filter(attempt =>
    attempt.student_name.toLowerCase().includes(searchQuery.value.toLowerCase())
  )
})

const recordingsByAttempt = computed(() => {
  const groups = new Map()
  recordings.value.forEach((recording) => {
    if (!groups.has(recording.attempt_id)) {
      groups.set(recording.attempt_id, {
        attemptId: recording.attempt_id,
        studentName: recording.student_name,
        attemptStartedAt: recording.attempt_started_at,
        items: []
      })
    }
    groups.get(recording.attempt_id).items.push(recording)
  })
  return Array.from(groups.values())
})

const totalAttempts = computed(() => attempts.value.length)
const completedAttempts = computed(() => attempts.value.filter(a => a.finished_at).length)
const inProgressAttempts = computed(() => attempts.value.filter(a => !a.finished_at).length)

function onAudioSelected(event) {
  const file = event.target.files?.[0]
  if (file) {
    newQuestion.value.audioFileName = file.name
    newQuestion.value.audioData = file
  }
}

function onImageSelected(event) {
  const file = event.target.files?.[0]
  if (file) {
    newQuestion.value.imageFileName = file.name
    newQuestion.value.imageData = file
  }
}

async function loadAttempts() {
  try {
    const data = await invoke('get_attempts')
    attempts.value = data || []
  } catch (error) {
    console.error('Failed to load attempts:', error)
  }
}

async function loadQuestions() {
  try {
    const data = await invoke('get_questions')
    questions.value = data || []
  } catch (error) {
    console.error('Failed to load questions:', error)
  }
}

async function loadRecordings() {
  try {
    revokeRecordingUrls()
    const data = await invoke('get_recordings')
    recordings.value = (data || []).map((recording) => ({
      ...recording,
      audioUrl: '',
      isLoading: false
    }))
  } catch (error) {
    console.error('Failed to load recordings:', error)
  }
}

async function addQuestion() {
  if (!newQuestion.value.part) {
    alert('Please select a part')
    return
  }

  if (newQuestion.value.part === '1.2' && !newQuestion.value.imageData) {
    alert('Please upload an image for Part 1.2')
    return
  }

  addingQuestion.value = true
  try {
    const { part, subPart } = parsePartSelection(newQuestion.value.part)
    let audioPath = null
    let imagePath = null

    if (newQuestion.value.audioData) {
      const filename = `part${part}_${subPart || 0}_${Date.now()}.wav`

      const arrayBuffer = await newQuestion.value.audioData.arrayBuffer()
      const audioBytes = Array.from(new Uint8Array(arrayBuffer))

      const savedFilename = await invoke('save_audio_file', {
        filename: filename,
        audioData: audioBytes
      })

      audioPath = savedFilename
    }

    if (newQuestion.value.imageData) {
      const extension = getFileExtension(newQuestion.value.imageData.name)
      const filename = `part${part}_${subPart || 0}_${Date.now()}.${extension}`
      const arrayBuffer = await newQuestion.value.imageData.arrayBuffer()
      const imageBytes = Array.from(new Uint8Array(arrayBuffer))

      const savedFilename = await invoke('save_image_file', {
        filename: filename,
        imageData: imageBytes
      })

      imagePath = savedFilename
    }

    await invoke('add_question', {
      part: part,
      subPart: subPart || undefined,
      responseTime: newQuestion.value.response_time,
      audioPath: audioPath,
      imagePath: imagePath,
      text: newQuestion.value.text?.trim() || ''
    })

    resetQuestionForm()
    await loadQuestions()
    alert('Question added successfully!')
  } catch (error) {
    console.error('Failed to add question:', error)
    alert('Error adding question: ' + (error?.message || error))
  } finally {
    addingQuestion.value = false
  }
}

async function loadRecordingAudio(recording) {
  if (recording.audioUrl || recording.isLoading) return
  recording.isLoading = true
  try {
    const audioData = await invoke('get_response_audio', {
      responseId: recording.id
    })
    const audioArray = new Uint8Array(audioData)
    const audioBlob = new Blob([audioArray], { type: 'audio/webm' })
    recording.audioUrl = URL.createObjectURL(audioBlob)
  } catch (error) {
    console.error('Failed to load recording audio:', error)
  } finally {
    recording.isLoading = false
  }
}

function revokeRecordingUrls() {
  recordings.value.forEach((recording) => {
    if (recording.audioUrl) {
      URL.revokeObjectURL(recording.audioUrl)
    }
  })
}

function resetQuestionForm() {
  newQuestion.value = {
    part: '',
    response_time: 30,
    text: '',
    audioFileName: '',
    audioData: null,
    imageFileName: '',
    imageData: null
  }
  if (audioInput.value) {
    audioInput.value.value = ''
  }
  if (imageInput.value) {
    imageInput.value.value = ''
  }
}

function parsePartSelection(partValue) {
  if (partValue === '1.1') return { part: 1, subPart: 1 }
  if (partValue === '1.2') return { part: 1, subPart: 2 }
  return { part: Number(partValue), subPart: 0 }
}

function getFileExtension(filename) {
  const segments = filename.split('.')
  return segments.length > 1 ? segments.pop().toLowerCase() : 'png'
}

function formatPart(part, subPart) {
  if (part === 1 && subPart === 1) return 'Part 1.1'
  if (part === 1 && subPart === 2) return 'Part 1.2'
  return `Part ${part}`
}

function formatDate(dateString) {
  const date = new Date(dateString)
  return date.toLocaleString()
}

function viewAttemptDetails(attempt) {
  alert(`Viewing details for ${attempt.student_name}\n\nStarted: ${formatDate(attempt.started_at)}\nFinished: ${attempt.finished_at ? formatDate(attempt.finished_at) : 'Not finished'}`)
}

async function deleteQuestion(questionId) {
  if (confirm('Are you sure you want to delete this question? This action cannot be undone.')) {
    try {
      await invoke('delete_question', {
        questionId: questionId
      })
      await loadQuestions()
      alert('Question deleted successfully!')
    } catch (error) {
      console.error('Failed to delete question:', error)
      alert('Error deleting question: ' + (error?.message || String(error)))
    }
  }
}

onMounted(() => {
  loadAttempts()
  loadQuestions()
  loadRecordings()
})

onUnmounted(() => {
  revokeRecordingUrls()
})

watch(() => newQuestion.value.part, (value) => {
  if (value !== '1.2') {
    newQuestion.value.imageFileName = ''
    newQuestion.value.imageData = null
    if (imageInput.value) {
      imageInput.value.value = ''
    }
  }
})
</script>

<style scoped>
.admin-dashboard {
  min-height: 100vh;
  background: linear-gradient(135deg, #f8fafc 0%, #f1f5f9 50%, #e2e8f0 100%);
  padding: 32px;
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
  background: rgba(255, 255, 255, 0.9);
  padding: 24px 28px;
  border-radius: 18px;
  box-shadow: 0 24px 70px rgba(15, 23, 42, 0.12);
  border: 1px solid rgba(148, 163, 184, 0.25);
}

.eyebrow {
  font-size: 12px;
  text-transform: uppercase;
  letter-spacing: 0.3em;
  color: #64748b;
  margin-bottom: 6px;
}

.header h1 {
  margin: 0;
  font-family: 'Fraunces', serif;
  font-size: 30px;
  color: #0f172a;
}

.logout-btn {
  background: #ef4444;
  color: white;
  padding: 0.75rem 1.5rem;
  border-radius: 12px;
  text-decoration: none;
  font-weight: 600;
  transition: transform 0.2s ease, background 0.2s ease;
}

.logout-btn:hover {
  background: #dc2626;
  transform: translateY(-2px);
}

.tabs {
  display: flex;
  flex-wrap: wrap;
  gap: 0.75rem;
  margin-bottom: 24px;
}

.tab-btn {
  padding: 0.6rem 1.2rem;
  background: #e2e8f0;
  border: none;
  border-radius: 999px;
  cursor: pointer;
  font-weight: 600;
  color: #475569;
  transition: all 0.2s ease;
}

.tab-btn.active {
  background: #1d4ed8;
  color: white;
}

.tab-btn:hover {
  background: #2563eb;
  color: white;
}

.tab-content {
  background: rgba(255, 255, 255, 0.95);
  padding: 28px;
  border-radius: 20px;
  box-shadow: 0 24px 60px rgba(15, 23, 42, 0.08);
  border: 1px solid rgba(148, 163, 184, 0.25);
}

.section-header {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  margin-bottom: 16px;
}

.search-input {
  width: 100%;
  max-width: 360px;
  padding: 0.75rem;
  border: 1px solid rgba(148, 163, 184, 0.4);
  border-radius: 12px;
  font-size: 0.95rem;
}

.table-wrapper {
  overflow-x: auto;
}

.attempts-table {
  width: 100%;
  border-collapse: collapse;
  margin-top: 1rem;
  min-width: 640px;
}

.attempts-table thead {
  background: #f1f5f9;
}

.attempts-table th,
.attempts-table td {
  padding: 1rem;
  text-align: left;
  border-bottom: 1px solid #e2e8f0;
}

.attempts-table th {
  font-weight: 600;
  color: #334155;
}

.status-badge {
  padding: 0.25rem 0.75rem;
  border-radius: 9999px;
  font-size: 0.875rem;
  font-weight: 600;
}

.status-badge.completed {
  background: #dcfce7;
  color: #166534;
}

.status-badge.pending {
  background: #fef9c3;
  color: #a16207;
}

.action-btn {
  background: #1d4ed8;
  color: white;
  padding: 0.5rem 1rem;
  border: none;
  border-radius: 8px;
  cursor: pointer;
  font-size: 0.875rem;
  transition: transform 0.2s ease, background 0.2s ease;
}

.action-btn:hover {
  background: #1e40af;
  transform: translateY(-1px);
}

.empty-state {
  text-align: center;
  padding: 2rem;
  color: #64748b;
}

.section {
  margin-bottom: 2rem;
  padding-bottom: 2rem;
  border-bottom: 1px solid rgba(148, 163, 184, 0.3);
}

.form-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
  gap: 16px;
  margin: 20px 0;
}

.form-span {
  grid-column: 1 / -1;
}

.form-group label {
  display: block;
  margin-bottom: 0.5rem;
  font-weight: 600;
  color: #475569;
}

.form-input {
  width: 100%;
  padding: 0.75rem;
  border: 1px solid rgba(148, 163, 184, 0.4);
  border-radius: 10px;
  font-size: 0.95rem;
  font-family: 'Space Grotesk', sans-serif;
}

.add-btn {
  background: #16a34a;
  color: white;
  padding: 0.75rem 1.5rem;
  border: none;
  border-radius: 10px;
  cursor: pointer;
  font-weight: 600;
  transition: transform 0.2s ease, background 0.2s ease;
}

.add-btn:hover:not(:disabled) {
  background: #15803d;
  transform: translateY(-1px);
}

.add-btn:disabled {
  background: #94a3b8;
  cursor: not-allowed;
}

.file-input-wrapper {
  position: relative;
  display: inline-block;
  width: 100%;
}

.file-input {
  display: none;
}

.file-label {
  display: block;
  padding: 0.75rem;
  background: #f1f5f9;
  border: 2px dashed rgba(148, 163, 184, 0.5);
  border-radius: 10px;
  cursor: pointer;
  text-align: center;
  font-weight: 600;
  color: #475569;
  transition: all 0.2s ease;
}

.file-label:hover {
  border-color: #1d4ed8;
  color: #1d4ed8;
}

.questions-list {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
  gap: 1rem;
}

.question-card {
  background: #f8fafc;
  padding: 1rem;
  border-radius: 12px;
  border: 1px solid rgba(148, 163, 184, 0.3);
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 1rem;
}

.question-info {
  color: #1f2937;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.question-text {
  color: #475569;
  font-size: 0.85rem;
  line-height: 1.4;
}

.tag {
  display: inline-flex;
  padding: 2px 8px;
  border-radius: 999px;
  background: #e2e8f0;
  color: #475569;
  font-size: 12px;
  font-weight: 600;
  width: fit-content;
}

.delete-btn {
  background: #ef4444;
  color: white;
  padding: 0.5rem 0.75rem;
  border: none;
  border-radius: 8px;
  cursor: pointer;
  font-size: 0.85rem;
  white-space: nowrap;
}

.delete-btn:hover {
  background: #dc2626;
}

.recordings-list {
  display: grid;
  gap: 20px;
  margin-top: 16px;
}

.recording-group {
  border: 1px solid rgba(148, 163, 184, 0.3);
  border-radius: 16px;
  padding: 18px;
  background: #f8fafc;
}

.group-header h3 {
  font-family: 'Fraunces', serif;
  margin: 0 0 4px;
  font-size: 20px;
}

.group-header p {
  margin: 0;
  color: #64748b;
  font-size: 13px;
}

.recording-items {
  display: grid;
  gap: 12px;
  margin-top: 16px;
}

.recording-item {
  display: flex;
  justify-content: space-between;
  flex-wrap: wrap;
  gap: 12px;
  padding: 12px;
  border-radius: 12px;
  background: white;
  border: 1px solid rgba(148, 163, 184, 0.25);
}

.recording-info {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
  align-items: center;
  color: #334155;
  font-size: 13px;
}

.recording-question {
  color: #475569;
  font-weight: 600;
  max-width: 100%;
}

.recording-player {
  display: flex;
  align-items: center;
  gap: 10px;
}

.load-btn {
  background: #0f172a;
  color: white;
  border: none;
  padding: 0.5rem 1rem;
  border-radius: 10px;
  cursor: pointer;
  font-weight: 600;
  transition: background 0.2s ease;
}

.load-btn:hover:not(:disabled) {
  background: #1f2937;
}

.load-btn:disabled {
  background: #94a3b8;
  cursor: not-allowed;
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 1.5rem;
  margin-top: 1.5rem;
}

.stat-card {
  background: linear-gradient(135deg, #0f172a 0%, #1d4ed8 100%);
  color: white;
  padding: 1.5rem;
  border-radius: 16px;
  text-align: center;
}

.stat-card h3 {
  margin: 0 0 0.5rem 0;
  font-size: 0.9rem;
  opacity: 0.8;
}

.stat-value {
  margin: 0;
  font-size: 2.5rem;
  font-weight: 700;
}

@media (max-width: 720px) {
  .admin-dashboard {
    padding: 20px;
  }

  .header {
    flex-direction: column;
    align-items: flex-start;
    gap: 16px;
  }
}
</style>
