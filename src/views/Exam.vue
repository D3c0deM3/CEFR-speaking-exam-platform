<template>
  <div class="exam-page" v-if="!isFinished">
    <div class="ambient-bg"></div>
    <div
      v-if="isInstructionActive"
      class="instruction-overlay"
      :class="{ 'fade-out': isInstructionFading }"
      aria-live="polite"
    >
      <div class="instruction-panel">
        <span class="instruction-tag">Instruction</span>
        <h2>{{ instructionTitle }}</h2>
        <p>{{ instructionMessage }}</p>
      </div>
    </div>
    <header class="exam-topbar">
      <div class="brand">
        <span class="brand-title">CEFR Speaking</span>
        <span class="brand-sub">Mock Test</span>
      </div>
      <div class="top-meta">
        <div class="chip part-chip">{{ currentPartLabel }}</div>
        <div class="chip question-chip">Question {{ currentQuestion + 1 }} / {{ questions.length }}</div>
      </div>
      <div class="timer" :class="{ warning: timeRemaining < 10 }">
        {{ formattedTime }}
      </div>
    </header>

    <div class="progress-container">
      <div class="progress-bar" :style="{ width: progress + '%' }"></div>
      <div class="progress-text">{{ currentPartLabel }} - Question {{ currentQuestion + 1 }}</div>
    </div>

    <main class="exam-layout" v-if="currentQuestionData">
      <section class="question-card">
        <div class="status-row">
          <span class="status-pill" :class="{ playing: isPlaying, recording: isRecording }">
            {{ statusLabel }}
          </span>
          <span class="info-pill" v-if="hasQuestionAudio">Audio Prompt</span>
          <span class="info-pill" v-if="hasQuestionImage">Image Prompt</span>
        </div>

        <div v-if="currentQuestionData?.text" class="question-text">
          {{ currentQuestionData.text }}
        </div>

        <div class="image-frame" v-if="hasQuestionImage && imageUrl">
          <img :src="imageUrl" alt="Question visual" />
        </div>

        <div class="audio-player" v-if="hasQuestionAudio">
          <button
            v-if="needsManualPlay"
            @click="playQuestionAudio(true)"
            class="play-button"
            :disabled="isPlaying || isRecording"
          >
            {{ isPlaying ? 'Playing...' : 'Play Question Audio' }}
          </button>
          <p class="helper-text" v-if="needsManualPlay">
            Tap play to begin. Recording starts after the bell.
          </p>
        </div>

        <div v-else class="no-audio-message">
          {{ isPreparing ? 'Preparation time. Recording begins after the bell.' : 'No audio prompt. Recording begins after the bell.' }}
        </div>

        <div class="voice-visual" :class="{ active: isRecording }">
          <div class="voice-waves" aria-hidden="true" :style="waveStyle">
            <svg class="wave-svg" viewBox="0 0 600 120" role="presentation">
              <defs>
                <linearGradient id="waveGradient" x1="0%" y1="0%" x2="100%" y2="0%">
                  <stop offset="0%" stop-color="#38bdf8" />
                  <stop offset="45%" stop-color="#6366f1" />
                  <stop offset="100%" stop-color="#fb7185" />
                </linearGradient>
                <linearGradient id="waveAccent" x1="0%" y1="0%" x2="100%" y2="0%">
                  <stop offset="0%" stop-color="#22d3ee" />
                  <stop offset="50%" stop-color="#a855f7" />
                  <stop offset="100%" stop-color="#f97316" />
                </linearGradient>
                <filter id="waveGlow" x="-20%" y="-50%" width="140%" height="200%">
                  <feGaussianBlur stdDeviation="6" result="blur" />
                  <feMerge>
                    <feMergeNode in="blur" />
                    <feMergeNode in="SourceGraphic" />
                  </feMerge>
                </filter>
              </defs>
              <path
                class="wave-base"
                d="M0 60 C60 55 120 65 180 60 C240 55 300 65 360 60 C420 55 480 65 540 60 C570 58 600 60 600 60"
              />
              <path
                class="wave wave-1"
                d="M0 60 C60 20 120 100 180 60 C240 20 300 100 360 60 C420 20 480 100 540 60 C570 50 600 60 600 60"
              />
              <path
                class="wave wave-2"
                d="M0 60 C70 40 140 80 210 60 C280 40 350 80 420 60 C490 40 560 80 600 60"
              />
              <path
                class="wave wave-3"
                d="M0 60 C80 55 160 65 240 60 C320 55 400 65 480 60 C560 55 600 60 600 60"
              />
              <path
                class="wave wave-4"
                d="M0 60 C90 70 180 50 270 60 C360 70 450 50 540 60 C580 64 600 60 600 60"
              />
              <path
                class="wave wave-5"
                d="M0 60 C75 30 150 90 225 60 C300 30 375 90 450 60 C525 30 600 60 600 60"
              />
              <path
                class="wave wave-6"
                d="M0 60 C100 45 200 75 300 60 C400 45 500 75 600 60"
              />
            </svg>
          </div>
          <span>
            {{ isPreparing ? 'Preparation time' : isRecording ? 'Recording your response' : 'Ready to record' }}
          </span>
        </div>
        <div class="auto-note" v-if="!isPlaying && !isRecording && !isPreparing">
          Recording starts after the bell.
        </div>
      </section>

      <section class="info-card">
        <h3>Instructions</h3>
        <p v-if="currentPart === 1 && currentSubPart === 1">
          Part 1.1: Short personal questions. Keep your answers brief and natural.
        </p>
        <p v-else-if="currentPart === 1 && currentSubPart === 2">
          Part 1.2: Look at the image and respond with details and opinions.
        </p>
        <p v-else-if="currentPart === 2">
          Part 2: Speak for 1-2 minutes on the prompt.
        </p>
        <p v-else>
          Part 3: One discussion question related to the Part 2 topic.
        </p>

        <div class="info-grid">
          <div class="info-item">
            <span class="info-label">Response Time</span>
            <span class="info-value">{{ currentQuestionData?.response_time || 0 }}s</span>
          </div>
          <div class="info-item">
            <span class="info-label">Progress</span>
            <span class="info-value">{{ progress }}%</span>
          </div>
        </div>
      </section>
    </main>

    <div v-else class="loading">
      Loading questions...
    </div>
  </div>

  <div v-else class="complete-screen">
    <div class="complete-content">
      <h1>Exam Completed!</h1>
      <div class="checkmark">OK</div>
      <p class="thank-you">Thank you for completing the speaking test.</p>
      <p class="instructions">Please inform your teacher that you have finished.</p>
      <button @click="returnToStart" class="home-button">Return to Start</button>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import { useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import { useExamStore } from '../stores/exam'

const router = useRouter()
const examStore = useExamStore()

const isPlaying = ref(false)
const needsManualPlay = ref(false)
const mediaRecorder = ref(null)
const audioChunks = ref([])
const imageUrl = ref('')
const isAutoAdvancing = ref(false)
const audioLevel = ref(0)
const isInstructionActive = ref(false)
const isInstructionFading = ref(false)
const instructionTitle = ref('')
const instructionMessage = ref('')
const isPreparing = ref(false)
const introSequenceDone = ref(false)
const hasPlayedEndAudio = ref(false)
const flowLock = ref(false)
const audioPhase = ref('')
const lastSectionKey = ref('')

let recordingStartTime = 0
let activeAudio = null
let activeAudioUrl = null
let activeImageUrl = null
let isUnmounting = false
let activeStream = null
let meterAnimationId = null
let analyserNode = null
let analyserData = null
let analyserContext = null
let pendingAfterQuestionAudio = null

// Computed properties
const currentPart = computed(() => examStore.currentPart)
const currentSubPart = computed(() => examStore.currentSubPart)
const currentPartLabel = computed(() => examStore.currentPartLabel)
const currentQuestion = computed(() => examStore.currentQuestion)
const questions = computed(() => examStore.questions)
const currentQuestionData = computed(() => examStore.currentQuestionData)
const isRecording = computed(() => examStore.isRecording)
const timeRemaining = computed(() => examStore.timeRemaining)
const formattedTime = computed(() => examStore.formattedTime)
const progress = computed(() => examStore.progress)
const isFinished = computed(() => examStore.isFinished)

const PREP_SECONDS = 60
const INSTRUCTION_FADE_MS = 450

const AUDIO_FILES = {
  intro: 'intro.mp3',
  intro2: 'intro2.mp3',
  part1_1: 'part_1.1.mp3',
  part1_2: 'part_1.2.mp3',
  part2: 'part_2.mp3',
  part3: 'part_3.mp3',
  bell: 'bell_sound.mp3',
  end: 'end.mp3'
}

const SECTION_INSTRUCTIONS = {
  '1-1': {
    title: 'Part 1.1',
    message: 'Part 1.1. In this part, I will ask you a few questions about yourself. For each question, you will have 30 seconds to answer. You should speak after this sound.',
    audio: AUDIO_FILES.part1_1,
    bellAfter: true
  },
  '1-2': {
    title: 'Part 1.2',
    message: 'Part 1.2 You will now see two pictures. You will need to answer some questions based on these pictures. You will have 30 seconds to answer each question. Please speak after this sound.',
    audio: AUDIO_FILES.part1_2,
    bellAfter: true
  },
  '2-0': {
    title: 'Part 2',
    message: 'Part 2. In this part, you will be given a picture followed by one question. You do not need to describe the picture, but focus on the question provided. You will have one minute to prepare and two minutes to answer. You will hear this sound when the time for preparation is over.',
    audio: AUDIO_FILES.part2,
    bellAfter: true
  },
  '3-0': {
    title: 'Part 3',
    message: 'Part 3. You will now be given one discussion question related to Part 2. You will have one minute to prepare for the task and two minutes to speak. You will hear this sound when the time for preparation is over.',
    audio: AUDIO_FILES.part3,
    bellAfter: true
  }
}

const sectionKey = computed(() => `${currentPart.value}-${currentSubPart.value}`)
const needsPreparation = computed(() => {
  // Only show the prep timer once per section for Parts 2 and 3.
  return (currentPart.value === 2 || currentPart.value === 3) && currentQuestion.value === 0
})

const hasQuestionAudio = computed(() => {
  const audioPath = currentQuestionData.value?.audio_path
  return typeof audioPath === 'string' && audioPath.trim() !== ''
})

const hasQuestionImage = computed(() => {
  const imagePath = currentQuestionData.value?.image_path
  return typeof imagePath === 'string' && imagePath.trim() !== ''
})

const statusLabel = computed(() => {
  if (isPreparing.value) return 'Preparation time'
  if (isPlaying.value) return audioPhase.value === 'bell' ? 'Starting recording' : 'Listening to question'
  if (isRecording.value) return 'Recording response'
  return 'Ready'
})

const waveStyle = computed(() => {
  const level = isRecording.value ? audioLevel.value : 0
  const scale = (0.75 + level * 4.8).toFixed(2)
  const glow = `${14 + level * 84}px`
  return {
    '--wave-level': level.toFixed(3),
    '--wave-scale': scale,
    '--wave-glow': glow
  }
})

onMounted(async () => {
  console.log('Exam mounted, questions:', questions.value)
  console.log('Current question data:', currentQuestionData.value)
  await beginExamSequence()
})

onUnmounted(() => {
  isUnmounting = true
  cleanupAudio()
  cleanupImage()
  stopRecording()
  examStore.stopTimer()
  stopAudioMeter()
})

watch(currentQuestionData, async (value) => {
  if (!value || isFinished.value) return
  await handleQuestionChange()
}, { immediate: true })

watch(timeRemaining, async (value) => {
  if (value !== 0) return
  if (isPreparing.value) {
    await completePreparation()
    return
  }
  if (isRecording.value) {
    stopRecording()
  }
})

watch(isFinished, async (value) => {
  if (!value || hasPlayedEndAudio.value) return
  hasPlayedEndAudio.value = true
  await playSystemAudio(AUDIO_FILES.end, 'instruction')
})

async function beginExamSequence() {
  if (introSequenceDone.value || isUnmounting) return
  await runIntroSequence()
  introSequenceDone.value = true
  await handleQuestionChange()
}

async function handleQuestionChange() {
  if (flowLock.value || isUnmounting) return
  if (!introSequenceDone.value || !currentQuestionData.value || isFinished.value) return
  flowLock.value = true
  try {
    const key = sectionKey.value
    if (key && key !== lastSectionKey.value) {
      lastSectionKey.value = key
      await runSectionIntro(key)
    }
    await startQuestionFlow()
  } finally {
    flowLock.value = false
  }
}

async function runIntroSequence() {
  await showInstructionScreen({
    title: 'Multilevel Mock exam',
    message: 'Your exam starts in 10 seconds.',
    audioFiles: [AUDIO_FILES.intro, AUDIO_FILES.intro2]
  })
}

async function runSectionIntro(key) {
  const config = SECTION_INSTRUCTIONS[key]
  if (!config) return
  await showInstructionScreen({
    title: config.title,
    message: config.message,
    audioFiles: [config.audio],
    bellAfter: config.bellAfter
  })
}

async function showInstructionScreen({ title, message, audioFiles, bellAfter }) {
  if (isUnmounting) return
  instructionTitle.value = title
  instructionMessage.value = message
  isInstructionActive.value = true
  isInstructionFading.value = false
  examStore.stopTimer()

  if (Array.isArray(audioFiles)) {
    await playAudioSequence(audioFiles)
  }

  if (bellAfter) {
    await playSystemAudio(AUDIO_FILES.bell, 'bell')
  }

  await fadeInstructionOut()
}

async function fadeInstructionOut() {
  if (isUnmounting) return
  isInstructionFading.value = true
  await delay(INSTRUCTION_FADE_MS)
  isInstructionActive.value = false
  isInstructionFading.value = false
}

async function playAudioSequence(files) {
  for (const file of files) {
    if (isUnmounting) return
    await playSystemAudio(file, 'instruction')
  }
}

async function playSystemAudio(filename, phase) {
  if (!filename || isUnmounting) return false
  try {
    const audioResult = await loadAudioData(filename)
    if (!audioResult || !audioResult.data || audioResult.data.length === 0) {
      throw new Error('No audio data received from backend')
    }
    const audioArray = new Uint8Array(audioResult.data)
    const audioBlob = new Blob([audioArray], { type: resolveAudioType(audioResult.filename) })
    const audioUrl = URL.createObjectURL(audioBlob)

    cleanupAudio()
    activeAudioUrl = audioUrl
    activeAudio = new Audio(audioUrl)
    isPlaying.value = true
    audioPhase.value = phase || ''

    return await new Promise((resolve) => {
      activeAudio.onended = () => {
        isPlaying.value = false
        cleanupAudio()
        resolve(true)
      }
      activeAudio.onerror = () => {
        isPlaying.value = false
        cleanupAudio()
        resolve(false)
      }
      activeAudio.play().catch(() => {
        isPlaying.value = false
        cleanupAudio()
        resolve(false)
      })
    })
  } catch (error) {
    isPlaying.value = false
    cleanupAudio()
    console.warn('Failed to play audio:', filename, error)
    return false
  } finally {
    audioPhase.value = ''
  }
}

async function playBellAndRecord() {
  if (isUnmounting || isFinished.value) return
  await playSystemAudio(AUDIO_FILES.bell, 'bell')
  if (isUnmounting || isFinished.value) return
  await beginRecording()
}

async function startPreparation() {
  if (isUnmounting || isFinished.value) return
  isPreparing.value = true
  examStore.stopTimer()
  examStore.timeRemaining = PREP_SECONDS
  examStore.startTimer()
}

async function completePreparation() {
  if (!isPreparing.value || isUnmounting) return
  isPreparing.value = false
  examStore.stopTimer()
  await playBellAndRecord()
}

async function runAfterQuestionAudio() {
  const action = pendingAfterQuestionAudio
  pendingAfterQuestionAudio = null
  if (action) {
    await action()
  }
}

function delay(ms) {
  return new Promise(resolve => setTimeout(resolve, ms))
}

async function startQuestionFlow() {
  needsManualPlay.value = false
  isPreparing.value = false
  pendingAfterQuestionAudio = null
  cleanupAudio()
  cleanupImage()

  if (hasQuestionImage.value) {
    await loadQuestionImage(currentQuestionData.value.image_path)
  }

  if (needsPreparation.value) {
    pendingAfterQuestionAudio = async () => {
      await startPreparation()
    }
  } else {
    pendingAfterQuestionAudio = async () => {
      await playBellAndRecord()
    }
  }

  if (hasQuestionAudio.value) {
    const played = await playQuestionAudio(false)
    if (!played && !needsManualPlay.value && pendingAfterQuestionAudio) {
      await runAfterQuestionAudio()
    }
    return
  }

  if (pendingAfterQuestionAudio) {
    await runAfterQuestionAudio()
  }
}

async function loadQuestionImage(filename) {
  if (!filename || typeof filename !== 'string') return
  try {
    const imageData = await invoke('get_image_file', { filename: filename.trim() })
    const imageArray = new Uint8Array(imageData)
    const mimeType = resolveImageType(filename)
    const imageBlob = new Blob([imageArray], { type: mimeType })
    const newUrl = URL.createObjectURL(imageBlob)
    cleanupImage()
    activeImageUrl = newUrl
    imageUrl.value = newUrl
  } catch (error) {
    console.error('Failed to load image:', error)
    cleanupImage()
  }
}

function resolveImageType(filename) {
  const ext = filename.split('.').pop()?.toLowerCase()
  if (ext === 'png') return 'image/png'
  if (ext === 'webp') return 'image/webp'
  if (ext === 'gif') return 'image/gif'
  return 'image/jpeg'
}

function resolveAudioType(filename) {
  const ext = filename.split('.').pop()?.toLowerCase()
  if (ext === 'mp3') return 'audio/mpeg'
  if (ext === 'm4a') return 'audio/mp4'
  if (ext === 'ogg') return 'audio/ogg'
  if (ext === 'webm') return 'audio/webm'
  return 'audio/wav'
}

async function loadAudioData(filename) {
  const candidates = buildAudioCandidates(filename)
  let lastError = null
  for (const candidate of candidates) {
    try {
      const data = await invoke('get_audio_file', { filename: candidate })
      return { data, filename: candidate }
    } catch (error) {
      lastError = error
    }
  }
  throw lastError || new Error(`Failed to load audio: ${filename}`)
}

function buildAudioCandidates(filename) {
  if (!filename) return []
  if (hasAudioExtension(filename)) return [filename]
  const extensions = ['.wav', '.mp3', '.m4a', '.ogg', '.webm']
  const candidates = [filename, ...extensions.map(ext => `${filename}${ext}`)]
  return [...new Set(candidates)]
}

function hasAudioExtension(filename) {
  return /\.(wav|mp3|m4a|ogg|webm)$/i.test(filename)
}

async function playQuestionAudio(manual) {
  const audioPath = currentQuestionData.value?.audio_path
  if (!audioPath || typeof audioPath !== 'string' || audioPath.trim() === '' || isPlaying.value || isRecording.value) {
    return false
  }

  try {
    isPlaying.value = true
    audioPhase.value = 'question'
    const filename = audioPath.trim()
    const audioResult = await loadAudioData(filename)

    if (!audioResult || !audioResult.data || audioResult.data.length === 0) {
      throw new Error('No audio data received from backend')
    }

    const audioArray = new Uint8Array(audioResult.data)
    const audioBlob = new Blob([audioArray], { type: resolveAudioType(audioResult.filename) })
    const audioUrl = URL.createObjectURL(audioBlob)

    cleanupAudio()
    activeAudioUrl = audioUrl
    activeAudio = new Audio(audioUrl)

    activeAudio.onended = async () => {
      isPlaying.value = false
      cleanupAudio()
      await runAfterQuestionAudio()
    }

    activeAudio.onerror = async () => {
      isPlaying.value = false
      cleanupAudio()
      await runAfterQuestionAudio()
    }

    await activeAudio.play()
    needsManualPlay.value = false
    return true
  } catch (error) {
    isPlaying.value = false
    cleanupAudio()
    const errorName = error?.name || ''
    if (!manual && errorName === 'NotAllowedError') {
      needsManualPlay.value = true
      return false
    }
    await runAfterQuestionAudio()
    return false
  }
}

async function beginRecording() {
  if (!currentQuestionData.value || isRecording.value || isPlaying.value || isFinished.value || isPreparing.value || isInstructionActive.value) return
  await startRecording()
}

async function startRecording() {
  try {
    const stream = await navigator.mediaDevices.getUserMedia({
      audio: {
        echoCancellation: true,
        noiseSuppression: true,
        sampleRate: 44100
      }
    })

    activeStream = stream
    startAudioMeter(stream)

    mediaRecorder.value = new MediaRecorder(stream, {
      mimeType: 'audio/webm;codecs=opus'
    })

    audioChunks.value = []

    mediaRecorder.value.ondataavailable = (event) => {
      if (event.data.size > 0) {
        audioChunks.value.push(event.data)
      }
    }

    mediaRecorder.value.onstop = async () => {
      const audioBlob = new Blob(audioChunks.value, { type: 'audio/webm' })
      const duration = Math.round((Date.now() - recordingStartTime) / 1000)

      stopAudioMeter()
      await examStore.saveResponse(audioBlob, duration)
      stream.getTracks().forEach(track => track.stop())
      activeStream = null
      if (isUnmounting) return
      await advanceToNextQuestion()
    }

    const responseTime = currentQuestionData.value?.response_time || 30
    examStore.timeRemaining = responseTime
    mediaRecorder.value.start()
    examStore.startRecording()
    examStore.startTimer()
    recordingStartTime = Date.now()
  } catch (error) {
    console.error('Recording error:', error)
    examStore.stopRecording()
    examStore.stopTimer()
    stopAudioMeter()
    alert('Could not access microphone. Please check permissions.')
  }
}

function stopRecording() {
  if (mediaRecorder.value && mediaRecorder.value.state !== 'inactive') {
    mediaRecorder.value.stop()
    examStore.stopRecording()
    examStore.stopTimer()
    stopAudioMeter()
    if (activeStream) {
      activeStream.getTracks().forEach(track => track.stop())
      activeStream = null
    }
  }
}

async function advanceToNextQuestion() {
  if (isAutoAdvancing.value || isFinished.value) return
  isAutoAdvancing.value = true
  await examStore.nextQuestion()
  isAutoAdvancing.value = false
}

function cleanupAudio() {
  if (activeAudio) {
    activeAudio.pause()
    activeAudio.src = ''
    activeAudio = null
  }
  if (activeAudioUrl) {
    URL.revokeObjectURL(activeAudioUrl)
    activeAudioUrl = null
  }
  isPlaying.value = false
  audioPhase.value = ''
}

function cleanupImage() {
  if (activeImageUrl) {
    URL.revokeObjectURL(activeImageUrl)
    activeImageUrl = null
  }
  imageUrl.value = ''
}

function startAudioMeter(stream) {
  stopAudioMeter()
  analyserContext = new (window.AudioContext || window.webkitAudioContext)()
  analyserContext.resume().catch(() => {})
  const sourceNode = analyserContext.createMediaStreamSource(stream)
  analyserNode = analyserContext.createAnalyser()
  analyserNode.fftSize = 512
  analyserData = new Uint8Array(analyserNode.fftSize)
  sourceNode.connect(analyserNode)

  const updateLevel = () => {
    if (!analyserNode) return
    analyserNode.getByteTimeDomainData(analyserData)
    let sum = 0
    for (let i = 0; i < analyserData.length; i += 1) {
      const value = (analyserData[i] - 128) / 128
      sum += value * value
    }
    const rms = Math.sqrt(sum / analyserData.length)
    audioLevel.value = Math.min(1, Math.max(rms * 9.2, audioLevel.value * 0.7))
    meterAnimationId = requestAnimationFrame(updateLevel)
  }

  updateLevel()
}

function stopAudioMeter() {
  if (meterAnimationId) {
    cancelAnimationFrame(meterAnimationId)
    meterAnimationId = null
  }
  if (analyserContext) {
    analyserContext.close()
    analyserContext = null
  }
  analyserNode = null
  analyserData = null
  audioLevel.value = 0
}

function returnToStart() {
  examStore.resetExam()
  router.push('/')
}
</script>

<style scoped>
.exam-page {
  min-height: 100vh;
  background: radial-gradient(circle at top, rgba(252, 231, 156, 0.2), transparent 55%),
    linear-gradient(145deg, #faf5ef 0%, #f3efe7 50%, #efe7dc 100%);
  padding: 24px;
  position: relative;
  overflow: hidden;
}

.ambient-bg {
  position: absolute;
  inset: 0;
  background:
    radial-gradient(circle at 15% 20%, rgba(34, 81, 128, 0.08), transparent 40%),
    radial-gradient(circle at 85% 15%, rgba(245, 158, 11, 0.12), transparent 45%),
    radial-gradient(circle at 80% 85%, rgba(16, 185, 129, 0.1), transparent 40%);
  pointer-events: none;
  z-index: 0;
}

.instruction-overlay {
  position: fixed;
  inset: 0;
  display: grid;
  place-items: center;
  background:
    radial-gradient(circle at 18% 20%, rgba(59, 130, 246, 0.18), transparent 55%),
    radial-gradient(circle at 80% 75%, rgba(249, 115, 22, 0.2), transparent 55%),
    linear-gradient(135deg, rgba(255, 255, 255, 0.96), rgba(248, 250, 252, 0.95));
  backdrop-filter: blur(6px);
  z-index: 5;
  opacity: 1;
  transition: opacity 0.45s ease, transform 0.45s ease;
}

.instruction-overlay.fade-out {
  opacity: 0;
  transform: scale(0.98);
  pointer-events: none;
}

.instruction-panel {
  background: rgba(255, 255, 255, 0.94);
  border-radius: 28px;
  padding: 48px 52px;
  text-align: center;
  max-width: 720px;
  margin: 24px;
  border: 1px solid rgba(148, 163, 184, 0.25);
  box-shadow: 0 30px 90px rgba(15, 23, 42, 0.18);
  animation: floatIn 0.6s ease;
}

.instruction-tag {
  font-size: 12px;
  letter-spacing: 0.3em;
  text-transform: uppercase;
  color: #1d4ed8;
  font-weight: 700;
}

.instruction-panel h2 {
  font-family: 'Fraunces', serif;
  font-size: 32px;
  margin: 16px 0 14px;
  color: #0f172a;
}

.instruction-panel p {
  color: #475569;
  font-size: 16px;
  line-height: 1.7;
}

.exam-topbar {
  position: relative;
  z-index: 1;
  display: grid;
  grid-template-columns: 1fr auto auto;
  align-items: center;
  gap: 16px;
  padding: 16px 24px;
  border-radius: 18px;
  background: rgba(255, 255, 255, 0.85);
  border: 1px solid rgba(17, 24, 39, 0.08);
  box-shadow: 0 20px 60px rgba(15, 23, 42, 0.08);
  backdrop-filter: blur(10px);
}

.brand {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.brand-title {
  font-family: 'Fraunces', serif;
  font-size: 22px;
  font-weight: 600;
  color: #0f172a;
}

.brand-sub {
  font-size: 13px;
  letter-spacing: 0.2em;
  text-transform: uppercase;
  color: #64748b;
}

.top-meta {
  display: flex;
  gap: 10px;
  justify-content: center;
  flex-wrap: wrap;
}

.chip {
  padding: 6px 14px;
  border-radius: 999px;
  font-size: 13px;
  font-weight: 600;
  letter-spacing: 0.02em;
  background: #0f172a;
  color: #f8fafc;
}

.part-chip {
  background: #f97316;
}

.question-chip {
  background: #1d4ed8;
}

.timer {
  font-family: 'Space Grotesk', sans-serif;
  font-size: 22px;
  font-weight: 700;
  color: #0f172a;
  padding: 10px 18px;
  border-radius: 12px;
  background: #fef3c7;
  border: 1px solid rgba(124, 45, 18, 0.1);
}

.timer.warning {
  background: #fee2e2;
  color: #b91c1c;
  animation: pulse 1s infinite;
}

@keyframes pulse {
  0%, 100% { transform: scale(1); }
  50% { transform: scale(1.02); }
}

.progress-container {
  position: relative;
  z-index: 1;
  height: 10px;
  background: rgba(15, 23, 42, 0.1);
  border-radius: 999px;
  margin: 26px 0 34px;
  overflow: hidden;
}

.progress-bar {
  height: 100%;
  background: linear-gradient(90deg, #22c55e, #16a34a);
  transition: width 0.3s ease;
}

.progress-text {
  position: absolute;
  top: 14px;
  left: 0;
  right: 0;
  text-align: center;
  font-size: 13px;
  font-weight: 600;
  color: #475569;
}

.exam-layout {
  position: relative;
  z-index: 1;
  display: grid;
  grid-template-columns: minmax(0, 2fr) minmax(260px, 1fr);
  gap: 24px;
  align-items: start;
}

.question-card,
.info-card {
  background: rgba(255, 255, 255, 0.92);
  border-radius: 24px;
  padding: 32px;
  box-shadow: 0 24px 70px rgba(15, 23, 42, 0.12);
  border: 1px solid rgba(148, 163, 184, 0.2);
  animation: floatIn 0.6s ease;
}

@keyframes floatIn {
  from { opacity: 0; transform: translateY(10px); }
  to { opacity: 1; transform: translateY(0); }
}

.status-row {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
  margin-bottom: 24px;
  align-items: center;
}

.question-text {
  font-size: 18px;
  font-weight: 600;
  color: #0f172a;
  margin-bottom: 18px;
  line-height: 1.5;
}

.status-pill {
  padding: 8px 14px;
  border-radius: 999px;
  background: #f1f5f9;
  font-weight: 600;
  font-size: 13px;
  color: #0f172a;
}

.status-pill.playing {
  background: #e0f2fe;
  color: #0369a1;
}

.status-pill.recording {
  background: #fee2e2;
  color: #b91c1c;
}

.info-pill {
  padding: 6px 12px;
  border-radius: 999px;
  background: #fef3c7;
  color: #92400e;
  font-size: 12px;
  font-weight: 600;
}

.image-frame {
  width: 100%;
  border-radius: 18px;
  overflow: hidden;
  margin-bottom: 24px;
  border: 1px solid rgba(15, 23, 42, 0.08);
  background: #fff7ed;
}

.image-frame img {
  width: 100%;
  display: block;
  object-fit: cover;
}

.audio-player {
  display: flex;
  flex-direction: column;
  gap: 12px;
  align-items: flex-start;
}

.play-button {
  background: #0f172a;
  color: #f8fafc;
  border: none;
  padding: 14px 28px;
  border-radius: 14px;
  font-size: 15px;
  font-weight: 600;
  cursor: pointer;
  transition: transform 0.2s ease, background 0.2s ease;
}

.play-button:hover:not(:disabled) {
  background: #1f2937;
  transform: translateY(-2px);
}

.play-button:disabled {
  background: #94a3b8;
  cursor: not-allowed;
}

.helper-text {
  color: #64748b;
  font-size: 13px;
}

.no-audio-message {
  background: #fef3c7;
  border-radius: 12px;
  padding: 16px 18px;
  color: #92400e;
  font-weight: 600;
}

.voice-visual {
  display: flex;
  align-items: center;
  gap: 16px;
  margin-top: 24px;
  padding: 16px 20px;
  background: rgba(15, 23, 42, 0.04);
  border-radius: 18px;
  border: 1px solid rgba(148, 163, 184, 0.4);
  font-weight: 600;
  color: #475569;
}

.voice-visual.active {
  background: rgba(29, 78, 216, 0.08);
  border-color: rgba(29, 78, 216, 0.35);
  color: #1e3a8a;
}

.voice-waves {
  width: 220px;
  height: 72px;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 8px 4px;
  position: relative;
  border-radius: 999px;
  overflow: hidden;
}

.voice-waves::before {
  content: '';
  position: absolute;
  inset: 10% 6%;
  border-radius: 999px;
  background:
    radial-gradient(circle at 10% 50%, rgba(56, 189, 248, 0.15), transparent 60%),
    radial-gradient(circle at 90% 50%, rgba(249, 115, 22, 0.12), transparent 60%),
    linear-gradient(90deg, rgba(15, 23, 42, 0.08), rgba(15, 23, 42, 0.02), rgba(15, 23, 42, 0.08));
  filter: blur(6px);
  opacity: 0.9;
}

.voice-waves::after {
  content: '';
  position: absolute;
  inset: 18% 12%;
  border-radius: 999px;
  border: 1px solid rgba(148, 163, 184, 0.18);
  opacity: 0.6;
}

.wave-svg {
  width: 100%;
  height: 100%;
  transform-origin: center;
  transform: scaleY(var(--wave-scale));
  transition: transform 0.08s ease-out, filter 0.2s ease;
  filter: drop-shadow(0 0 var(--wave-glow) rgba(59, 130, 246, 0.4));
  position: relative;
  z-index: 1;
}

.wave-base {
  fill: none;
  stroke: rgba(148, 163, 184, 0.45);
  stroke-width: 1.2;
  opacity: 0.5;
}

.wave {
  fill: none;
  stroke: url(#waveGradient);
  stroke-width: 2.4;
  stroke-linecap: round;
  opacity: 0.9;
  filter: url(#waveGlow);
  transform-origin: center;
  animation: waveFloat 3.6s ease-in-out infinite;
}

.wave-2 {
  stroke-width: 2.1;
  opacity: 0.7;
  animation-duration: 4.2s;
}

.wave-3 {
  stroke-width: 1.8;
  opacity: 0.5;
  animation-duration: 5s;
}

.wave-4 {
  stroke-width: 1.6;
  opacity: 0.35;
  animation-duration: 5.6s;
}

.wave-5 {
  stroke: url(#waveAccent);
  stroke-width: 1.4;
  opacity: 0.55;
  stroke-dasharray: 10 16;
  animation: waveFloat 4.8s ease-in-out infinite, waveDash 6s linear infinite;
}

.wave-6 {
  stroke: rgba(224, 231, 255, 0.75);
  stroke-width: 1.1;
  opacity: 0.5;
  stroke-dasharray: 4 10;
  animation: waveFloat 6.4s ease-in-out infinite, waveDash 7.4s linear infinite reverse;
}

.voice-visual.active .wave-svg {
  filter: drop-shadow(0 0 var(--wave-glow) rgba(99, 102, 241, 0.55));
}

@keyframes waveFloat {
  0%, 100% { transform: translateX(0); }
  50% { transform: translateX(-18px); }
}

@keyframes waveDash {
  to { stroke-dashoffset: -120; }
}

.auto-note {
  margin-top: 20px;
  color: #64748b;
  font-weight: 600;
}

.info-card h3 {
  font-family: 'Fraunces', serif;
  font-size: 20px;
  margin-bottom: 16px;
  color: #0f172a;
}

.info-card p {
  color: #475569;
  font-size: 14px;
  line-height: 1.6;
  margin-bottom: 20px;
}

.info-grid {
  display: grid;
  gap: 14px;
}

.info-item {
  padding: 12px 14px;
  border-radius: 12px;
  background: #f8fafc;
  border: 1px solid rgba(148, 163, 184, 0.2);
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 13px;
}

.info-label {
  color: #64748b;
  font-weight: 600;
}

.info-value {
  color: #0f172a;
  font-weight: 700;
}

.loading {
  text-align: center;
  padding: 60px;
  color: #64748b;
  font-size: 18px;
}

.complete-screen {
  min-height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, #fb7185 0%, #f97316 45%, #f59e0b 100%);
  padding: 24px;
}

.complete-content {
  background: white;
  border-radius: 28px;
  padding: 60px;
  text-align: center;
  max-width: 520px;
  box-shadow: 0 30px 90px rgba(0, 0, 0, 0.2);
}

.complete-content h1 {
  font-family: 'Fraunces', serif;
  font-size: 36px;
  color: #0f172a;
  margin-bottom: 24px;
}

.checkmark {
  font-size: 72px;
  color: #16a34a;
  margin: 20px 0;
}

.thank-you {
  font-size: 18px;
  color: #1f2937;
  margin-bottom: 12px;
}

.complete-content .instructions {
  color: #475569;
  margin-bottom: 32px;
  font-size: 15px;
}

.home-button {
  background: #0f172a;
  color: white;
  border: none;
  padding: 14px 36px;
  border-radius: 14px;
  font-size: 16px;
  font-weight: 600;
  cursor: pointer;
  transition: transform 0.2s ease, background 0.2s ease;
}

.home-button:hover {
  background: #1f2937;
  transform: translateY(-2px);
}

@media (max-width: 960px) {
  .exam-topbar {
    grid-template-columns: 1fr;
    text-align: center;
  }

  .top-meta {
    justify-content: center;
  }

  .timer {
    justify-self: center;
  }

  .exam-layout {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 600px) {
  .exam-page {
    padding: 16px;
  }

  .exam-topbar {
    padding: 16px;
  }

  .question-card,
  .info-card {
    padding: 22px;
  }

  .complete-content {
    padding: 40px 28px;
  }

  .instruction-panel {
    padding: 32px 24px;
  }

  .instruction-panel h2 {
    font-size: 26px;
  }

  .voice-waves {
    width: 170px;
    height: 60px;
  }
}
</style>
