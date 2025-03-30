<script setup>
import { computed, inject, ref } from 'vue';
import TeamIcon from './TeamIcon.vue';
import { alert_appsync_error, alert_success } from '@/modules/utils';
import DisplayPlayerClicks from './DisplayClicks.vue';
import DisplayPlayerLatency from './DisplayLatency.vue';

const props = defineProps({
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
    return props.player.avg_latency;
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
    :class="is_current_player ? 'border-2 border-secondary' : ''"
  >
    <div class="basis-1/6 sm:basis-1/12 text-lg font-bold self-center grow text-center">
      #{{ player.rank ? player.rank : ' -' }}
    </div>
    <div class="basis-4/6 sm:basis-10/12 grid grid-cols-1 grid-row-2 sm:grid-cols-3 sm:grid-row-1">
      <div class="flex flex-row items-center justify-center ml-8 sm:justify-between sm:ml-0">
        <div class="font-black">{{ player.name }}</div>
        <team-icon :name="player.team" class="h-6 m-2 sm:m-4"></team-icon>
      </div>
      <div class="flex flex-row justify-center gap-4 sm:col-span-2">
        <DisplayPlayerClicks :clicks="clicks"></DisplayPlayerClicks>
        <DisplayPlayerLatency :avg_latency="avg_latency"></DisplayPlayerLatency>
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
