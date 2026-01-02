<script setup lang="ts">
import { clearError } from '#app'
import { AlertTriangle, Home, RefreshCw, ShieldAlert } from 'lucide-vue-next'

const props = defineProps({
  error: Object
})

const handleError = () => clearError({ redirect: '/' })

const statusCode = props.error?.statusCode || 500
const message = props.error?.statusMessage || 'CRITICAL_SYSTEM_FAILURE'
</script>

<template>
  <div class="min-h-screen bg-cyber-black text-white font-sans selection:bg-red-500/30 flex flex-col items-center justify-center relative overflow-hidden">
    <div class="fixed inset-0 bg-grid-pattern bg-[size:40px_40px] opacity-10 pointer-events-none animate-[scan_5s_linear_infinite]"></div>
    <div class="fixed top-[-20%] left-[-10%] w-[600px] h-[600px] bg-red-600/10 rounded-full blur-[120px] pointer-events-none animate-pulse-slow"></div>
    <div class="fixed bottom-[-20%] right-[-10%] w-[600px] h-[600px] bg-cyber-accent/10 rounded-full blur-[120px] pointer-events-none"></div>
    <div class="fixed inset-0 bg-[linear-gradient(transparent_50%,rgba(0,0,0,0.5)_50%)] bg-[length:100%_4px] pointer-events-none z-20 opacity-20"></div>

    <main class="relative z-30 max-w-lg w-full p-6 text-center">
      <div class="flex justify-center mb-6">
        <div class="relative">
          <ShieldAlert class="w-20 h-20 text-red-500/80 animate-pulse" />
          <ShieldAlert class="w-20 h-20 text-cyber-primary/50 absolute top-0 left-0 animate-glitch opacity-50" />
        </div>
      </div>
      <h1 class="text-8xl md:text-9xl font-cyber font-bold tracking-tighter text-transparent bg-clip-text bg-gradient-to-b from-white to-slate-500 relative inline-block mb-4" data-text="ERROR">
        {{ statusCode }}
        <span class="absolute top-0 left-0 -ml-1 text-red-500 opacity-70 animate-glitch">{{ statusCode }}</span>
        <span class="absolute top-0 left-0 ml-1 text-blue-500 opacity-70 animate-glitch" style="animation-delay: 0.1s">{{ statusCode }}</span>
      </h1>
      <div class="bg-black/60 backdrop-blur-md border border-red-500/30 rounded-lg p-6 mb-8 text-left relative overflow-hidden group">
        <div class="absolute top-0 left-0 w-full h-1 bg-red-500/50"></div>
        <div class="absolute bottom-0 right-0 w-4 h-4 border-b-2 border-r-2 border-red-500"></div>
        <div class="font-mono text-sm space-y-2">
          <p class="text-red-400"><span class="text-slate-500">root@sys:~/logs#</span> <span class="animate-pulse">cat error.log</span></p>
          <p class="text-slate-300">> PROCESS TERMINATED UNEXPECTEDLY</p>
          <p class="text-slate-300">> CODE: <span class="text-red-500 font-bold">{{ statusCode }}</span></p>
          <p class="text-slate-300">> MSG: <span class="text-cyber-primary">"{{ message }}"</span></p>
          <p class="text-slate-500 mt-4 text-xs">// CONTACT ADMIN OR REBOOT SYSTEM</p>
        </div>
      </div>
      <button @click="handleError" class="group relative inline-flex items-center justify-center gap-3 px-8 py-4 bg-transparent border border-white/20 hover:border-cyber-primary text-white font-cyber tracking-widest uppercase transition-all duration-300 hover:bg-cyber-primary/10 hover:shadow-[0_0_20px_rgba(0,243,255,0.4)] clip-btn">
        <Home class="w-5 h-5 group-hover:text-cyber-primary transition-colors" />
        <span>Return to Base</span>
        <div class="absolute top-0 left-0 w-2 h-2 bg-white group-hover:bg-cyber-primary transition-colors"></div>
        <div class="absolute bottom-0 right-0 w-2 h-2 bg-white group-hover:bg-cyber-primary transition-colors"></div>
      </button>
    </main>
    <div class="absolute bottom-8 text-[10px] text-slate-600 font-mono tracking-[0.5em]">SYSTEM_ID: NULL_PTR_EXCEPTION</div>
  </div>
</template>