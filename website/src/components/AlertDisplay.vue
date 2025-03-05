<script setup>
import { alerts } from '@/modules/utils';
</script>

<template>
  <div
    v-if="alerts.length > 0"
    class="fixed z-50 top-2 w-[90%] left-[5%] md:w-[60%] md:left-[20%] lg:w-1/2 lg:left-1/4"
  >
    <div
      v-for="(alert, idx) in alerts"
      :key="idx"
      role="alert"
      class="alert p-1 rounded-xl mb-1 gap-1 md:gap-4 grid-cols-12"
      :class="alert.alert_class"
    >
      <div
        class="flex flex-row text-sm col-start-2 col-span-10 sm:col-span-2 w-full justify-center"
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          class="h-5 w-5 shrink-0 fill-current"
          viewBox="0 0 24 24"
        >
          <path
            d="M13,13H11V7H13M13,17H11V15H13M12,2A10,10 0 0,0 2,12A10,10 0 0,0 12,22A10,10 0 0,0 22,12A10,10 0 0,0 12,2Z"
          />
        </svg>
        <div>
          <b>{{ alert.title }}</b>
        </div>
      </div>
      <div
        class="flex flex-row text-xs col-start-2 col-span-10 sm:col-start-auto sm:col-span-8 w-full justify-center"
      >
        {{ alert.message }}
      </div>
      <button
        class="btn btn-circle btn-outline btn-xs text-current col-start-12 col-span-1 row-span-full"
        @click="alert.close()"
      >
        <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 24 24">
          <path
            d="M19,6.41L17.59,5L12,10.59L6.41,5L5,6.41L10.59,12L5,17.59L6.41,19L12,13.41L17.59,19L19,17.59L13.41,12L19,6.41Z"
          />
        </svg>
      </button>
    </div>
  </div>
</template>
