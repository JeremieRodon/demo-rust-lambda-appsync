<script setup>
defineProps({
  team_name: String,
  click_count: Number,
  avg_latency: Number,
});
</script>

<template>
  <div>
    <div>{{ team_name }}</div>
    <div>{{ click_count }}</div>
    <div>{{ avg_latency }}</div>
  </div>
</template>
