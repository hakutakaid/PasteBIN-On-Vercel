<script setup lang="ts">
    import { AlertTriangle, ShieldCheck, Loader2 } from 'lucide-vue-next'
    
    // Menerima data dari luar (props)
    const props = defineProps<{
      show: boolean
      title?: string
      message?: string
      type?: 'info' | 'warning' | 'success' | 'error'
      loading?: boolean
    }>()
    
    // Mengirim sinyal ke luar (emits)
    const emit = defineEmits<{
      (e: 'confirm'): void
      (e: 'cancel'): void
    }>()
    
    // Warna berdasarkan tipe alert
    const typeColors = {
      info: 'text-cyber-primary border-cyber-primary',
      success: 'text-green-400 border-green-400',
      warning: 'text-yellow-400 border-yellow-400',
      error: 'text-red-500 border-red-500'
    }
</script>

<template>
    <Teleport to="body">
        <transition name="modal">
            <div v-if="show" class="fixed inset-0 z-[9999] flex items-center justify-center p-4">

                <div class="fixed inset-0 bg-black/80 backdrop-blur-sm transition-opacity" @click="$emit('cancel')"></div>

                <div class="relative w-full max-w-md bg-cyber-dark/90 border border-cyber-accent/50 rounded-lg shadow-[0_0_30px_rgba(112,0,255,0.3)] p-6 overflow-hidden clip-path-cyber scale-100 transition-all">

                    <div class="absolute top-0 left-0 w-full h-1 bg-gradient-to-r from-transparent via-cyber-accent to-transparent"></div>

                    <div class="flex items-center gap-3 mb-4">
                        <div :class="`p-2 rounded-full bg-opacity-20 border ${typeColors[type || 'info']}`">
                            <AlertTriangle v-if="type === 'warning' || type === 'error'" class="w-6 h-6" />
                            <ShieldCheck v-else class="w-6 h-6" />
                        </div>
                        <h3 class="text-xl font-cyber tracking-wider text-white">{{ title || 'SYSTEM ALERT' }}</h3>
                    </div>

                    <p class="text-slate-300 font-mono text-sm mb-8 leading-relaxed">
                        {{ message || 'Are you sure you want to proceed?' }}
                    </p>

                    <div class="flex justify-end gap-4">
                        <button @click="$emit('cancel')" :disabled="loading" class="px-6 py-2 rounded font-cyber text-sm tracking-widest text-slate-400 border border-white/10 hover:bg-white/5 hover:text-white transition-all disabled:opacity-50">
                            CANCEL
                        </button>

                        <button @click="$emit('confirm')" :disabled="loading" class="relative px-6 py-2 rounded font-cyber text-sm tracking-widest bg-cyber-accent text-white hover:bg-cyber-accent/80 hover:shadow-[0_0_15px_rgba(112,0,255,0.5)] transition-all flex items-center justify-center gap-2 disabled:opacity-70 disabled:cursor-not-allowed clip-btn">
                            <Loader2 v-if="loading" class="w-4 h-4 animate-spin" />
                            <span v-else>CONFIRM</span>
                        </button>
                    </div>

                </div>
            </div>
        </transition>
    </Teleport>
</template>