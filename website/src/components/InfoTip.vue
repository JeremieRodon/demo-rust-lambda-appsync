<script setup>
defineProps({
  tooltip_text: String,
});
</script>

<template>
  <div
    class="tooltip tooltip-info before:text-xs before:text-white before:max-w-40 hover:after:delay-300 hover:before:delay-300"
    :data-tip="tooltip_text"
  >
    <svg xmlns="http://www.w3.org/2000/svg" class="w-3.5 h-3.5 fill-info" viewBox="0 0 24 24">
      <path
        d="M13,9H11V7H13M13,17H11V11H13M12,2A10,10 0 0,0 2,12A10,10 0 0,0 12,22A10,10 0 0,0 22,12A10,10 0 0,0 12,2Z"
      />
    </svg>
  </div>
</template>
