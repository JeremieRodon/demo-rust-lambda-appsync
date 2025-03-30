<script setup>
import { alert_appsync_error, alert_success } from '@/modules/utils';
import { signOut } from 'aws-amplify/auth';
import { computed, inject, ref } from 'vue';

const client = inject('appsync_admin_client');

const signed_user = inject('signed_user');
const signed_user_is_admin = inject('signed_user_is_admin');
const email = computed(() => {
  if (signed_user.value) {
    return signed_user.value.email;
  }
  return '';
});
const in_operation = ref(null);
const game_status = inject('game_status');
const game_duration = ref(20);

const ranked_players = inject('ranked_players');

async function alter_game_state(mutation_name) {
  in_operation.value = true;
  try {
    const new_status = (
      await client.graphql({
        query: `
        mutation AlterGameState {
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

async function start_game(duration) {
  await alter_game_state('startGame');
  if (duration != null) {
    setTimeout(stop_game, duration * 1000);
  }
}
async function stop_game() {
  await alter_game_state('stopGame');
}
async function reset_game() {
  await alter_game_state('resetGame');
}
async function delete_all_players() {
  in_operation.value = true;
  const player_count = ranked_players.value.length;

  const mutations = ranked_players.value
    .map((player, i) => {
      return `player${i}: removePlayer(player_id:"${player.id}"){id name team clicks avg_latency avg_latency_clicks}`;
    })
    .join('\n');

  console.log(mutations);

  try {
    if (player_count == 0) {
      return;
    }
    await client.graphql({
      query: `
        mutation RemoveAllPlayer {
          ${mutations}
        }
      `,
    });
    alert_success(`${player_count} player(s) removed`);
  } catch (e) {
    alert_appsync_error(e, `Could not remove all players in the game 😭`);
  } finally {
    in_operation.value = false;
  }
}
</script>

<template>
  <main class="flex flex-col items-center">
    <div class="text-lg flex flex-col items-center m-4">
      <div class="font-bold">{{ email }}</div>
      <div class="font-black">
        Admin:
        <span :class="signed_user_is_admin ? 'text-success' : 'text-error'">{{
          signed_user_is_admin
        }}</span>
      </div>
    </div>
    <div class="flex flex-col sm:flex-row gap-4">
      <div class="flex flex-col">
        <fieldset class="fieldset sm:order-2">
          <legend class="fieldset-legend">Game duration</legend>
          <select class="select select-xs" v-model="game_duration">
            <option :value="null">Unlimited</option>
            <option :value="20">20 seconds</option>
            <option :value="30">30 seconds</option>
            <option :value="40">40 seconds</option>
            <option :value="60">60 seconds</option>
          </select>
        </fieldset>
        <button
          class="btn btn-primary uppercase"
          tabindex="-1"
          @click="start_game(game_duration)"
          :disabled="in_operation || game_status != 'RESET'"
        >
          <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" class="h-3/4 fill-current">
            <title>play</title>
            <path d="M8,5.14V19.14L19,12.14L8,5.14Z" />
          </svg>
          Start game
        </button>
      </div>
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
    <button
      class="btn btn-error m-8 font-bold uppercase"
      tabindex="-1"
      @click="delete_all_players"
      :disabled="in_operation || game_status != 'RESET'"
    >
      <svg xmlns="http://www.w3.org/2000/svg" class="h-3/4 fill-current" viewBox="0 0 24 24">
        <path
          d="M17 4V6H3V4H6.5L7.5 3H12.5L13.5 4H17M4 19V7H16V19C16 20.1 15.1 21 14 21H6C4.9 21 4 20.1 4 19M19 15H21V17H19V15M19 7H21V13H19V7Z"
        />
      </svg>
      Remove all players
    </button>
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
