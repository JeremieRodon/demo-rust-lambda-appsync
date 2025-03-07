<script setup>
import { alert_appsync_error, alert_success } from '@/modules/utils';
import { generateClient } from 'aws-amplify/api';
import { signOut } from 'aws-amplify/auth';
import { computed, inject, ref } from 'vue';

const signed_user = inject('signed_user');
const email = computed(() => {
  if (signed_user.value) {
    return signed_user.value.email;
  }
  return '';
});
const is_admin = computed(() => {
  return signed_user.value != null && signed_user.value.is_admin;
});
const in_operation = ref(null);
const game_status = inject('game_status');
const client = generateClient({
  authMode: 'userPool',
});

async function alter_game_state(mutation_name) {
  in_operation.value = true;
  try {
    const new_status = (
      await client.graphql({
        query: `
        mutation alterGameState {
          ${mutation_name}
        }
      `,
      })
    ).data[mutation_name];

    alert_success(`New game state: ${new_status}`);
  } catch (e) {
    alert_appsync_error(e, `Could not perform ${mutation_name} on the game 😭`);
  } finally {
    in_operation.value = false;
  }
}

async function start_game() {
  await alter_game_state('startGame');
}
async function stop_game() {
  await alter_game_state('stopGame');
}
async function reset_game() {
  await alter_game_state('resetGame');
}
</script>

<template>
  <main class="flex flex-col items-center">
    <div class="text-lg flex flex-col items-center m-4">
      <div class="font-bold">{{ email }}</div>
      <div class="font-black">
        Admin: <span :class="is_admin ? 'text-success' : 'text-error'">{{ is_admin }}</span>
      </div>
    </div>
    <div class="flex flex-row gap-4">
      <button
        class="btn btn-primary uppercase"
        tabindex="-1"
        @click="start_game"
        :disabled="in_operation || game_status != 'RESET'"
      >
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" class="h-3/4 fill-current">
          <title>play</title>
          <path d="M8,5.14V19.14L19,12.14L8,5.14Z" />
        </svg>
        Start Game
      </button>
      <button
        class="btn btn-primary uppercase"
        tabindex="-1"
        @click="stop_game"
        :disabled="in_operation || game_status != 'STARTED'"
      >
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" class="h-3/4 fill-current">
          <path d="M18,18H6V6H18V18Z" />
        </svg>
        Stop Game
      </button>
      <button
        class="btn btn-primary uppercase"
        tabindex="-1"
        @click="reset_game"
        :disabled="in_operation || game_status != 'STOPPED'"
      >
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" class="h-3/4 fill-current">
          <path
            d="M2 12C2 16.97 6.03 21 11 21C13.39 21 15.68 20.06 17.4 18.4L15.9 16.9C14.63 18.25 12.86 19 11 19C4.76 19 1.64 11.46 6.05 7.05C10.46 2.64 18 5.77 18 12H15L19 16H19.1L23 12H20C20 7.03 15.97 3 11 3C6.03 3 2 7.03 2 12Z"
          />
        </svg>
        Reset Game
      </button>
    </div>
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
