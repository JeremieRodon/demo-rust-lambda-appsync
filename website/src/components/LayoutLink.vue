<script setup>
defineProps({
  to: {
    type: String,
  },
  title: {
    type: String,
  },
});
</script>

<template>
  <router-link :to="to" class="flex flex-col items-center" activeClass="text-primary">
    <slot></slot>
    <div class="font-light text-xs">{{ title }}</div>
  </router-link>
</template>
