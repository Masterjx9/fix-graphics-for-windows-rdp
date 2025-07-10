<script setup lang="ts">
import { ref } from 'vue'
import { Label } from '@/components/ui/label'
import { RadioGroup, RadioGroupItem } from '@/components/ui/radio-group'
import { invoke } from '@tauri-apps/api/core'
import { Button } from '@/components/ui/button'

interface VideoController {
  Name?: string
  DriverVersion?: string
  Status?: string
}

const gpus         = ref<VideoController[]>([])
const selectedGpu  = ref<string | null>(null)
const loading      = ref(false)
const gpuStatusMap = ref<Record<string, 'green' | 'red' | 'yellow'>>({})
const buttonText = ref('Reset GPU')
const isResetting = ref(false)
const showResultAlert = ref(false)
const resultAlert = ref('GPU reset successfully!')


async function fetchGPUInfo() {
  const result = (await invoke('get_gpu_info')) as VideoController[]
  gpus.value = result
  loading.value = true
  for (const gpu of result) {
    gpuStatusMap.value[gpu.Name ?? 'unknown'] =
      gpu.Status === 'OK' ? 'green' : 'red'
  }
}
fetchGPUInfo()

async function resetGPU() {
  console.log('Resetting GPU:', selectedGpu.value)
  isResetting.value = true
  buttonText.value = 'Resetting...'
  // disable the button

  gpuStatusMap.value[selectedGpu.value ?? 'unknown'] = 'yellow'
  try {
    const result = await invoke('reset_gpu', { gpuName: selectedGpu.value })
    console.log('Reset result:', result)
    console.log('Reset result:', gpuStatusMap.value[selectedGpu.value ?? 'unknown'])
    gpuStatusMap.value[selectedGpu.value ?? 'unknown'] = 'green'
    resultAlert.value = 'GPU reset successfully!'
    showResultAlert.value = true
    setTimeout(() => {
      showResultAlert.value = false
    }, 4000)

  } catch (e) {
    console.error(e)
    resultAlert.value = 'Failed to reset GPU: ' + e
    gpuStatusMap.value[selectedGpu.value ?? 'unknown'] = 'red'
  } finally {
    isResetting.value = false
    buttonText.value = 'Reset GPU'
  }
  
}
</script>

<template>
  <main class="container">
    <h1>Reset Graphics for Windows RDP</h1>

    <!-- success that is hidden -->
    <p v-if="showResultAlert">
      <b>{{ resultAlert }}</b>
    </p>

    <!-- Loading state -->
    <p v-if="!loading">Loading…</p>

    <!-- GPU selector -->
    <RadioGroup v-else v-model="selectedGpu" class="mt-6">
      <div
        v-for="(gpu, idx) in gpus"
        :key="idx"
        class="flex items-center space-x-2 mb-2"
      >
        <RadioGroupItem :id="`gpu-${idx}`" :value="gpu.Name" />
        <Label :for="`gpu-${idx}`" class="flex-1">
          {{ gpu.Name }} - v{{ gpu.DriverVersion }}
        </Label>
        <span
            :class="[
              'ml-auto h-2 w-2 rounded-full',
              {
      'bg-green-500': gpuStatusMap[gpu.Name ?? 'unknown'] === 'green',
      'bg-yellow-400': gpuStatusMap[gpu.Name ?? 'unknown'] === 'yellow',
      'bg-red-500': gpuStatusMap[gpu.Name ?? 'unknown'] === 'red',
    }
            ]"
          />
      </div>
    </RadioGroup>

    <p class="mt-4">Selected GPU: {{ selectedGpu }}</p>
    <br />
    <Button
      :disabled="!selectedGpu || isResetting"
      @click="resetGPU"
      class="mt-4"
      style="cursor: pointer;"
    >
      {{ buttonText }}
    </Button>
    <div class="mt-10 border border-border rounded-xl p-4 text-sm text-center">
  <p class="flex items-center justify-center gap-2 text-muted-foreground">
    Like this project? Contribute on GitHub!
    <a
      href="https://github.com/masterjx9/fix-graphics-for-windows-rdp"
      target="_blank"
      class="text-blue-500 hover:text-blue-700"
    >
      <svg
        role="img"
        viewBox="0 0 24 24"
        xmlns="http://www.w3.org/2000/svg"
        class="w-5 h-5 inline-block"
        fill="currentColor"
      >
        <title>GitHub</title>
        <path
          d="M12 .297c-6.63 0-12 5.373-12 12 0 5.303 3.438 9.8 8.205 11.385.6.113.82-.258.82-.577 0-.285-.01-1.04-.015-2.04-3.338.724-4.042-1.61-4.042-1.61C4.422 18.07 3.633 17.7 3.633 17.7c-1.087-.744.084-.729.084-.729 1.205.084 1.838 1.236 1.838 1.236 1.07 1.835 2.809 1.305 3.495.998.108-.776.417-1.305.76-1.605-2.665-.3-5.466-1.332-5.466-5.93 0-1.31.465-2.38 1.235-3.22-.135-.303-.54-1.523.105-3.176 0 0 1.005-.322 3.3 1.23.96-.267 1.98-.399 3-.405 1.02.006 2.04.138 3 .405 2.28-1.552 3.285-1.23 3.285-1.23.645 1.653.24 2.873.12 3.176.765.84 1.23 1.91 1.23 3.22 0 4.61-2.805 5.625-5.475 5.92.42.36.81 1.096.81 2.22 0 1.606-.015 2.896-.015 3.286 0 .315.21.69.825.57C20.565 22.092 24 17.592 24 12.297c0-6.627-5.373-12-12-12"
        />
      </svg>
    </a>
  </p>
</div>



  </main>
</template>

<style scoped>
.container {
  margin: 0 auto;
  padding-top: 10vh;
  max-width: 400px;
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
}
h1 {
  font-size: 2rem;
  font-weight: bold;
  margin-bottom: 2rem;
}
.mt-4 { margin-top: 1rem; }
.mt-6 { margin-top: 1.5rem; }
.mb-2 { margin-bottom: 0.5rem; }
</style>
