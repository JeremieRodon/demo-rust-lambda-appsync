<script setup>
import { inject } from 'vue';

const signed_admin = inject('signed_admin');
</script>

<template>
  <main>
    <h1 class="text-4xl my-6 font-black text-center">Administration</h1>
    {{ signed_admin }}
  </main>
</template>
