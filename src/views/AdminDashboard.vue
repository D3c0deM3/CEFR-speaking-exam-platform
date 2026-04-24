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

    <div class="telegram-settings-card">
      <div class="telegram-settings-header">
        <h2>Telegram Receiver</h2>
        <p>Add Telegram chat IDs. Every new recording will be sent to all IDs in this list.</p>
      </div>
      <div class="telegram-settings-controls">
        <input
          v-model.trim="telegramChatIdInput"
          type="text"
          class="form-input"
          placeholder="e.g. 764168975 or -1001234567890"
          @keyup.enter="addTelegramChatId"
        />
        <button
          class="icon-btn add-telegram-btn"
          :disabled="savingTelegramChatIds"
          title="Add chat ID"
          aria-label="Add chat ID"
          @click="addTelegramChatId"
        >
          +
        </button>
      </div>
      <div v-if="telegramChatIds.length > 0" class="telegram-chat-list">
        <div
          v-for="chatId in telegramChatIds"
          :key="chatId"
          class="telegram-chat-chip"
        >
          <span>{{ chatId }}</span>
          <button
            class="icon-btn delete-telegram-btn"
            :disabled="savingTelegramChatIds"
            title="Remove chat ID"
            aria-label="Remove chat ID"
            @click="removeTelegramChatId(chatId)"
          >
            x
          </button>
        </div>
      </div>
      <p v-else class="telegram-empty">No chat IDs added yet.</p>
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
                <div class="attempt-actions">
                  <button class="action-btn" @click="viewAttemptDetails(attempt)">View Details</button>
                  <button
                    class="danger-btn"
                    :disabled="attempt.isDeleting"
                    @click="deleteAttempt(attempt)"
                  >
                    {{ attempt.isDeleting ? 'Deleting...' : 'Delete' }}
                  </button>
                </div>
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
          <div class="form-group" v-if="needsPack">
            <label>Test Pack ID</label>
            <input
              v-model.trim="newQuestion.packId"
              type="text"
              class="form-input"
              placeholder="e.g. Test 1"
            />
          </div>
          <div class="form-group" v-if="needsPack">
            <label>Order in Pack</label>
            <input
              v-model.number="newQuestion.packOrder"
              type="number"
              min="1"
              class="form-input"
            />
          </div>
          <div class="form-group form-span" v-if="showQuestionTextInput">
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
          <div class="form-group" v-if="showImageUpload">
            <label>{{ imageLabel }}</label>
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
        <div class="transfer-header">
          <div>
            <h3>Import / Export Questions</h3>
            <p>Export a filtered set into one file, then import that file to add the questions back into the app.</p>
          </div>
          <span class="transfer-count">{{ filteredExportQuestions.length }} ready</span>
        </div>
        <div class="form-grid">
          <div class="form-group">
            <label>Export Part</label>
            <select v-model="exportOptions.part" class="form-input">
              <option value="all">All parts</option>
              <option value="1.1">Part 1.1</option>
              <option value="1.2">Part 1.2</option>
              <option value="2">Part 2</option>
              <option value="3">Part 3</option>
            </select>
          </div>
          <div class="form-group">
            <label>Export Pack</label>
            <select v-model="exportOptions.packId" class="form-input">
              <option value="all">All packs</option>
              <option
                v-for="packId in exportPackOptions"
                :key="packId"
                :value="packId"
              >
                {{ packId }}
              </option>
            </select>
          </div>
          <div class="form-group">
            <label>From Question ID</label>
            <input
              v-model="exportOptions.fromId"
              type="number"
              min="1"
              class="form-input"
              placeholder="Any"
            />
          </div>
          <div class="form-group">
            <label>To Question ID</label>
            <input
              v-model="exportOptions.toId"
              type="number"
              min="1"
              class="form-input"
              placeholder="Any"
            />
          </div>
        </div>
        <div class="transfer-actions">
          <label class="checkbox-row">
            <input v-model="exportOptions.selectedOnly" type="checkbox" />
            <span>Export selected only ({{ selectedQuestionIds.length }})</span>
          </label>
          <button class="secondary-btn" @click="selectFilteredQuestions">
            Select filtered
          </button>
          <button class="secondary-btn" @click="clearQuestionSelection">
            Clear selection
          </button>
          <button
            class="add-btn"
            :disabled="exportingQuestions || filteredExportQuestions.length === 0"
            @click="exportQuestionsFile"
          >
            {{ exportingQuestions ? 'Exporting...' : 'Export Questions' }}
          </button>
        </div>
        <div class="import-row">
          <input
            ref="importFileInput"
            type="file"
            accept=".json,application/json"
            class="file-input"
            @change="onImportFileSelected"
          />
          <label class="file-label import-label" @click="$refs.importFileInput?.click()">
            {{ importFileName ? `Selected: ${importFileName}` : 'Choose Export File' }}
          </label>
          <button
            class="action-btn"
            :disabled="importingQuestions || !importFileText"
            @click="importQuestionsFile"
          >
            {{ importingQuestions ? 'Importing...' : 'Import Questions' }}
          </button>
        </div>
      </div>

      <div class="section">
        <h3>Existing Questions</h3>
        <div v-if="questions.length === 0" class="empty-state">
          <p>No questions added yet</p>
        </div>
        <div v-else class="questions-list">
          <div v-for="question in questions" :key="question.id" class="question-card">
            <div class="question-info">
              <label class="question-select">
                <input
                  type="checkbox"
                  :checked="isQuestionSelected(question.id)"
                  @change="toggleQuestionSelection(question.id, $event.target.checked)"
                />
                <span>ID {{ question.id }}</span>
              </label>
              <strong>{{ formatPart(question.part, question.sub_part) }}</strong>
              <span v-if="question.text" class="question-text">{{ question.text }}</span>
              <span>{{ question.response_time }}s</span>
              <span v-if="question.pack_id" class="tag">Pack {{ question.pack_id }}</span>
              <span v-if="question.pack_order" class="tag">Order {{ question.pack_order }}</span>
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

    <!-- Full Tests Tab -->
    <div v-if="activeTab === 'Full Tests'" class="tab-content">
      <div class="section">
        <div class="section-header">
          <div>
            <h2>Create Full Test</h2>
            <p class="muted-copy">Choose questions from Part 1.1, Part 1.2, Part 2, and Part 3, then save them as one student-facing test.</p>
          </div>
          <span class="transfer-count">{{ fullTestQuestionIds.length }} selected</span>
        </div>

        <div class="form-grid">
          <div class="form-group form-span">
            <label>Test Name</label>
            <input
              v-model.trim="newFullTest.name"
              type="text"
              class="form-input"
              placeholder="e.g. Mock Test 1"
              @keyup.enter="createFullTest"
            />
          </div>
        </div>

        <div class="test-section-counts">
          <span
            v-for="section in sectionOrder"
            :key="section.key"
            class="test-count-chip"
            :class="{ complete: fullTestSectionCounts[section.key] > 0 }"
          >
            {{ section.label }}: {{ fullTestSectionCounts[section.key] || 0 }}
          </span>
        </div>

        <div v-if="questions.length === 0" class="empty-state">
          <p>No questions added yet. Add questions before creating a full test.</p>
        </div>
        <div v-else class="test-builder-grid">
          <section
            v-for="group in fullTestQuestionGroups"
            :key="group.key"
            class="test-builder-group"
          >
            <div class="test-group-header">
              <h3>{{ group.label }}</h3>
              <button
                type="button"
                class="secondary-btn compact"
                @click="toggleFullTestSection(group.questions)"
              >
                {{ isFullTestSectionSelected(group.questions) ? 'Clear' : 'Select all' }}
              </button>
            </div>

            <label
              v-for="question in group.questions"
              :key="question.id"
              class="test-question-option"
              :class="{ selected: isFullTestQuestionSelected(question.id) }"
            >
              <input
                type="checkbox"
                :checked="isFullTestQuestionSelected(question.id)"
                @change="toggleFullTestQuestion(question.id, $event.target.checked)"
              />
              <span>
                <strong>ID {{ question.id }}</strong>
                <small>{{ describeQuestion(question) }}</small>
                <em>
                  {{ question.response_time }}s
                  <template v-if="question.pack_id"> - Pack {{ question.pack_id }}</template>
                  <template v-if="question.pack_order"> - Order {{ question.pack_order }}</template>
                </em>
              </span>
            </label>
          </section>
        </div>

        <div class="test-actions">
          <button
            class="secondary-btn"
            type="button"
            @click="clearFullTestSelection"
          >
            Clear selection
          </button>
          <button
            class="add-btn"
            :disabled="creatingFullTest || !canCreateFullTest"
            @click="createFullTest"
          >
            {{ creatingFullTest ? 'Saving...' : 'Save Full Test' }}
          </button>
        </div>
      </div>

      <div class="section">
        <h2>Existing Full Tests</h2>
        <div v-if="fullTests.length === 0" class="empty-state">
          <p>No full tests created yet</p>
        </div>
        <div v-else class="full-tests-list">
          <div
            v-for="test in fullTests"
            :key="test.id"
            class="full-test-card"
          >
            <div class="full-test-main">
              <h3>{{ test.name }}</h3>
              <p>{{ test.questions.length }} question(s) - Created {{ formatDate(test.created_at) }}</p>
              <div class="test-section-counts">
                <span
                  v-for="section in sectionOrder"
                  :key="section.key"
                  class="test-count-chip"
                  :class="{ complete: getFullTestSectionCount(test, section) > 0 }"
                >
                  {{ section.label }}: {{ getFullTestSectionCount(test, section) }}
                </span>
              </div>
            </div>
            <button
              class="delete-btn"
              :disabled="test.isDeleting"
              @click="deleteFullTest(test)"
            >
              {{ test.isDeleting ? 'Deleting...' : 'Delete' }}
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
        <div
          v-for="attempt in recordingsByAttempt"
          :key="attempt.attemptId"
          class="recording-group"
          :id="`recording-group-${attempt.attemptId}`"
          :class="{ highlight: selectedAttemptId === attempt.attemptId }"
        >
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
                <button
                  class="delete-recording-btn"
                  :disabled="recording.isDeleting"
                  @click="deleteRecording(recording)"
                >
                  {{ recording.isDeleting ? 'Deleting...' : 'Delete' }}
                </button>
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
import { ref, computed, onMounted, onUnmounted, watch, nextTick } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const activeTab = ref('Attempts')
const tabs = ['Attempts', 'Questions', 'Full Tests', 'Recordings', 'Statistics']
const searchQuery = ref('')
const attempts = ref([])
const questions = ref([])
const fullTests = ref([])
const recordings = ref([])
const addingQuestion = ref(false)
const creatingFullTest = ref(false)
const telegramChatIdInput = ref('')
const telegramChatIds = ref([])
const savingTelegramChatIds = ref(false)
const audioInput = ref(null)
const imageInput = ref(null)
const importFileInput = ref(null)
const selectedAttemptId = ref(null)
const selectedQuestionIds = ref([])
const fullTestQuestionIds = ref([])
const exportingQuestions = ref(false)
const importingQuestions = ref(false)
const importFileName = ref('')
const importFileText = ref('')
const exportOptions = ref({
  part: 'all',
  packId: 'all',
  fromId: '',
  toId: '',
  selectedOnly: false
})

const newQuestion = ref({
  part: '',
  response_time: 30,
  text: '',
  packId: '',
  packOrder: 1,
  audioFileName: '',
  audioData: null,
  imageFileName: '',
  imageData: null
})

const newFullTest = ref({
  name: ''
})

const sectionOrder = [
  { key: '1-1', label: 'Part 1.1', part: 1, subPart: 1 },
  { key: '1-2', label: 'Part 1.2', part: 1, subPart: 2 },
  { key: '2-0', label: 'Part 2', part: 2, subPart: 0 },
  { key: '3-0', label: 'Part 3', part: 3, subPart: 0 }
]

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

const exportPackOptions = computed(() => {
  const packIds = questions.value
    .map(question => question.pack_id)
    .filter(packId => typeof packId === 'string' && packId.trim())

  return Array.from(new Set(packIds)).sort((a, b) => a.localeCompare(b))
})

const exportFilteredQuestions = computed(() => {
  const fromId = parseOptionalPositiveInteger(exportOptions.value.fromId)
  const toId = parseOptionalPositiveInteger(exportOptions.value.toId)

  return questions.value.filter((question) => {
    if (exportOptions.value.part !== 'all') {
      const selectedPart = parsePartSelection(exportOptions.value.part)
      if (question.part !== selectedPart.part || question.sub_part !== selectedPart.subPart) {
        return false
      }
    }

    if (exportOptions.value.packId !== 'all' && question.pack_id !== exportOptions.value.packId) {
      return false
    }

    if (fromId !== null && question.id < fromId) {
      return false
    }

    if (toId !== null && question.id > toId) {
      return false
    }

    return true
  })
})

const filteredExportQuestions = computed(() => {
  if (!exportOptions.value.selectedOnly) {
    return exportFilteredQuestions.value
  }

  const selectedIds = new Set(selectedQuestionIds.value)
  return exportFilteredQuestions.value.filter(question => selectedIds.has(question.id))
})

const fullTestQuestionGroups = computed(() => {
  return sectionOrder
    .map((section) => ({
      ...section,
      questions: questions.value
        .filter(question => question.part === section.part && question.sub_part === section.subPart)
        .sort(sortQuestionForPicker)
    }))
    .filter(group => group.questions.length > 0)
})

const fullTestSectionCounts = computed(() => {
  const selectedIds = new Set(fullTestQuestionIds.value)
  return sectionOrder.reduce((counts, section) => {
    counts[section.key] = questions.value.filter((question) => (
      selectedIds.has(question.id) &&
      question.part === section.part &&
      question.sub_part === section.subPart
    )).length
    return counts
  }, {})
})

const missingFullTestSections = computed(() => {
  return sectionOrder
    .filter(section => (fullTestSectionCounts.value[section.key] || 0) === 0)
    .map(section => section.label)
})

const canCreateFullTest = computed(() => {
  return Boolean(newFullTest.value.name.trim()) &&
    fullTestQuestionIds.value.length > 0 &&
    missingFullTestSections.value.length === 0
})

const needsPack = computed(() => newQuestion.value.part === '1.1' || newQuestion.value.part === '1.2')
const showImageUpload = computed(() => ['1.2', '2', '3'].includes(newQuestion.value.part))
const imageRequired = computed(() => newQuestion.value.part === '1.2' || newQuestion.value.part === '3')
const imageLabel = computed(() => {
  if (newQuestion.value.part === '1.2') return 'Image File (Required for Part 1.2)'
  if (newQuestion.value.part === '3') return 'Image File (Required for Part 3)'
  if (newQuestion.value.part === '2') return 'Image File (Optional)'
  return 'Image File'
})
const showQuestionTextInput = computed(() => newQuestion.value.part !== '3')

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

async function onImportFileSelected(event) {
  const file = event.target.files?.[0]
  if (!file) {
    importFileName.value = ''
    importFileText.value = ''
    return
  }

  importFileName.value = file.name
  importFileText.value = await file.text()
}

async function loadAttempts() {
  try {
    const data = await invoke('get_attempts')
    attempts.value = (data || []).map((attempt) => ({
      ...attempt,
      isDeleting: false
    }))
  } catch (error) {
    console.error('Failed to load attempts:', error)
  }
}

async function loadQuestions() {
  try {
    const data = await invoke('get_questions')
    questions.value = data || []
    const activeIds = new Set(questions.value.map(question => question.id))
    selectedQuestionIds.value = selectedQuestionIds.value.filter(id => activeIds.has(id))
    fullTestQuestionIds.value = fullTestQuestionIds.value.filter(id => activeIds.has(id))
  } catch (error) {
    console.error('Failed to load questions:', error)
  }
}

async function loadFullTests() {
  try {
    const data = await invoke('get_full_tests')
    fullTests.value = (data || []).map((test) => ({
      ...test,
      isDeleting: false
    }))
  } catch (error) {
    console.error('Failed to load full tests:', error)
  }
}

async function loadRecordings() {
  try {
    revokeRecordingUrls()
    const data = await invoke('get_recordings')
    recordings.value = (data || []).map((recording) => ({
      ...recording,
      audioUrl: '',
      isLoading: false,
      isDeleting: false
    }))
  } catch (error) {
    console.error('Failed to load recordings:', error)
  }
}

async function loadTelegramChatIds() {
  try {
    const chatIds = await invoke('get_telegram_chat_ids')
    telegramChatIds.value = Array.isArray(chatIds)
      ? chatIds.map((chatId) => chatId.toString())
      : []
  } catch (error) {
    console.error('Failed to load Telegram chat IDs:', error)
  }
}

async function addTelegramChatId() {
  const chatId = telegramChatIdInput.value.trim()
  if (!chatId) {
    alert('Please enter a Telegram chat ID')
    return
  }
  if (telegramChatIds.value.includes(chatId)) {
    alert('This Telegram chat ID is already in the list')
    return
  }

  const nextChatIds = [...telegramChatIds.value, chatId]
  const saved = await persistTelegramChatIds(nextChatIds)
  if (saved) {
    telegramChatIdInput.value = ''
  }
}

async function removeTelegramChatId(chatIdToRemove) {
  const nextChatIds = telegramChatIds.value.filter(chatId => chatId !== chatIdToRemove)
  await persistTelegramChatIds(nextChatIds)
}

async function persistTelegramChatIds(nextChatIds) {
  savingTelegramChatIds.value = true
  try {
    const savedChatIds = await invoke('set_telegram_chat_ids', { chatIds: nextChatIds })
    telegramChatIds.value = Array.isArray(savedChatIds)
      ? savedChatIds.map((chatId) => chatId.toString())
      : nextChatIds
    return true
  } catch (error) {
    console.error('Failed to save Telegram chat IDs:', error)
    alert('Error saving Telegram chat IDs: ' + (error?.message || String(error)))
    return false
  } finally {
    savingTelegramChatIds.value = false
  }
}

function isQuestionSelected(questionId) {
  return selectedQuestionIds.value.includes(questionId)
}

function toggleQuestionSelection(questionId, isSelected) {
  if (isSelected) {
    if (!selectedQuestionIds.value.includes(questionId)) {
      selectedQuestionIds.value = [...selectedQuestionIds.value, questionId]
    }
    return
  }

  selectedQuestionIds.value = selectedQuestionIds.value.filter(id => id !== questionId)
}

function selectFilteredQuestions() {
  const nextIds = new Set(selectedQuestionIds.value)
  exportFilteredQuestions.value.forEach(question => nextIds.add(question.id))
  selectedQuestionIds.value = Array.from(nextIds)
}

function clearQuestionSelection() {
  selectedQuestionIds.value = []
}

function sortQuestionForPicker(a, b) {
  const orderA = Number(a.pack_order) || Number.MAX_SAFE_INTEGER
  const orderB = Number(b.pack_order) || Number.MAX_SAFE_INTEGER

  if (a.part !== b.part) return a.part - b.part
  if (a.sub_part !== b.sub_part) return a.sub_part - b.sub_part
  if (orderA !== orderB) return orderA - orderB
  return a.id - b.id
}

function isFullTestQuestionSelected(questionId) {
  return fullTestQuestionIds.value.includes(questionId)
}

function toggleFullTestQuestion(questionId, isSelected) {
  if (isSelected) {
    if (!fullTestQuestionIds.value.includes(questionId)) {
      fullTestQuestionIds.value = [...fullTestQuestionIds.value, questionId]
    }
    return
  }

  fullTestQuestionIds.value = fullTestQuestionIds.value.filter(id => id !== questionId)
}

function isFullTestSectionSelected(sectionQuestions) {
  return sectionQuestions.length > 0 &&
    sectionQuestions.every(question => fullTestQuestionIds.value.includes(question.id))
}

function toggleFullTestSection(sectionQuestions) {
  const sectionIds = sectionQuestions.map(question => question.id)

  if (isFullTestSectionSelected(sectionQuestions)) {
    fullTestQuestionIds.value = fullTestQuestionIds.value.filter(id => !sectionIds.includes(id))
    return
  }

  fullTestQuestionIds.value = [...new Set([...fullTestQuestionIds.value, ...sectionIds])]
}

function clearFullTestSelection() {
  fullTestQuestionIds.value = []
}

function describeQuestion(question) {
  const text = question.text?.trim()
  if (text) return text
  if (question.image_path) return 'Image prompt'
  if (question.audio_path) return 'Audio prompt'
  return 'Untitled question'
}

function getFullTestSectionCount(test, section) {
  return (test.questions || []).filter((question) => (
    question.part === section.part && question.sub_part === section.subPart
  )).length
}

async function createFullTest() {
  if (!newFullTest.value.name.trim()) {
    alert('Please enter a full test name')
    return
  }

  if (missingFullTestSections.value.length > 0) {
    alert(`Please include at least one question from: ${missingFullTestSections.value.join(', ')}`)
    return
  }

  creatingFullTest.value = true
  try {
    const selectedIds = new Set(fullTestQuestionIds.value)
    const questionIds = questions.value
      .filter(question => selectedIds.has(question.id))
      .sort(sortQuestionForPicker)
      .map(question => question.id)

    await invoke('create_full_test', {
      name: newFullTest.value.name.trim(),
      questionIds
    })

    newFullTest.value.name = ''
    fullTestQuestionIds.value = []
    await loadFullTests()
    alert('Full test saved successfully!')
  } catch (error) {
    console.error('Failed to create full test:', error)
    alert('Error creating full test: ' + (error?.message || String(error)))
  } finally {
    creatingFullTest.value = false
  }
}

async function deleteFullTest(test) {
  if (test.isDeleting) return
  const confirmDelete = confirm(`Delete "${test.name}"? Students will no longer be able to choose it.`)
  if (!confirmDelete) return

  test.isDeleting = true
  try {
    await invoke('delete_full_test', {
      fullTestId: test.id
    })
    fullTests.value = fullTests.value.filter(item => item.id !== test.id)
  } catch (error) {
    console.error('Failed to delete full test:', error)
    alert('Error deleting full test: ' + (error?.message || String(error)))
  } finally {
    test.isDeleting = false
  }
}

async function exportQuestionsFile() {
  const questionIds = filteredExportQuestions.value.map(question => question.id)
  if (questionIds.length === 0) {
    alert('Choose at least one question to export')
    return
  }

  exportingQuestions.value = true
  try {
    const exportJson = await invoke('export_questions', {
      questionIds
    })
    const blob = new Blob([exportJson], { type: 'application/json' })
    const downloadUrl = URL.createObjectURL(blob)
    const downloadLink = document.createElement('a')
    downloadLink.href = downloadUrl
    downloadLink.download = `cefr-questions-${new Date().toISOString().slice(0, 10)}.json`
    document.body.appendChild(downloadLink)
    downloadLink.click()
    downloadLink.remove()
    URL.revokeObjectURL(downloadUrl)
    alert(`Exported ${questionIds.length} question(s) successfully.`)
  } catch (error) {
    console.error('Failed to export questions:', error)
    alert('Error exporting questions: ' + (error?.message || String(error)))
  } finally {
    exportingQuestions.value = false
  }
}

async function importQuestionsFile() {
  if (!importFileText.value) {
    alert('Choose a questions export file first')
    return
  }

  importingQuestions.value = true
  try {
    const result = await invoke('import_questions', {
      exportJson: importFileText.value
    })
    await loadQuestions()
    await loadFullTests()
    importFileName.value = ''
    importFileText.value = ''
    if (importFileInput.value) {
      importFileInput.value.value = ''
    }
    alert(`Imported ${result.imported || 0} question(s) successfully.`)
  } catch (error) {
    console.error('Failed to import questions:', error)
    alert('Error importing questions: ' + (error?.message || String(error)))
  } finally {
    importingQuestions.value = false
  }
}

async function addQuestion() {
  if (!newQuestion.value.part) {
    alert('Please select a part')
    return
  }

  if (imageRequired.value && !newQuestion.value.imageData) {
    const partLabel = newQuestion.value.part === '3' ? 'Part 3' : 'Part 1.2'
    alert(`Please upload an image for ${partLabel}`)
    return
  }

  if (needsPack.value && !newQuestion.value.packId.trim()) {
    alert('Please enter a test pack ID for Part 1.1 and Part 1.2')
    return
  }

  if (needsPack.value && (!Number.isFinite(newQuestion.value.packOrder) || newQuestion.value.packOrder < 1)) {
    alert('Please enter the order of this question in the test pack')
    return
  }

  addingQuestion.value = true
  try {
    const { part, subPart } = parsePartSelection(newQuestion.value.part)
    const requiresPack = part === 1 && (subPart === 1 || subPart === 2)
    let audioPath = null
    let imagePath = null

    if (newQuestion.value.audioData) {
      const extension = getFileExtension(newQuestion.value.audioData.name, 'wav')
      const uniqueSuffix = `${Date.now()}_${Math.random().toString(16).slice(2, 8)}`
      const filename = `part${part}_${subPart || 0}_${uniqueSuffix}.${extension}`

      const arrayBuffer = await newQuestion.value.audioData.arrayBuffer()
      const audioBytes = Array.from(new Uint8Array(arrayBuffer))

      const savedFilename = await invoke('save_audio_file', {
        filename: filename,
        audioData: audioBytes
      })

      audioPath = savedFilename
    }

    if (newQuestion.value.imageData) {
      const extension = getFileExtension(newQuestion.value.imageData.name, 'png')
      const uniqueSuffix = `${Date.now()}_${Math.random().toString(16).slice(2, 8)}`
      const filename = `part${part}_${subPart || 0}_${uniqueSuffix}.${extension}`
      const arrayBuffer = await newQuestion.value.imageData.arrayBuffer()
      const imageBytes = Array.from(new Uint8Array(arrayBuffer))

      const savedFilename = await invoke('save_image_file', {
        filename: filename,
        imageData: imageBytes
      })

      imagePath = savedFilename
    }

    const questionText = newQuestion.value.part === '3' ? '' : (newQuestion.value.text?.trim() || '')
    const packId = requiresPack ? newQuestion.value.packId.trim() : ''
    const packOrder = requiresPack ? Number(newQuestion.value.packOrder || 0) : 0

    await invoke('add_question', {
      part: part,
      subPart: subPart || undefined,
      responseTime: newQuestion.value.response_time,
      audioPath: audioPath,
      imagePath: imagePath,
      text: questionText,
      packId: packId,
      packOrder: packOrder
    })

    resetQuestionForm()
    await loadQuestions()
    await loadFullTests()
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

async function deleteRecording(recording) {
  if (recording.isDeleting) return
  const confirmDelete = confirm('Delete this recording? This will remove the audio file and cannot be undone.')
  if (!confirmDelete) return
  recording.isDeleting = true
  try {
    await invoke('delete_response', {
      responseId: recording.id
    })
    if (recording.audioUrl) {
      URL.revokeObjectURL(recording.audioUrl)
    }
    recordings.value = recordings.value.filter(item => item.id !== recording.id)
  } catch (error) {
    console.error('Failed to delete recording:', error)
    alert('Error deleting recording: ' + (error?.message || String(error)))
  } finally {
    recording.isDeleting = false
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
    packId: '',
    packOrder: 1,
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

function parseOptionalPositiveInteger(value) {
  if (value === null || value === undefined || value === '') return null
  const parsed = Number(value)
  return Number.isFinite(parsed) && parsed > 0 ? parsed : null
}

function getFileExtension(filename, fallback = 'png') {
  const segments = filename.split('.')
  return segments.length > 1 ? segments.pop().toLowerCase() : fallback
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

async function viewAttemptDetails(attempt) {
  selectedAttemptId.value = attempt.id
  activeTab.value = 'Recordings'
  await nextTick()
  scrollToAttemptRecordings(attempt.id)
}

function scrollToAttemptRecordings(attemptId) {
  const target = document.getElementById(`recording-group-${attemptId}`)
  if (target) {
    target.scrollIntoView({ behavior: 'smooth', block: 'start' })
  }
}

async function deleteAttempt(attempt) {
  if (attempt.isDeleting) return
  const confirmDelete = confirm('Delete this attempt and all related recordings? This cannot be undone.')
  if (!confirmDelete) return
  attempt.isDeleting = true
  try {
    await invoke('delete_attempt', {
      attemptId: attempt.id
    })
    attempts.value = attempts.value.filter(item => item.id !== attempt.id)
    await loadRecordings()
    if (selectedAttemptId.value === attempt.id) {
      selectedAttemptId.value = null
    }
  } catch (error) {
    console.error('Failed to delete attempt:', error)
    alert('Error deleting attempt: ' + (error?.message || String(error)))
  } finally {
    attempt.isDeleting = false
  }
}

async function deleteQuestion(questionId) {
  if (confirm('Are you sure you want to delete this question? This action cannot be undone.')) {
    try {
      await invoke('delete_question', {
        questionId: questionId
      })
      await loadQuestions()
      await loadFullTests()
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
  loadFullTests()
  loadRecordings()
  loadTelegramChatIds()
})

onUnmounted(() => {
  revokeRecordingUrls()
})

watch(() => newQuestion.value.part, (value) => {
  const allowImage = ['1.2', '2', '3'].includes(value)
  if (!allowImage) {
    newQuestion.value.imageFileName = ''
    newQuestion.value.imageData = null
    if (imageInput.value) {
      imageInput.value.value = ''
    }
  }

  if (value !== '1.1' && value !== '1.2') {
    newQuestion.value.packId = ''
    newQuestion.value.packOrder = 1
  }

  if (value === '3') {
    newQuestion.value.text = ''
  }
})

watch(recordingsByAttempt, (groups) => {
  if (!selectedAttemptId.value || activeTab.value !== 'Recordings') return
  const exists = groups.some(group => group.attemptId === selectedAttemptId.value)
  if (exists) {
    nextTick(() => scrollToAttemptRecordings(selectedAttemptId.value))
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

.telegram-settings-card {
  margin-bottom: 24px;
  padding: 20px;
  border-radius: 16px;
  background: rgba(255, 255, 255, 0.95);
  border: 1px solid rgba(148, 163, 184, 0.25);
  box-shadow: 0 16px 40px rgba(15, 23, 42, 0.08);
}

.telegram-settings-header h2 {
  margin: 0;
  font-size: 18px;
  color: #0f172a;
}

.telegram-settings-header p {
  margin: 8px 0 0;
  color: #64748b;
  font-size: 14px;
}

.telegram-settings-controls {
  margin-top: 14px;
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  gap: 12px;
  align-items: center;
}

.icon-btn {
  border: none;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  font-weight: 700;
  cursor: pointer;
  transition: transform 0.2s ease, background 0.2s ease;
}

.add-telegram-btn {
  background: #0f766e;
  color: white;
  border-radius: 10px;
  width: 44px;
  height: 44px;
  font-size: 28px;
  line-height: 1;
}

.add-telegram-btn:hover:not(:disabled) {
  background: #115e59;
  transform: translateY(-1px);
}

.add-telegram-btn:disabled {
  background: #94a3b8;
  cursor: not-allowed;
}

.telegram-chat-list {
  margin-top: 14px;
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
}

.telegram-chat-chip {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 6px 8px 6px 12px;
  border-radius: 999px;
  background: #f1f5f9;
  border: 1px solid rgba(148, 163, 184, 0.35);
  color: #0f172a;
  font-size: 13px;
  font-weight: 600;
}

.delete-telegram-btn {
  width: 22px;
  height: 22px;
  border-radius: 999px;
  background: #ef4444;
  color: white;
  font-size: 12px;
  line-height: 1;
}

.delete-telegram-btn:hover:not(:disabled) {
  background: #dc2626;
}

.delete-telegram-btn:disabled {
  background: #fca5a5;
  cursor: not-allowed;
}

.telegram-empty {
  margin: 14px 0 0;
  color: #64748b;
  font-size: 14px;
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

.muted-copy {
  margin: 8px 0 0;
  color: #64748b;
  font-size: 14px;
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

.attempt-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.danger-btn {
  background: #ef4444;
  color: white;
  padding: 0.5rem 1rem;
  border: none;
  border-radius: 8px;
  cursor: pointer;
  font-size: 0.875rem;
  font-weight: 600;
  transition: transform 0.2s ease, background 0.2s ease;
}

.danger-btn:hover:not(:disabled) {
  background: #dc2626;
  transform: translateY(-1px);
}

.danger-btn:disabled {
  background: #fca5a5;
  cursor: not-allowed;
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

.transfer-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 16px;
  margin-bottom: 18px;
}

.transfer-header p {
  margin: 8px 0 0;
  color: #64748b;
  font-size: 14px;
}

.transfer-count {
  display: inline-flex;
  align-items: center;
  min-height: 32px;
  padding: 4px 10px;
  border-radius: 8px;
  background: #e0f2fe;
  color: #075985;
  font-size: 13px;
  font-weight: 700;
  white-space: nowrap;
}

.transfer-actions,
.import-row {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 12px;
}

.transfer-actions {
  margin-bottom: 16px;
}

.transfer-actions .add-btn {
  border-radius: 8px;
}

.checkbox-row,
.question-select {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  color: #475569;
  font-size: 14px;
  font-weight: 600;
}

.secondary-btn {
  background: #334155;
  color: white;
  padding: 0.7rem 1rem;
  border: none;
  border-radius: 8px;
  cursor: pointer;
  font-weight: 600;
  transition: transform 0.2s ease, background 0.2s ease;
}

.secondary-btn:hover {
  background: #0f172a;
  transform: translateY(-1px);
}

.secondary-btn.compact {
  padding: 0.45rem 0.75rem;
  font-size: 0.8rem;
}

.import-label {
  width: min(360px, 100%);
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

.test-section-counts {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin: 12px 0 18px;
}

.test-count-chip {
  display: inline-flex;
  align-items: center;
  padding: 5px 9px;
  border-radius: 999px;
  background: #fee2e2;
  color: #991b1b;
  font-size: 12px;
  font-weight: 700;
}

.test-count-chip.complete {
  background: #dcfce7;
  color: #166534;
}

.test-builder-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
  gap: 16px;
  margin-top: 18px;
}

.test-builder-group {
  display: grid;
  gap: 10px;
  padding: 16px;
  border-radius: 14px;
  background: #f8fafc;
  border: 1px solid rgba(148, 163, 184, 0.3);
}

.test-group-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.test-group-header h3 {
  margin: 0;
  font-size: 16px;
  color: #0f172a;
}

.test-question-option {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  padding: 10px;
  border-radius: 10px;
  background: white;
  border: 1px solid rgba(148, 163, 184, 0.28);
  cursor: pointer;
}

.test-question-option.selected {
  background: #eff6ff;
  border-color: rgba(37, 99, 235, 0.42);
}

.test-question-option input {
  margin-top: 3px;
  accent-color: #1d4ed8;
}

.test-question-option span {
  display: grid;
  min-width: 0;
  gap: 4px;
}

.test-question-option strong {
  color: #0f172a;
  font-size: 13px;
}

.test-question-option small {
  color: #334155;
  font-size: 13px;
  line-height: 1.35;
}

.test-question-option em {
  color: #64748b;
  font-size: 12px;
  font-style: normal;
  font-weight: 600;
}

.test-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
  margin-top: 18px;
}

.full-tests-list {
  display: grid;
  gap: 14px;
  margin-top: 16px;
}

.full-test-card {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
  padding: 16px;
  border-radius: 14px;
  background: #f8fafc;
  border: 1px solid rgba(148, 163, 184, 0.3);
}

.full-test-main {
  min-width: 0;
}

.full-test-main h3 {
  margin: 0 0 5px;
  color: #0f172a;
}

.full-test-main p {
  margin: 0;
  color: #64748b;
  font-size: 13px;
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

.recording-group.highlight {
  border-color: #2563eb;
  box-shadow: 0 0 0 3px rgba(37, 99, 235, 0.2);
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

.delete-recording-btn {
  background: #ef4444;
  color: white;
  border: none;
  padding: 0.5rem 0.9rem;
  border-radius: 10px;
  cursor: pointer;
  font-weight: 600;
  transition: background 0.2s ease;
}

.delete-recording-btn:hover:not(:disabled) {
  background: #dc2626;
}

.delete-recording-btn:disabled {
  background: #fca5a5;
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

  .telegram-settings-controls {
    grid-template-columns: 1fr;
  }
}
</style>
