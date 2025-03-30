<script setup>
import { computed } from 'vue';

const props = defineProps({
  avg_latency: Number,
});
const avg_latency_rounded = computed(() => {
  if (isNaN(props.avg_latency)) {
    return '-';
  } else {
    return Math.round(props.avg_latency * 100) / 100;
  }
});
</script>

<template>
  <div
    class="grid grid-cols-1 grid-rows-2 sm:grid-cols-2 sm:grid-rows-1 justify-start items-center sm:px-2"
  >
    <div class="text-base-content/60 font-bold text-center text-sm">Avg. Latency</div>
    <div class="flex flex-row items-center sm:justify-center">
      <svg
        xmlns="http://www.w3.org/2000/svg"
        fill="none"
        viewBox="0 0 24 24"
        class="inline h-8 w-8 stroke-current shrink-0"
      >
        <path
          stroke-linecap="round"
          stroke-linejoin="round"
          stroke-width="2"
          d="M13 10V3L4 14h7v7l9-11h-7z"
        ></path>
      </svg>
      <div class="flex flex-col">
        <div class="text-lg font-black">{{ avg_latency_rounded }}</div>
        <div class="text-base-content/60 text-xs font-light">milliseconds</div>
      </div>
    </div>
  </div>
</template>
