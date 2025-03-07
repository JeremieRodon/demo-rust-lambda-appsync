<script setup>
import { computed, inject, ref } from 'vue';
import TeamIcon from './TeamIcon.vue';
import { alert_appsync_error, alert_success } from '@/modules/utils';

const props = defineProps({
  rank: Number,
  player: Object,
});

const current_player = inject('current_player');
const is_current_player = computed(() => {
  if (current_player.value) {
    return current_player.value.id == props.player.id;
  } else {
    return false;
  }
});

const clicks = computed(() => {
  if (props.player.clicks) {
    return props.player.clicks;
  } else {
    return 0;
  }
});

const avg_latency = computed(() => {
  if (props.player.avg_latency) {
    return Math.floor(props.player.avg_latency * 100) / 100;
  } else {
    return NaN;
  }
});

const signed_user_is_admin = inject('signed_user_is_admin');
const client = inject('appsync_admin_client');
const in_operation = ref(false);

async function delete_player() {
  in_operation.value = true;
  const player_id = props.player.id;
  const variables = {
    player_id,
  };
  console.log(variables);
  try {
    await client.graphql({
      query: `
        mutation RemovePlayer($player_id: ID!) {
            removePlayer(player_id: $player_id) {
              id
              name
              team
              clicks
              avg_latency
              avg_latency_clicks
            }
          }
      `,
      variables,
    });
    alert_success('Player deleted');
  } catch (e) {
    alert_appsync_error(e, 'Could not delete player');
  } finally {
    in_operation.value = false;
  }
}
</script>

<template>
  <div
    class="flex flex-row max-w-full min-w-fit gap-2"
    :class="is_current_player ? 'border-2 border-secondary rounded-box' : ''"
  >
    <div class="basis-1/6 sm:basis-1/12 text-lg font-bold self-center grow text-center">
      #{{ rank }}
    </div>
    <div class="basis-4/6 sm:basis-10/12 grid grid-cols-1 grid-row-2 sm:grid-cols-3 sm:grid-row-1">
      <div class="flex flex-row items-center justify-center ml-8 sm:justify-between sm:ml-0">
        <div class="font-black">{{ player.name }}</div>
        <team-icon :name="player.team" class="h-6 m-2 sm:m-4"></team-icon>
      </div>
      <div class="flex flex-row justify-center gap-4 sm:col-span-2">
        <div
          class="grid grid-cols-1 grid-rows-2 sm:grid-cols-2 sm:grid-rows-1 justify-start items-center sm:px-2"
        >
          <div class="text-base-content/60 font-bold sm:text-center text-sm">Total Clicks</div>
          <div class="flex flex-row items-center sm:justify-center">
            <svg
              xmlns="http://www.w3.org/2000/svg"
              viewBox="0 0 24 24"
              class="inline h-8 w-8 fill-current shrink-0"
            >
              <path
                d="M10.76,8.69A0.76,0.76 0 0,0 10,9.45V20.9C10,21.32 10.34,21.66 10.76,21.66C10.95,21.66 11.11,21.6 11.24,21.5L13.15,19.95L14.81,23.57C14.94,23.84 15.21,24 15.5,24C15.61,24 15.72,24 15.83,23.92L18.59,22.64C18.97,22.46 19.15,22 18.95,21.63L17.28,18L19.69,17.55C19.85,17.5 20,17.43 20.12,17.29C20.39,16.97 20.35,16.5 20,16.21L11.26,8.86L11.25,8.87C11.12,8.76 10.95,8.69 10.76,8.69M15,10V8H20V10H15M13.83,4.76L16.66,1.93L18.07,3.34L15.24,6.17L13.83,4.76M10,0H12V5H10V0M3.93,14.66L6.76,11.83L8.17,13.24L5.34,16.07L3.93,14.66M3.93,3.34L5.34,1.93L8.17,4.76L6.76,6.17L3.93,3.34M7,10H2V8H7V10"
              />
            </svg>
            <div class="flex flex-col">
              <div class="text-lg font-black">{{ clicks }}</div>
              <div class="text-base-content/60 text-xs font-light">clicks</div>
            </div>
          </div>
        </div>
        <div
          class="grid grid-cols-1 grid-rows-2 sm:grid-cols-2 sm:grid-rows-1 justify-start items-center sm:px-2"
        >
          <div class="text-base-content/60 font-bold sm:text-center text-sm">Avg. Latency</div>
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
              <div class="text-lg font-black">{{ avg_latency }}</div>
              <div class="text-base-content/60 text-xs font-light">milliseconds</div>
            </div>
          </div>
        </div>
      </div>
    </div>
    <div class="basis-1/6 sm:basis-1/12 self-center text-center">
      <button
        v-if="signed_user_is_admin"
        class="btn btn-square btn-xs btn-primary"
        tabindex="-1"
        :disabled="in_operation"
        @click="delete_player()"
      >
        <svg
          v-show="!in_operation"
          xmlns="http://www.w3.org/2000/svg"
          class="h-5 w-5 fill-current"
          viewBox="0 0 24 24"
        >
          <path
            d="M19,6.41L17.59,5L12,10.59L6.41,5L5,6.41L10.59,12L5,17.59L6.41,19L12,13.41L17.59,19L19,17.59L13.41,12L19,6.41Z"
          />
        </svg>
        <span v-show="in_operation" class="loading loading-spinner loading-md"></span>
      </button>
    </div>
  </div>
</template>
