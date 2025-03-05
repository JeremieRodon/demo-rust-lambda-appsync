<script setup>
import { signOut } from 'aws-amplify/auth';
import { inject } from 'vue';

const signed_user = inject('signed_user');
</script>

<template>
  <main class="flex flex-col items-center">
    <h1 class="text-4xl my-6 font-black text-center">Admin</h1>
    <div>{{ signed_user }}</div>
    <button class="btn btn-ghost my-auto font-bold uppercase" tabindex="-1" @click="signOut">
      <svg xmlns="http://www.w3.org/2000/svg" class="h-3/4 fill-current" viewBox="0 0 24 24">
        <path
          d="M17 7L15.59 8.41L18.17 11H8V13H18.17L15.59 15.58L17 17L22 12M4 5H12V3H4C2.9 3 2 3.9 2 5V19C2 20.1 2.9 21 4 21H12V19H4V5Z"
        />
      </svg>
      Déconnexion
    </button>
  </main>
</template>
