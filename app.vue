<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import {
  Shield, Delete, UploadCloud, FileCode, FolderOpen, Loader2,
  Terminal, Copy, LogOut, List, Trash2, ExternalLink, RefreshCw, Edit
} from 'lucide-vue-next'
import CyberAlert from '~/components/CyberAlert.vue'

const isAuthenticated = ref(false)
const pinInput = ref('')
const pinError = ref(false)
const checkingPin = ref(false)
const maxPinLength = 4

const currentView = ref<'upload' | 'manage'>('upload')
const loading = ref(false)
const fileName = ref('')
const fileContent = ref('')
const uploadResult = ref<string | null>(null)
const fileInput = ref<HTMLInputElement | null>(null)
const copied = ref(false)
const editingId = ref<string | null>(null)

const files = ref<Array<{ id: string, key: string, ttl: number }>>([])
const loadingFiles = ref(false)
const fileToDelete = ref<string | null>(null)
const selectedTtl = ref<number>(0) // 0 = permanent

const alert = ref({ show: false, title: '', message: '', type: 'info' as any })

const handlePinInput = (num: string) => {
  if (pinInput.value.length < maxPinLength && !checkingPin.value) {
    pinInput.value += num
    pinError.value = false
  }
}

const handleBackspace = () => {
  if (!checkingPin.value) {
    pinInput.value = pinInput.value.slice(0, -1)
    pinError.value = false
  }
}

const checkUnlock = async () => {
  if (pinInput.value.length === 0) return
  checkingPin.value = true
  pinError.value = false
  try {
    await $fetch('/api/verify', { method: 'POST', body: { pin: pinInput.value } })
    isAuthenticated.value = true
    fetchFiles()
  } catch (e) {
    pinError.value = true
    pinInput.value = ''
    setTimeout(() => pinError.value = false, 500)
  } finally {
    checkingPin.value = false
  }
}

const handleLogout = () => {
  isAuthenticated.value = false
  pinInput.value = ''
  fileContent.value = ''
  uploadResult.value = null
  currentView.value = 'upload'
  resetEditState()
}

const handleKeydown = (e: KeyboardEvent) => {
  if (isAuthenticated.value) return
  if (!isNaN(Number(e.key)) && e.key !== ' ') handlePinInput(e.key)
  else if (e.key === 'Backspace') handleBackspace()
  else if (e.key === 'Enter') checkUnlock()
}

const resetEditState = () => {
  editingId.value = null
  fileContent.value = ''
  fileName.value = ''
  uploadResult.value = null
  selectedTtl.value = 0 // Reset to permanent
}

const formatTtl = (ttl: number): string => {
  if (ttl === -1) return '∞ Permanent'
  if (ttl <= 0) return 'Expired'
  const days = Math.floor(ttl / 86400)
  const hours = Math.floor((ttl % 86400) / 3600)
  const minutes = Math.floor((ttl % 3600) / 60)
  if (days > 0) return `${days}d ${hours}h`
  if (hours > 0) return `${hours}h ${minutes}m`
  return `${minutes}m`
}

const fetchFiles = async () => {
  loadingFiles.value = true
  try {
    const res: any = await $fetch('/api/list')
    if (res.success) files.value = res.files
  } catch (e) {
    showAlert('SYSTEM ERROR', 'Failed to retrieve file matrix.', 'error')
  } finally {
    loadingFiles.value = false
  }
}

const startEdit = async (id: string) => {
  loadingFiles.value = true
  try {
    const content: string = await $fetch(`/raw/${id}`, { responseType: 'text' })
    fileContent.value = content
    editingId.value = id
    fileName.value = id
    currentView.value = 'upload'
    uploadResult.value = null
    showAlert('EDIT MODE', `Loaded file ${id}. Ready to overwrite.`, 'info')
  } catch (e) {
    showAlert('FETCH ERROR', 'Failed to load file content.', 'error')
  } finally {
    loadingFiles.value = false
  }
}

const confirmDelete = (id: string) => {
  fileToDelete.value = id
  showAlert('CONFIRM DELETION', `Permanently purge file ID: ${id}?`, 'warning')
}

const executeDelete = async () => {
  if (!fileToDelete.value) return
  const id = fileToDelete.value
  alert.value.show = false
  loadingFiles.value = true
  try {
    await $fetch('/api/delete', { method: 'POST', body: { id } })
    files.value = files.value.filter(f => f.id !== id)
    if (editingId.value === id) resetEditState()
    showAlert('DELETION COMPLETE', `File ${id} has been scrubbed.`, 'success')
  } catch (e) {
    showAlert('DELETION FAILED', 'Could not delete file.', 'error')
  } finally {
    loadingFiles.value = false
    fileToDelete.value = null
  }
}

const showAlert = (title: string, message: string, type: 'success' | 'error' | 'warning' | 'info' = 'success') => {
  alert.value = { show: true, title, message, type }
}

const closeAlert = () => {
  alert.value.show = false
  fileToDelete.value = null
}

const handleAlertConfirm = () => {
  if (fileToDelete.value) executeDelete()
  else closeAlert()
}

const triggerFileSelect = () => {
  fileInput.value?.click()
}

const handleFileSelect = (event: Event) => {
  const input = event.target as HTMLInputElement
  if (input.files && input.files[0]) {
    const file = input.files[0]
    fileName.value = file.name
    const reader = new FileReader()
    reader.onload = (e) => {
      if (e.target?.result) fileContent.value = e.target.result as string
    }
    reader.readAsText(file)
  }
}

const handleSaveOrUpload = async () => {
  if (!fileContent.value) return
  loading.value = true
  uploadResult.value = null
  try {
    let resultId = ''
    if (editingId.value) {
      // Logic Update
      const data: any = await $fetch('/api/update', {
        method: 'POST',
        body: { 
          id: editingId.value, 
          content: fileContent.value,
          ttl: selectedTtl.value > 0 ? selectedTtl.value : null // null = permanent
        }
      })
      resultId = data.id
      const ttlText = selectedTtl.value > 0 ? formatTtl(selectedTtl.value) : 'Permanent'
      showAlert('UPDATE SUCCESS', `File ${resultId} overwritten. TTL: ${ttlText}`, 'success')
    } else {
      // Logic Upload Baru (dengan Custom Filename jika ada)
      const payload: any = { 
        content: fileContent.value,
        ttl: selectedTtl.value > 0 ? selectedTtl.value : null // null = permanent
      }
      if (fileName.value) payload.filename = fileName.value

      const data: any = await $fetch('/api/upload', {
        method: 'POST',
        body: payload
      })
      resultId = data.id
      const ttlText = selectedTtl.value > 0 ? formatTtl(selectedTtl.value) : 'Permanent'
      showAlert('UPLOAD SUCCESS', `Script stored. TTL: ${ttlText}`, 'success')
    }
    const domain = window.location.origin
    uploadResult.value = `${domain}/raw/${resultId}`
    fetchFiles()
  } catch (err: any) {
    showAlert('OPERATION FAILED', err.statusMessage || 'Failed to process request.', 'error')
  } finally {
    loading.value = false
  }
}

const copyResult = (text: string) => {
  navigator.clipboard.writeText(text)
  copied.value = true
  setTimeout(() => copied.value = false, 2000)
}

const getRawLink = (id: string) => {
  return `${window.location.origin}/raw/${id}`
}

onMounted(() => { window.addEventListener('keydown', handleKeydown) })
onUnmounted(() => { window.removeEventListener('keydown', handleKeydown) })
</script>

<template>
  <div class="min-h-screen bg-cyber-black text-white font-sans flex flex-col items-center justify-center relative overflow-hidden selection:bg-cyber-primary/30">
    <div class="fixed inset-0 bg-grid-pattern bg-[size:40px_40px] opacity-20 pointer-events-none animate-[scan_20s_linear_infinite]"></div>
    <div class="fixed top-[-20%] right-[-10%] w-[600px] h-[600px] bg-cyber-primary/10 rounded-full blur-[120px] pointer-events-none"></div>

    <CyberAlert :show="alert.show" :title="alert.title" :message="alert.message" :type="alert.type" @cancel="closeAlert" @confirm="handleAlertConfirm" />

    <transition name="fade-slide" mode="out-in">
      <div v-if="!isAuthenticated" class="z-50 w-full max-w-md px-6 flex flex-col items-center">
        <div class="mb-8 text-center space-y-2">
          <Shield class="w-12 h-12 text-cyber-primary mx-auto animate-pulse" />
          <h2 class="text-2xl font-cyber tracking-[0.2em] text-white">SECURE STORAGE</h2>
          <p class="text-xs font-mono text-slate-500">REDIS ENCRYPTED VAULT</p>
        </div>
        <div class="mb-8 flex justify-center gap-4">
          <div v-for="i in maxPinLength" :key="i" class="w-4 h-4 rounded-full border border-cyber-primary transition-all duration-200" :class="{ 'bg-cyber-primary shadow-[0_0_10px_#00f3ff]': i <= pinInput.length, 'bg-transparent opacity-30': i > pinInput.length, 'animate-shake bg-red-500 border-red-500': pinError }"></div>
        </div>
        <div class="grid grid-cols-3 gap-4 w-full max-w-[280px]">
          <button v-for="num in ['1', '2', '3', '4', '5', '6', '7', '8', '9']" :key="num" @click="handlePinInput(num)" :disabled="checkingPin" class="h-16 rounded-lg bg-cyber-dark/50 border border-white/10 hover:border-cyber-primary hover:bg-cyber-primary/10 text-xl font-mono text-white flex items-center justify-center relative group disabled:opacity-50 transition-all">
            {{ num }}
          </button>
          <button @click="handleBackspace" :disabled="checkingPin" class="h-16 rounded-lg bg-red-900/20 border border-red-500/20 hover:border-red-500 text-red-400 flex items-center justify-center disabled:opacity-50 transition-all">
            <Delete class="w-6 h-6" />
          </button>
          <button @click="handlePinInput('0')" :disabled="checkingPin" class="h-16 rounded-lg bg-cyber-dark/50 border border-white/10 hover:border-cyber-primary hover:bg-cyber-primary/10 text-xl font-mono text-white flex items-center justify-center disabled:opacity-50 transition-all">0</button>
          <button @click="checkUnlock" :disabled="checkingPin" class="h-16 rounded-lg bg-cyber-primary/20 border border-cyber-primary/50 text-cyber-primary hover:bg-cyber-primary hover:text-black flex items-center justify-center font-bold disabled:opacity-50 transition-all">
            <Loader2 v-if="checkingPin" class="w-6 h-6 animate-spin" /><span v-else>ENT</span>
          </button>
        </div>
        <p v-if="pinError" class="mt-8 text-xs text-red-500 font-mono animate-glitch">INVALID ACCESS PIN</p>
      </div>

      <main v-else class="w-full max-w-4xl z-10 px-4 py-8 flex flex-col items-center">
        <div class="w-full flex justify-between items-center mb-8 border-b border-white/10 pb-4">
          <div>
            <h1 class="text-2xl md:text-3xl font-cyber font-bold text-transparent bg-clip-text bg-gradient-to-r from-white via-cyber-primary to-white drop-shadow-[0_0_10px_rgba(0,243,255,0.5)]">
              RAW<span class="text-cyber-primary">HOST</span>
            </h1>
            <p class="text-[10px] font-mono text-slate-500 tracking-widest">REDIS DATABASE INTERFACE</p>
          </div>
          <button @click="handleLogout" class="flex items-center gap-2 px-3 py-1.5 rounded bg-red-500/10 border border-red-500/20 hover:bg-red-500/20 text-red-400 text-xs font-mono transition-all">
            <LogOut class="w-3 h-3" /> OUT
          </button>
        </div>

        <div class="w-full relative group">
          <div class="absolute -inset-[1px] bg-gradient-to-r from-cyber-primary via-blue-500 to-cyber-primary rounded-xl opacity-50 blur-sm animate-pulse-fast"></div>

          <div class="relative bg-cyber-dark/95 backdrop-blur-xl border border-white/10 rounded-xl shadow-2xl overflow-hidden flex flex-col md:flex-row min-h-[500px]">
            <aside class="w-full md:w-64 bg-black/40 border-b md:border-b-0 md:border-r border-white/5 p-4 flex flex-row md:flex-col gap-2">
              <button @click="currentView = 'upload'; if (!editingId) resetEditState()" :class="['flex-1 md:flex-none flex items-center gap-3 px-4 py-3 rounded-lg text-sm font-cyber tracking-wider transition-all', currentView === 'upload' ? 'bg-cyber-primary/20 text-cyber-primary border border-cyber-primary/50 shadow-[0_0_15px_rgba(0,243,255,0.2)]' : 'text-slate-400 hover:text-white hover:bg-white/5']">
                <UploadCloud v-if="!editingId" class="w-4 h-4" />
                <Edit v-else class="w-4 h-4" />
                {{ editingId ? 'EDITOR' : 'UPLOAD' }}
              </button>
              <button @click="currentView = 'manage'; fetchFiles()" :class="['flex-1 md:flex-none flex items-center gap-3 px-4 py-3 rounded-lg text-sm font-cyber tracking-wider transition-all', currentView === 'manage' ? 'bg-cyber-primary/20 text-cyber-primary border border-cyber-primary/50 shadow-[0_0_15px_rgba(0,243,255,0.2)]' : 'text-slate-400 hover:text-white hover:bg-white/5']">
                <List class="w-4 h-4" /> MANAGE
              </button>
            </aside>

            <div class="flex-grow p-6 md:p-8 overflow-y-auto max-h-[70vh] custom-scrollbar">
              <div v-if="currentView === 'upload'" class="space-y-6 animate-fade-in">
                <div class="flex items-center justify-between">
                  <div class="flex flex-col">
                    <h3 class="text-lg font-cyber text-white">
                      {{ editingId ? 'EDIT_MODE' : 'NEW_ENTRY' }}
                    </h3>
                    <span class="text-[10px] font-mono" :class="editingId ? 'text-yellow-400' : 'text-cyber-primary'">
                      {{ editingId ? `TARGET_ID: ${editingId}` : 'READY_TO_WRITE' }}
                    </span>
                  </div>
                  <button v-if="editingId" @click="resetEditState" class="text-xs text-red-400 hover:text-white underline font-mono">
                    CANCEL_EDIT
                  </button>
                </div>

                <div class="space-y-4">
                  <div class="flex gap-3">
                    <div class="relative flex-grow">
                      <div class="absolute inset-y-0 left-0 pl-4 flex items-center pointer-events-none">
                        <FileCode class="h-4 w-4 text-cyber-primary" />
                      </div>
                      <input v-model="fileName" type="text" :placeholder="editingId ? 'File ID Locked' : 'filename_alias (optional)'" :disabled="!!editingId" class="w-full bg-cyber-gray/50 border border-cyber-primary/30 rounded-lg py-3 pl-10 pr-4 text-sm text-cyber-primary font-mono placeholder:text-slate-600 focus:outline-none focus:border-cyber-primary focus:ring-1 focus:ring-cyber-primary disabled:opacity-50 disabled:cursor-not-allowed transition-all" />
                    </div>
                    <input type="file" ref="fileInput" @change="handleFileSelect" class="hidden" />
                    <button @click="triggerFileSelect" class="px-4 bg-cyber-gray border border-cyber-primary/30 rounded-lg hover:bg-cyber-primary/20 hover:border-cyber-primary transition-all group/file" title="Load File">
                      <FolderOpen class="w-5 h-5 text-cyber-primary group-hover/file:scale-110 transition-transform" />
                    </button>
                  </div>

                  <!-- TTL Selector -->
                  <div class="flex items-center gap-3">
                    <span class="text-xs font-mono text-slate-400 whitespace-nowrap">EXPIRY:</span>
                    <div class="flex flex-wrap gap-2">
                      <button 
                        v-for="opt in [{label: '∞ PERM', value: 0}, {label: '1H', value: 3600}, {label: '24H', value: 86400}, {label: '7D', value: 604800}, {label: '30D', value: 2592000}]" 
                        :key="opt.value"
                        @click="selectedTtl = opt.value"
                        :class="['px-3 py-1.5 rounded text-xs font-mono border transition-all', selectedTtl === opt.value ? 'bg-cyber-primary/20 border-cyber-primary text-cyber-primary' : 'bg-black/30 border-white/10 text-slate-400 hover:border-cyber-primary/50']"
                      >
                        {{ opt.label }}
                      </button>
                    </div>
                  </div>

                  <div class="relative group/editor">
                    <textarea v-model="fileContent" rows="12" placeholder="// Paste script or text content here..." class="w-full bg-black/50 border border-white/10 rounded-lg p-4 font-mono text-xs md:text-sm text-slate-300 focus:outline-none focus:border-cyber-primary focus:bg-black/80 transition-all resize-y selection:bg-cyber-primary/30 custom-scrollbar"></textarea>
                  </div>
                </div>

                <div class="flex justify-end pt-2">
                  <button @click="handleSaveOrUpload" :disabled="loading || !fileContent" :class="['w-full md:w-auto relative px-8 py-3 font-cyber font-bold tracking-wider hover:shadow-[0_0_20px_rgba(0,243,255,0.6)] disabled:opacity-50 transition-all clip-btn', editingId ? 'bg-yellow-400 text-black hover:bg-white' : 'bg-cyber-primary text-black hover:bg-white']">
                    <span class="relative z-10 flex items-center justify-center gap-2">
                      <Loader2 v-if="loading" class="w-5 h-5 animate-spin" />
                      <span v-else>{{ editingId ? 'SAVE CHANGES' : 'GENERATE LINK' }}</span>
                      <UploadCloud v-if="!editingId" class="w-4 h-4" />
                      <Edit v-else class="w-4 h-4" />
                    </span>
                  </button>
                </div>

                <div v-if="uploadResult" class="mt-6 space-y-4 border-t border-white/5 pt-6 animate-fade-in">
                  <div class="text-cyber-primary text-xs font-mono uppercase tracking-widest flex items-center gap-2">
                    <Terminal class="w-4 h-4" /> {{ editingId ? 'UPDATE_SUCCESS' : 'UPLOAD_SUCCESS' }}
                  </div>
                  <div class="relative bg-black/60 rounded border border-cyber-primary/20 p-4 pr-12">
                    <code class="font-mono text-xs text-white break-all block">{{ uploadResult }}</code>
                    <button @click="copyResult(uploadResult)" class="absolute top-2 right-2 text-slate-500 hover:text-cyber-primary">
                      <Copy class="w-4 h-4" />
                    </button>
                  </div>
                  <a :href="uploadResult" target="_blank" class="block text-center bg-cyber-primary/10 hover:bg-cyber-primary/20 border border-cyber-primary/50 text-cyber-primary py-2 rounded text-sm font-cyber uppercase tracking-widest transition-all">OPEN RAW</a>
                </div>
              </div>

              <div v-else class="space-y-6 animate-fade-in h-full flex flex-col">
                <div class="flex items-center justify-between">
                  <h3 class="text-lg font-cyber text-white">DATABASE_INDEX</h3>
                  <button @click="fetchFiles" class="p-2 text-cyber-primary hover:bg-cyber-primary/10 rounded transition-colors" title="Refresh">
                    <RefreshCw class="w-4 h-4" :class="{ 'animate-spin': loadingFiles }" />
                  </button>
                </div>

                <div v-if="loadingFiles && files.length === 0" class="flex-grow flex flex-col items-center justify-center text-slate-500 gap-4">
                  <Loader2 class="w-8 h-8 animate-spin text-cyber-primary" />
                  <p class="font-mono text-xs tracking-widest">SCANNING REDIS KEYS...</p>
                </div>

                <div v-else-if="files.length === 0" class="flex-grow flex flex-col items-center justify-center text-slate-600 gap-2 opacity-50">
                  <FolderOpen class="w-12 h-12" />
                  <p class="font-mono text-xs">NO DATA FOUND</p>
                </div>

                <div v-else class="space-y-2 overflow-y-auto custom-scrollbar pr-2">
                  <div v-for="file in files" :key="file.id" class="group bg-white/5 border border-white/5 hover:border-cyber-primary/50 hover:bg-white/10 rounded-lg p-3 flex items-center justify-between transition-all">
                    <div class="flex items-center gap-3 overflow-hidden">
                      <div class="p-2 bg-black/50 rounded text-cyber-primary">
                        <FileCode class="w-4 h-4" />
                      </div>
                      <div class="min-w-0">
                        <p class="text-sm font-mono text-white truncate w-full">ID: <span class="text-cyber-primary">{{ file.id }}</span></p>
                        <p class="text-[10px] truncate" :class="file.ttl === -1 ? 'text-green-400' : 'text-yellow-400'">
                          TTL: {{ formatTtl(file.ttl) }}
                        </p>
                      </div>
                    </div>

                    <div class="flex items-center gap-2">
                      <button @click="startEdit(file.id)" class="p-2 text-yellow-400/70 hover:text-yellow-400 hover:bg-yellow-400/10 rounded transition-all" title="Edit Content">
                        <Edit class="w-4 h-4" />
                      </button>

                      <a :href="getRawLink(file.id)" target="_blank" class="p-2 text-slate-400 hover:text-cyber-primary hover:bg-cyber-primary/10 rounded transition-all" title="Open Raw">
                        <ExternalLink class="w-4 h-4" />
                      </a>
                      <button @click="copyResult(getRawLink(file.id))" class="p-2 text-slate-400 hover:text-white hover:bg-white/10 rounded transition-all" title="Copy Link">
                        <Copy class="w-4 h-4" />
                      </button>
                      <button @click="confirmDelete(file.id)" class="p-2 text-red-500/70 hover:text-red-500 hover:bg-red-500/10 rounded transition-all" title="Delete">
                        <Trash2 class="w-4 h-4" />
                      </button>
                    </div>
                  </div>
                </div>
              </div>

            </div>
          </div>
        </div>
      </main>
    </transition>
  </div>
</template>