<template>
  <div class="exam-page" v-if="!isFinished">
    <div class="ambient-bg"></div>
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
            @click="playQuestionAudio(true)"
            class="play-button"
            :disabled="isPlaying || isRecording"
          >
            {{ isPlaying ? 'Playing...' : needsManualPlay ? 'Play Question Audio' : 'Replay Audio' }}
          </button>
          <p class="helper-text" v-if="needsManualPlay">
            Tap play to begin. Recording starts automatically after the audio ends.
          </p>
          <p class="helper-text" v-else>
            Recording begins as soon as the audio ends.
          </p>
        </div>

        <div v-else class="no-audio-message">
          No audio prompt. Recording begins automatically.
        </div>

        <div class="voice-visual" :class="{ active: isRecording }">
          <div class="voice-orb" aria-hidden="true" :style="orbStyle">
            <span class="orb-core"></span>
            <span class="orb-ring"></span>
            <span class="orb-ring ring-2"></span>
            <svg class="orb-icon" viewBox="0 0 24 24" aria-hidden="true">
              <path
                d="M12 14.5a2.5 2.5 0 0 0 2.5-2.5V7a2.5 2.5 0 0 0-5 0v5a2.5 2.5 0 0 0 2.5 2.5Zm4.5-2.5a4.5 4.5 0 0 1-9 0M12 19v-2m-4 2h8"
                fill="none"
                stroke="currentColor"
                stroke-width="1.6"
                stroke-linecap="round"
                stroke-linejoin="round"
              />
            </svg>
          </div>
          <span>{{ isRecording ? 'Recording your response' : 'Ready to record' }}</span>
        </div>
        <div class="auto-note" v-if="!isPlaying && !isRecording">
          Recording starts automatically.
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
          Part 2: Speak for 1-2 minutes on the topic.
        </p>
        <p v-else>
          Part 3: Discussion related to the Part 2 topic.
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

const hasQuestionAudio = computed(() => {
  const audioPath = currentQuestionData.value?.audio_path
  return typeof audioPath === 'string' && audioPath.trim() !== ''
})

const hasQuestionImage = computed(() => {
  const imagePath = currentQuestionData.value?.image_path
  return typeof imagePath === 'string' && imagePath.trim() !== ''
})

const statusLabel = computed(() => {
  if (isPlaying.value) return 'Listening to question'
  if (isRecording.value) return 'Recording response'
  return 'Ready'
})

const orbStyle = computed(() => {
  const level = isRecording.value ? audioLevel.value : 0
  const scale = (0.82 + level * 0.75).toFixed(2)
  const glow = `${10 + level * 36}px`
  return {
    '--orb-level': level.toFixed(3),
    '--orb-scale': scale,
    '--orb-glow': glow
  }
})

onMounted(() => {
  console.log('Exam mounted, questions:', questions.value)
  console.log('Current question data:', currentQuestionData.value)
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
  await startQuestionFlow()
}, { immediate: true })

watch(timeRemaining, (value) => {
  if (value === 0 && isRecording.value) {
    stopRecording()
  }
})

async function startQuestionFlow() {
  needsManualPlay.value = false
  cleanupAudio()
  cleanupImage()

  if (hasQuestionImage.value) {
    await loadQuestionImage(currentQuestionData.value.image_path)
  }

  if (hasQuestionAudio.value) {
    const played = await playQuestionAudio(false)
    if (!played && !needsManualPlay.value) {
      await beginRecording()
    }
  } else {
    await beginRecording()
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

async function playQuestionAudio(manual) {
  const audioPath = currentQuestionData.value?.audio_path
  if (!audioPath || typeof audioPath !== 'string' || audioPath.trim() === '' || isPlaying.value || isRecording.value) {
    return false
  }

  try {
    isPlaying.value = true
    const filename = audioPath.trim()
    const audioData = await invoke('get_audio_file', { filename })

    if (!audioData || audioData.length === 0) {
      throw new Error('No audio data received from backend')
    }

    const audioArray = new Uint8Array(audioData)
    const audioBlob = new Blob([audioArray], { type: 'audio/wav' })
    const audioUrl = URL.createObjectURL(audioBlob)

    cleanupAudio()
    activeAudioUrl = audioUrl
    activeAudio = new Audio(audioUrl)

    activeAudio.onended = async () => {
      isPlaying.value = false
      cleanupAudio()
      await beginRecording()
    }

    activeAudio.onerror = async () => {
      isPlaying.value = false
      cleanupAudio()
      await beginRecording()
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
    await beginRecording()
    return false
  }
}

async function beginRecording() {
  if (!currentQuestionData.value || isRecording.value || isPlaying.value || isFinished.value) return
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
    audioLevel.value = Math.min(1, Math.max(rms * 3.2, audioLevel.value * 0.7))
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

.voice-orb {
  position: relative;
  width: 84px;
  height: 84px;
  display: grid;
  place-items: center;
  background: #0f172a;
  border-radius: 50%;
  box-shadow: 0 0 0 12px rgba(15, 23, 42, 0.35);
}

.orb-core {
  position: absolute;
  width: 40px;
  height: 40px;
  background: radial-gradient(circle at 30% 30%, #e2e8f0, #94a3b8 55%, #64748b 100%);
  border-radius: 50%;
  opacity: 0.25;
  transition: opacity 0.2s ease, transform 0.2s ease;
}

.orb-ring {
  position: absolute;
  width: 70px;
  height: 70px;
  border-radius: 50%;
  border: 2px solid rgba(148, 163, 184, 0.55);
  transform: scale(var(--orb-scale));
  transition: border-color 0.2s ease, transform 0.08s ease-out, box-shadow 0.2s ease;
}

.orb-ring.ring-2 {
  width: 86px;
  height: 86px;
  border-color: rgba(148, 163, 184, 0.25);
  transform: scale(calc(var(--orb-scale) * 1.12));
}

.orb-icon {
  width: 26px;
  height: 26px;
  color: rgba(226, 232, 240, 0.75);
}

.voice-visual.active .orb-core {
  opacity: 0.85;
  animation: morph 2.6s ease-in-out infinite;
  background: radial-gradient(circle at 30% 30%, #dbeafe, #60a5fa 60%, #1d4ed8 100%);
  box-shadow: 0 0 var(--orb-glow) rgba(59, 130, 246, 0.45);
}

.voice-visual.active .orb-ring {
  border-color: rgba(59, 130, 246, 0.75);
  box-shadow: 0 0 var(--orb-glow) rgba(59, 130, 246, 0.35);
}

.voice-visual.active .orb-ring.ring-2 {
  border-color: rgba(59, 130, 246, 0.35);
}

.voice-visual.active .orb-icon {
  color: #ffffff;
}

.auto-note {
  margin-top: 20px;
  color: #64748b;
  font-weight: 600;
}

@keyframes morph {
  0% { border-radius: 50% 60% 40% 50% / 55% 45% 55% 45%; transform: scale(1); }
  50% { border-radius: 55% 45% 60% 40% / 45% 55% 45% 55%; transform: scale(1.05); }
  100% { border-radius: 50% 60% 40% 50% / 55% 45% 55% 45%; transform: scale(1); }
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
}
</style>
