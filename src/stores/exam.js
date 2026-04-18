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
  const startMode = ref('random')
  const selectedQuestionsBySection = ref({})

  const defaultSections = [
    { part: 1, subPart: 1, count: 3, label: '1.1', key: '1-1' },
    { part: 1, subPart: 2, count: 3, label: '1.2', key: '1-2' },
    { part: 2, subPart: 0, count: 1, label: '2', key: '2-0' },
    { part: 3, subPart: 0, count: 1, label: '3', key: '3-0' }
  ]
  const sections = ref([...defaultSections])
  const sectionCounts = ref({})

  function getSectionKey(part, subPart) {
    return `${part}-${subPart || 0}`
  }

  function setSectionCount(part, subPart, count) {
    const key = getSectionKey(part, subPart)
    sectionCounts.value = { ...sectionCounts.value, [key]: count }
  }

  function resetFlowState(name) {
    studentName.value = name
    currentSectionIndex.value = 0
    currentQuestion.value = 0
    questions.value = []
    isRecording.value = false
    timeRemaining.value = 0
    isFinished.value = false
    sectionCounts.value = {}
    stopTimer()
  }

  function sortQuestionsForExam(questionList) {
    return [...questionList].sort((a, b) => {
      const orderA = Number(a.pack_order) || Number.MAX_SAFE_INTEGER
      const orderB = Number(b.pack_order) || Number.MAX_SAFE_INTEGER

      if (orderA !== orderB) return orderA - orderB
      return a.id - b.id
    })
  }

  function buildSelectedSections(selectedQuestions) {
    const grouped = {}

    selectedQuestions.forEach((question) => {
      const key = getSectionKey(question.part, question.sub_part)
      grouped[key] = grouped[key] || []
      grouped[key].push(question)
    })

    const selectedSections = defaultSections.filter((section) => grouped[section.key]?.length)

    return {
      grouped: Object.fromEntries(
        Object.entries(grouped).map(([key, value]) => [key, sortQuestionsForExam(value)])
      ),
      selectedSections
    }
  }
  
  // Actions
  async function startExam(name) {
    try {
      // Create attempt in database
      const id = await invoke('create_attempt', { studentName: name })
      attemptId.value = id
      startMode.value = 'random'
      selectedQuestionsBySection.value = {}
      sections.value = [...defaultSections]
      resetFlowState(name)

      // Load questions for part 1.1
      const firstSection = sections.value[0]
      const part1Questions = await invoke('get_random_questions', {
        part: firstSection.part,
        subPart: firstSection.subPart,
        count: firstSection.count,
        excludeIds: []
      })

      questions.value = part1Questions
      timeRemaining.value = part1Questions[0]?.response_time || 30
      setSectionCount(firstSection.part, firstSection.subPart, part1Questions.length)
      
    } catch (error) {
      console.error('Failed to start exam:', error)
      throw error
    }
  }

  async function startSelectedExam(name, selectedQuestions) {
    if (!Array.isArray(selectedQuestions) || selectedQuestions.length === 0) {
      throw new Error('Choose at least one question for the selected-question mode.')
    }

    const { grouped, selectedSections } = buildSelectedSections(selectedQuestions)

    if (selectedSections.length === 0) {
      throw new Error('Choose at least one valid question for the selected-question mode.')
    }

    try {
      const id = await invoke('create_attempt', { studentName: name })
      attemptId.value = id
      startMode.value = 'selected'
      selectedQuestionsBySection.value = grouped
      sections.value = selectedSections
      resetFlowState(name)

      const firstSection = sections.value[0]
      await loadPartQuestions(firstSection.part, grouped[firstSection.key].length, firstSection.subPart)
    } catch (error) {
      console.error('Failed to start selected exam:', error)
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
      if (currentSectionIndex.value < sections.value.length - 1) {
        currentSectionIndex.value += 1
        const nextSection = sections.value[currentSectionIndex.value]
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
    const key = getSectionKey(part, subPart)

    if (startMode.value === 'selected') {
      const selectedQuestions = selectedQuestionsBySection.value[key] || []

      questions.value = selectedQuestions
      timeRemaining.value = selectedQuestions[0]?.response_time || 30
      setSectionCount(part, subPart, selectedQuestions.length)
      return
    }

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
    startMode.value = 'random'
    selectedQuestionsBySection.value = {}
    sections.value = [...defaultSections]
    sectionCounts.value = {}
  }
  
  // Getters
  const currentQuestionData = computed(() => {
    return questions.value[currentQuestion.value] || null
  })
  
  const currentPart = computed(() => sections.value[currentSectionIndex.value]?.part || 1)
  const currentSubPart = computed(() => sections.value[currentSectionIndex.value]?.subPart || 0)
  const currentPartLabel = computed(() => `Part ${sections.value[currentSectionIndex.value]?.label || '1'}`)

  function getProgressSectionCount(section) {
    if (startMode.value === 'selected') {
      return selectedQuestionsBySection.value[section.key]?.length || 0
    }

    return sectionCounts.value[section.key] ?? section.count
  }

  const progress = computed(() => {
    const totalQuestions = sections.value.reduce((sum, section) => {
      return sum + getProgressSectionCount(section)
    }, 0)
    if (totalQuestions === 0) return 0

    const answeredBefore = sections.value
      .slice(0, currentSectionIndex.value)
      .reduce((sum, section) => {
        return sum + getProgressSectionCount(section)
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
    startSelectedExam,
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
