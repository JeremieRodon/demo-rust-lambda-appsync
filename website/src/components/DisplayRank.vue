<script setup>
import { computed } from 'vue';

const props = defineProps({
  rank: Number,
});

const color_class = computed(() => {
  if (props.rank == 1) {
    return 'text-yellow-300';
  } else if (props.rank == 2) {
    return 'text-gray-500';
  } else if (props.rank == 3) {
    return 'text-orange-800';
  } else {
    return '';
  }
});
</script>

<template>
  <div
    class="grid grid-cols-1 grid-rows-2 sm:grid-cols-2 sm:grid-rows-1 justify-start items-center sm:px-2 self-stretch"
  >
    <div class="text-base-content/60 font-bold text-center text-sm">Rank</div>
    <div class="flex flex-row items-center sm:justify-center" :class="color_class">
      <svg
        xmlns="http://www.w3.org/2000/svg"
        viewBox="0 0 24 24"
        class="inline h-8 w-8 fill-current shrink-0"
      >
        <path
          d="M17 9.5V7.5L15 8.1V5.5H13V8.7L11 9.3V6.5H9V9.9L7 10.5V12.5L9 11.9V13.9L7 14.5V16.5L9 15.9V18.5H11V15.3L13 14.7V17.5H15V14.1L17 13.5V11.5L15 12.1V10.1L17 9.5M13 12.7L11 13.3V11.3L13 10.7V12.7Z"
        />
      </svg>
      <div class="text-lg font-black w-8">
        {{ rank ? rank : '-' }}
      </div>
    </div>
  </div>
</template>
