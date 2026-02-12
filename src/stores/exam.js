import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export const useExamStore = defineStore('exam', () => {
  // State
  const attemptId = ref(null)
  const studentName = ref('')
  const currentSectionIndex = ref(0)
  const currentQuestion = ref(0)
  const questions = ref([])
  const isRecording = ref(false)
  const timeRemaining = ref(0)
  const isFinished = ref(false)
  const timerInterval = ref(null)

  const sections = [
    { part: 1, subPart: 1, count: 3, label: '1.1', key: '1-1' },
    { part: 1, subPart: 2, count: 3, label: '1.2', key: '1-2' },
    { part: 2, subPart: 0, count: 1, label: '2', key: '2-0' },
    { part: 3, subPart: 0, count: 1, label: '3', key: '3-0' }
  ]
  const sectionCounts = ref({})

  function getSectionKey(part, subPart) {
    return `${part}-${subPart || 0}`
  }

  function setSectionCount(part, subPart, count) {
    const key = getSectionKey(part, subPart)
    sectionCounts.value = { ...sectionCounts.value, [key]: count }
  }
  
  // Actions
  async function startExam(name) {
    try {
      // Create attempt in database
      const id = await invoke('create_attempt', { studentName: name })
      attemptId.value = id
      studentName.value = name

      currentSectionIndex.value = 0
      currentQuestion.value = 0

      // Load questions for part 1.1
      const firstSection = sections[0]
      const part1Questions = await invoke('get_random_questions', {
        part: firstSection.part,
        subPart: firstSection.subPart,
        count: firstSection.count,
        excludeIds: []
      })

      questions.value = part1Questions
      timeRemaining.value = part1Questions[0]?.response_time || 30
      setSectionCount(firstSection.part, firstSection.subPart, part1Questions.length)
      stopTimer()
      
    } catch (error) {
      console.error('Failed to start exam:', error)
      throw error
    }
  }
  
  function startTimer() {
    if (timerInterval.value) {
      clearInterval(timerInterval.value)
    }
    
    timerInterval.value = setInterval(() => {
      if (timeRemaining.value > 0) {
        timeRemaining.value--
      } else {
        clearInterval(timerInterval.value)
        timerInterval.value = null
      }
    }, 1000)
  }

  function stopTimer() {
    if (timerInterval.value) {
      clearInterval(timerInterval.value)
      timerInterval.value = null
    }
  }
  
  async function nextQuestion() {
    const nextIndex = currentQuestion.value + 1
    
    if (nextIndex >= questions.value.length) {
      // Move to next part
      if (currentSectionIndex.value < sections.length - 1) {
        currentSectionIndex.value += 1
        const nextSection = sections[currentSectionIndex.value]
        await loadPartQuestions(nextSection.part, nextSection.count, nextSection.subPart)
        currentQuestion.value = 0
      } else {
        await finishExam()
        isFinished.value = true
        stopTimer()
      }
    } else {
      currentQuestion.value = nextIndex
      timeRemaining.value = questions.value[nextIndex]?.response_time || 30
    }
    
    stopTimer()
  }
  
  async function loadPartQuestions(part, count, subPart = 0) {
    const excludeIds = questions.value.map(q => q.id)
    
    const newQuestions = await invoke('get_random_questions', {
      part,
      subPart: subPart || undefined,
      count,
      excludeIds
    })
    
    questions.value = newQuestions
    timeRemaining.value = newQuestions[0]?.response_time || 30
    setSectionCount(part, subPart, newQuestions.length)
  }
  
  async function finishExam() {
    if (attemptId.value) {
      await invoke('finish_attempt', { attemptId: attemptId.value })
    }
  }
  
  async function saveResponse(audioBlob, duration) {
    if (!attemptId.value || !questions.value[currentQuestion.value]) return
    
    try {
      // Convert blob to array
      const audioArrayBuffer = await audioBlob.arrayBuffer()
      const audioData = Array.from(new Uint8Array(audioArrayBuffer))
      
      await invoke('save_response', {
        attemptId: attemptId.value,
        questionId: questions.value[currentQuestion.value].id,
        audioData: audioData,
        duration: Math.round(duration)
      })
      
      return true
    } catch (error) {
      console.error('Failed to save response:', error)
      return false
    }
  }
  
  function startRecording() {
    isRecording.value = true
  }
  
  function stopRecording() {
    isRecording.value = false
  }
  
  function resetExam() {
    stopTimer()
    
    attemptId.value = null
    studentName.value = ''
    currentSectionIndex.value = 0
    currentQuestion.value = 0
    questions.value = []
    isRecording.value = false
    timeRemaining.value = 0
    isFinished.value = false
    sectionCounts.value = {}
  }
  
  // Getters
  const currentQuestionData = computed(() => {
    return questions.value[currentQuestion.value] || null
  })
  
  const currentPart = computed(() => sections[currentSectionIndex.value]?.part || 1)
  const currentSubPart = computed(() => sections[currentSectionIndex.value]?.subPart || 0)
  const currentPartLabel = computed(() => `Part ${sections[currentSectionIndex.value]?.label || '1'}`)

  const progress = computed(() => {
    const totalQuestions = sections.reduce((sum, section) => {
      const count = sectionCounts.value[section.key] ?? section.count
      return sum + count
    }, 0)
    const answeredBefore = sections
      .slice(0, currentSectionIndex.value)
      .reduce((sum, section) => {
        const count = sectionCounts.value[section.key] ?? section.count
        return sum + count
      }, 0)
    const answered = answeredBefore + currentQuestion.value

    return Math.round((answered / totalQuestions) * 100)
  })
  
  const formattedTime = computed(() => {
    const minutes = Math.floor(timeRemaining.value / 60)
    const seconds = timeRemaining.value % 60
    return `${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}`
  })
  
  return {
    // State
    attemptId,
    studentName,
    currentPart,
    currentSubPart,
    currentPartLabel,
    currentQuestion,
    questions,
    isRecording,
    timeRemaining,
    isFinished,
    
    // Actions
    startExam,
    nextQuestion,
    saveResponse,
    startRecording,
    stopRecording,
    resetExam,
    finishExam,
    startTimer,
    stopTimer,
    
    // Getters
    currentQuestionData,
    progress,
    formattedTime
  }
})
